use crate::money_ledger::MoneyAccountBalances;
use crate::{
    ActorId, BasisPoints, CorporateAction, CorporateRole, DomainEvent, FirmId, Money,
    MoneyAccountId, MoneyPosting, MoneyTransaction, MoneyTransactionPurpose, ResolutionId, World,
    WorldError,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BoardVote {
    For,
    Against,
    Abstain,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ResolutionStatus {
    Open,
    Approved,
    Rejected,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BoardMandate {
    AppointExecutive { actor: ActorId, role: CorporateRole },
    RemoveExecutive { actor: ActorId, role: CorporateRole },
    DeclareDividend { amount: Money },
    CommitInvestment { amount: Money },
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoardResolution {
    id: ResolutionId,
    firm: FirmId,
    action: CorporateAction,
    proposer: ActorId,
    votes: BTreeMap<ActorId, BoardVote>,
    status: ResolutionStatus,
    mandate: Option<BoardMandate>,
    executed: bool,
}
impl BoardResolution {
    #[must_use]
    pub const fn new(
        id: ResolutionId,
        firm: FirmId,
        action: CorporateAction,
        proposer: ActorId,
    ) -> Self {
        Self {
            id,
            firm,
            action,
            proposer,
            votes: BTreeMap::new(),
            status: ResolutionStatus::Open,
            mandate: None,
            executed: false,
        }
    }
    #[must_use]
    pub const fn with_mandate(mut self, mandate: BoardMandate) -> Self {
        self.mandate = Some(mandate);
        self
    }
    #[must_use]
    pub const fn mandate(&self) -> Option<BoardMandate> {
        self.mandate
    }
    #[must_use]
    pub const fn executed(&self) -> bool {
        self.executed
    }
    fn mark_executed(&mut self) {
        self.executed = true;
    }
    #[must_use]
    pub const fn id(&self) -> ResolutionId {
        self.id
    }
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn action(&self) -> CorporateAction {
        self.action
    }
    #[must_use]
    pub const fn proposer(&self) -> ActorId {
        self.proposer
    }
    #[must_use]
    pub fn votes(&self) -> &BTreeMap<ActorId, BoardVote> {
        &self.votes
    }
    #[must_use]
    pub const fn status(&self) -> ResolutionStatus {
        self.status
    }
    /// Records or replaces a director's vote while the resolution is open.
    /// # Errors
    /// Returns an error for a non-director or closed resolution.
    pub fn cast_vote(
        &mut self,
        directors: &BTreeSet<ActorId>,
        actor: ActorId,
        vote: BoardVote,
    ) -> Result<(), &'static str> {
        if self.status != ResolutionStatus::Open {
            return Err("resolution is closed");
        }
        if !directors.contains(&actor) {
            return Err("actor is not a director");
        }
        self.votes.insert(actor, vote);
        Ok(())
    }
    pub fn close(
        &mut self,
        directors: &BTreeSet<ActorId>,
        quorum: BasisPoints,
        approval: BasisPoints,
    ) -> ResolutionStatus {
        let total = directors.len() as u64;
        let cast = self.votes.len() as u64;
        if total == 0 || cast * 10_000 < total * u64::from(quorum.get()) {
            self.status = ResolutionStatus::Rejected;
            return self.status;
        }
        let for_votes = self
            .votes
            .values()
            .filter(|v| **v == BoardVote::For)
            .count() as u64;
        let decisive = self
            .votes
            .values()
            .filter(|v| **v != BoardVote::Abstain)
            .count() as u64;
        self.status = if decisive > 0 && for_votes * 10_000 >= decisive * u64::from(approval.get())
        {
            ResolutionStatus::Approved
        } else {
            ResolutionStatus::Rejected
        };
        self.status
    }
}
impl World {
    /// Proposes a board resolution through executive or director authority.
    /// # Errors
    /// Returns an error for duplicate IDs, unknown firms, or unauthorized proposers.
    pub fn propose_board_resolution(
        &mut self,
        resolution: BoardResolution,
    ) -> Result<(), WorldError> {
        if self.board_resolutions.contains_key(&resolution.id()) {
            return Err(WorldError::DuplicateBoardResolution(resolution.id()));
        }
        if !self.firms().contains_key(&resolution.firm()) {
            return Err(WorldError::UnknownFirm(resolution.firm()));
        }
        let authorized = self.firm_appointments().contains_key(&(
            resolution.firm(),
            resolution.proposer(),
            CorporateRole::ChiefExecutive,
        )) || self.firm_appointments().contains_key(&(
            resolution.firm(),
            resolution.proposer(),
            CorporateRole::BoardDirector,
        ));
        if !authorized {
            return Err(WorldError::UnauthorizedBoardAction(resolution.proposer()));
        }
        self.events.append(
            self.date,
            DomainEvent::BoardResolutionProposed {
                resolution: resolution.id(),
                firm: resolution.firm(),
                proposer: resolution.proposer(),
            },
        );
        self.board_resolutions.insert(resolution.id(), resolution);
        Ok(())
    }
    /// Casts a vote using current board membership.
    /// # Errors
    /// Returns an error for unknown resolution, non-director, or closed vote.
    pub fn cast_board_vote(
        &mut self,
        id: ResolutionId,
        actor: ActorId,
        vote: BoardVote,
    ) -> Result<(), WorldError> {
        let firm = self
            .board_resolutions
            .get(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .firm();
        let directors = self.directors(firm);
        self.board_resolutions
            .get_mut(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .cast_vote(&directors, actor, vote)
            .map_err(WorldError::InvalidBoardVote)?;
        self.events.append(
            self.date,
            DomainEvent::BoardVoteCast {
                resolution: id,
                actor,
                vote,
            },
        );
        Ok(())
    }
    /// Closes a resolution under current board membership and thresholds.
    /// # Errors
    /// Returns an error for an unknown resolution.
    pub fn close_board_resolution(
        &mut self,
        id: ResolutionId,
        quorum: BasisPoints,
        approval: BasisPoints,
    ) -> Result<ResolutionStatus, WorldError> {
        let firm = self
            .board_resolutions
            .get(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .firm();
        let directors = self.directors(firm);
        let status = self
            .board_resolutions
            .get_mut(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .close(&directors, quorum, approval);
        self.events.append(
            self.date,
            DomainEvent::BoardResolutionClosed {
                resolution: id,
                status,
            },
        );
        Ok(status)
    }
    /// Executes an approved, unexecuted personnel mandate exactly once.
    /// # Errors
    /// Returns a structured error without mutation when status, mandate, or appointment is invalid.
    pub fn execute_board_resolution(&mut self, id: ResolutionId) -> Result<(), WorldError> {
        let resolution = self
            .board_resolutions
            .get(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?;
        if resolution.status() != ResolutionStatus::Approved {
            return Err(WorldError::BoardResolutionNotApproved(id));
        }
        if resolution.executed() {
            return Err(WorldError::BoardResolutionAlreadyExecuted(id));
        }
        let firm = resolution.firm();
        let mandate = resolution
            .mandate()
            .ok_or(WorldError::MissingBoardMandate(id))?;
        match mandate {
            BoardMandate::AppointExecutive { actor, role } => {
                self.register_firm_appointment(crate::FirmAppointment::new(firm, actor, role))?;
            }
            BoardMandate::RemoveExecutive { actor, role } => {
                if self
                    .firm_appointments
                    .remove(&(firm, actor, role))
                    .is_none()
                {
                    return Err(WorldError::InvalidBoardExecution(
                        "appointment does not exist",
                    ));
                }
            }
            BoardMandate::DeclareDividend { amount } => self.execute_dividend(firm, amount)?,
            BoardMandate::CommitInvestment { amount } => {
                self.execute_investment_commitment(firm, amount)?;
            }
        }
        self.board_resolutions
            .get_mut(&id)
            .ok_or(WorldError::UnknownBoardResolution(id))?
            .mark_executed();
        self.events.append(
            self.date,
            DomainEvent::BoardResolutionExecuted { resolution: id },
        );
        Ok(())
    }
    fn execute_dividend(&mut self, firm: FirmId, amount: Money) -> Result<(), WorldError> {
        let amount_value = amount.minor_units();
        if amount_value <= 0 {
            return Err(WorldError::InvalidBoardExecution(
                "dividend must be positive",
            ));
        }
        let cash = self.firms()[&firm].cash().minor_units();
        if cash < amount_value {
            return Err(WorldError::InsufficientFirmCash(firm));
        }
        let stakes: Vec<_> = self
            .ownership_stakes()
            .values()
            .filter(|stake| stake.firm() == firm)
            .copied()
            .collect();
        let total: u32 = stakes
            .iter()
            .map(|stake| u32::from(stake.economic_rights().get()))
            .sum();
        if total != 10_000 {
            return Err(WorldError::InvalidBoardExecution(
                "economic ownership must total 100% before dividends",
            ));
        }
        let mut payouts = Vec::new();
        let mut assigned = 0_i64;
        for stake in &stakes {
            let value =
                i128::from(amount_value) * i128::from(stake.economic_rights().get()) / 10_000;
            let value = i64::try_from(value)
                .map_err(|_| WorldError::ArithmeticOverflow("dividend payout"))?;
            assigned = assigned
                .checked_add(value)
                .ok_or(WorldError::ArithmeticOverflow("dividend total"))?;
            payouts.push((stake.owner(), value));
        }
        let remainder = amount_value - assigned;
        if let Some(first) = payouts.first_mut() {
            first.1 += remainder;
        }
        let mut postings = vec![MoneyPosting::decrease(
            MoneyAccountId::FirmOperating(firm),
            amount,
        )];
        for &(actor, value) in &payouts {
            if value > 0 {
                postings.push(MoneyPosting::increase(
                    MoneyAccountId::ActorLiquid(actor),
                    Money::from_minor_units(value),
                ));
            }
        }
        let transaction = MoneyTransaction::new(
            MoneyTransactionPurpose::OwnerDistribution { firm },
            postings,
        )?;
        let journal_entries = self.prepare_money_journal_entries(&[transaction.clone()])?;
        MoneyAccountBalances::FirmsAndActors {
            firms: &mut self.firms,
            actor_cash: &mut self.actor_cash,
        }
        .settle(&transaction)?;
        self.append_money_journal_entries(journal_entries);
        self.events
            .append(self.date, DomainEvent::DividendPaid { firm, amount });
        Ok(())
    }
    /// What a firm's own memory says it must keep back before anyone takes
    /// profit out of it.
    ///
    /// Obligations are a single month's bill, and a firm that pays out
    /// everything above one month is betting that no month will ever be worse
    /// than the one it is in. The bet is lost eventually: one thin month of
    /// selling arrives, the till cannot cover wages, arrears begin, and the
    /// households whose wages went unpaid stop being customers.
    ///
    /// The reserve is evidence, not a rule of thumb: for every month the firm
    /// still remembers, the gap between what running the business costs and
    /// what it actually sold that month. The worst of those gaps is what the
    /// firm holds back. A firm that has never known a bad month keeps nothing
    /// back and pays exactly as it did before; a firm that has just lived
    /// through one keeps enough to live through it again. The memory is
    /// bounded, so an old scar fades once a year of good months has replaced
    /// it.
    fn observed_prudence_reserve(&self, firm: FirmId) -> Result<i128, WorldError> {
        let cost = self
            .observed_firm_obligation_terms(firm)?
            .recurring_monthly_cost();
        let Some(history) = self.firm_operating_history.get(&firm) else {
            return Ok(0);
        };
        Ok(history.iter().fold(0_i128, |worst, observation| {
            let shortfall =
                cost.saturating_sub(i128::from(observation.sales_revenue().minor_units()));
            worst.max(shortfall)
        }))
    }

    /// At the close of a year every solvent firm hands its owners the share of
    /// free cash their own policy claims.
    ///
    /// Free cash is what the till holds beyond the wages it owes, the arrears
    /// it has not paid, the inputs its production target needs, and the works
    /// its own unmet demand justifies. A firm therefore never pays out of money
    /// it owes, and a firm with somewhere to grow keeps what the growing costs.
    ///
    /// Without this, profit piles up inside firms for ever. Owners never
    /// accumulate the means to found anything, so a region that loses its firms
    /// can never grow new ones, and household savings can never become capital.
    pub(crate) fn execute_observed_owner_distributions(&mut self) -> Result<(), WorldError> {
        let firms: Vec<FirmId> = self.firms.keys().copied().collect();
        for firm in firms {
            if self.is_firm_insolvent(firm) {
                continue;
            }
            let Some(policy) = self.firm_policies.get(&firm).copied() else {
                continue;
            };
            let share = i128::from(policy.dividend().get());
            if share == 0 {
                continue;
            }
            // A firm with no trading record has no evidence of what it owes,
            // and no evidence is no ground for taking money out of it.
            if self.observed_operating_baseline(firm)?.is_none() {
                continue;
            }
            let economic: u32 = self
                .ownership_stakes()
                .values()
                .filter(|stake| stake.firm() == firm)
                .map(|stake| u32::from(stake.economic_rights().get()))
                .sum();
            if economic != 10_000 {
                continue;
            }
            let free_cash = self.observed_firm_obligation_terms(firm)?.free_cash()?;
            if free_cash <= 0 {
                continue;
            }
            // Before profit, prudence: the firm keeps what its own worst
            // remembered month would cost it.
            let prudent_cash = free_cash.saturating_sub(self.observed_prudence_reserve(firm)?);
            // A firm may hand over what it has earned and never the capital it
            // was founded with. A full till proves nothing on its own: the
            // opening endowment, an owner's injection against arrears and a
            // lender's principal all sit in the same till as trading income.
            // Only the trading income was ever profit.
            let earned = i128::from(
                self.firm_retained_earnings
                    .get(&firm)
                    .copied()
                    .unwrap_or_default()
                    .minor_units(),
            );
            let distributable = prudent_cash.min(earned);
            if distributable <= 0 {
                continue;
            }
            let claimed = distributable
                .checked_mul(share)
                .ok_or(WorldError::ArithmeticOverflow("owner distribution"))?
                / 10_000;
            let Ok(amount) = i64::try_from(claimed) else {
                return Err(WorldError::ArithmeticOverflow("owner distribution"));
            };
            if amount <= 0 {
                continue;
            }
            self.execute_dividend(firm, Money::from_minor_units(amount))?;
            // What has been handed over has been spent: it cannot be
            // distributed a second time in a later year.
            let retained = self.firm_retained_earnings.entry(firm).or_default();
            *retained = Money::from_minor_units(retained.minor_units().saturating_sub(amount));
        }
        Ok(())
    }

    pub(crate) fn execute_investment_commitment(
        &mut self,
        firm: FirmId,
        amount: Money,
    ) -> Result<(), WorldError> {
        let value = amount.minor_units();
        if value <= 0 {
            return Err(WorldError::InvalidBoardExecution(
                "investment must be positive",
            ));
        }
        let cash = self.firms()[&firm].cash().minor_units();
        if cash < value {
            return Err(WorldError::InsufficientFirmCash(firm));
        }
        let transaction = MoneyTransaction::transfer(
            MoneyTransactionPurpose::InvestmentReservation { firm },
            MoneyAccountId::FirmOperating(firm),
            MoneyAccountId::FirmInvestmentEscrow(firm),
            amount,
        )?;
        let journal_entries = self.prepare_money_journal_entries(&[transaction.clone()])?;
        MoneyAccountBalances::FirmsAndInvestmentEscrow {
            firms: &mut self.firms,
            committed_investments: &mut self.committed_investments,
        }
        .settle(&transaction)?;
        self.append_money_journal_entries(journal_entries);
        self.events
            .append(self.date, DomainEvent::InvestmentCommitted { firm, amount });
        Ok(())
    }
    #[must_use]
    pub fn actor_cash(&self) -> &BTreeMap<ActorId, Money> {
        &self.actor_cash
    }
    #[must_use]
    pub fn committed_investments(&self) -> &BTreeMap<FirmId, Money> {
        &self.committed_investments
    }
    #[must_use]
    pub fn board_resolutions(&self) -> &BTreeMap<ResolutionId, BoardResolution> {
        &self.board_resolutions
    }
    fn directors(&self, firm: FirmId) -> BTreeSet<ActorId> {
        self.firm_appointments()
            .keys()
            .filter(|(f, _, role)| *f == firm && *role == CorporateRole::BoardDirector)
            .map(|(_, actor, _)| *actor)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dividend_policy_world() -> World {
        let mut world = crate::capacity_investment::tests::hungry_market_world();
        world
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(1),
                crate::FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("marketing"),
                    BasisPoints::new(0).expect("reinvestment"),
                    BasisPoints::new(3_000).expect("dividend"),
                )
                .expect("policy"),
            )
            .expect("set policy");
        world
    }

    #[test]
    fn owners_are_paid_out_of_earnings_and_never_out_of_the_capital() {
        let mut world = dividend_policy_world();
        let firm = FirmId::new(1);
        let owner = ActorId::new(1);

        // The bakery was founded with a full till and has since traded for one
        // small month. Only that month was ever profit.
        world
            .record_firm_sale(firm, Money::from_minor_units(100))
            .expect("a month of trade");
        world
            .capture_monthly_firm_observation(firm)
            .expect("close the month");

        let free_cash = world
            .observed_firm_obligation_terms(firm)
            .expect("terms")
            .free_cash()
            .expect("free cash");
        let capital_claim = free_cash * 3_000 / 10_000;
        let before = world
            .actor_cash()
            .get(&owner)
            .copied()
            .unwrap_or_default()
            .minor_units();

        world
            .execute_observed_owner_distributions()
            .expect("distributions");

        let paid = i128::from(
            world
                .actor_cash()
                .get(&owner)
                .copied()
                .unwrap_or_default()
                .minor_units()
                - before,
        );
        assert!(paid > 0, "a firm that earned something may pay something");
        assert!(
            paid <= 100,
            "nothing beyond what the firm earned may leave it: paid {paid}"
        );
        assert!(
            paid < capital_claim,
            "the old rule would have handed over the founding capital: {capital_claim}"
        );
    }

    #[test]
    fn a_month_without_sales_is_kept_back_from_the_owners() {
        let mut world = dividend_policy_world();
        let firm = FirmId::new(1);
        let owner = ActorId::new(1);

        // The bakery has a full till, a wage bill, and the memory of one month
        // in which it sold nothing at all.
        world
            .capture_monthly_firm_observation(firm)
            .expect("a month with nothing sold");

        let cost = world
            .observed_firm_obligation_terms(firm)
            .expect("terms")
            .recurring_monthly_cost();
        let reserve = world.observed_prudence_reserve(firm).expect("reserve");
        assert!(cost > 0, "running this bakery costs it wages every month");
        assert_eq!(
            reserve, cost,
            "a month with nothing coming in costs a whole month of running, \
             and that is what the firm now knows to keep"
        );
        let free_cash = world
            .observed_firm_obligation_terms(firm)
            .expect("terms")
            .free_cash()
            .expect("free cash");
        // What the old rule would have handed over: a share of everything above
        // this single month's bill, with nothing kept for a worse one.
        let imprudent_claim = free_cash * 3_000 / 10_000;

        let before = world.actor_cash.get(&owner).copied().unwrap_or_default();
        world
            .execute_observed_owner_distributions()
            .expect("distributions");
        let paid = i128::from(
            world
                .actor_cash
                .get(&owner)
                .copied()
                .unwrap_or_default()
                .minor_units()
                - before.minor_units(),
        );
        assert!(
            paid < imprudent_claim,
            "the worst remembered month must stay in the till: paid {paid}, \
             the old rule would have paid {imprudent_claim}"
        );
        assert!(
            paid >= 0,
            "prudence holds money back; it never claws money in"
        );
    }

    #[test]
    fn paying_the_owners_moves_money_without_making_any() {
        let mut world = dividend_policy_world();
        let firm = FirmId::new(1);
        // The firm must have earned something before there is a payout whose
        // arithmetic is worth checking.
        world
            .record_firm_sale(firm, Money::from_minor_units(5_000))
            .expect("a month of trade");
        world
            .capture_monthly_firm_observation(firm)
            .expect("close the month");
        let before = world.observed_money_stock().expect("stock before");
        world
            .execute_observed_owner_distributions()
            .expect("distribution");
        assert!(world.money_journal().iter().any(|entry| {
            matches!(
                entry.purpose(),
                crate::MoneyTransactionPurpose::OwnerDistribution { firm: paid_firm }
                    if paid_firm == firm
            )
        }));
        let after = world.observed_money_stock().expect("stock after");
        assert!(
            after.owners.minor_units() > before.owners.minor_units(),
            "the owners must end the payout holding more than they did"
        );
        assert_eq!(
            after.firms.minor_units() + after.owners.minor_units(),
            before.firms.minor_units() + before.owners.minor_units(),
            "whatever the owners gained the firms must have lost, to the coin"
        );
        assert_eq!(
            before.total().expect("total before"),
            after.total().expect("total after"),
            "a payout moves money between hands; it never makes any"
        );
    }

    #[test]
    fn owner_distribution_assigns_the_rounding_remainder_to_the_first_owner() {
        let mut world = dividend_policy_world();
        let firm = FirmId::new(1);
        let first = ActorId::new(1);
        let second = ActorId::new(2);
        world
            .register_actor(
                crate::Actor::new(second, "Minority owner", crate::RegionId::new(1), 1981)
                    .expect("actor"),
            )
            .expect("register actor");
        world.ownership_stakes.clear();
        world
            .register_ownership_stake(crate::OwnershipStake::new(
                firm,
                first,
                BasisPoints::new(9_999).expect("rights"),
                BasisPoints::new(9_999).expect("rights"),
            ))
            .expect("first stake");
        world
            .register_ownership_stake(crate::OwnershipStake::new(
                firm,
                second,
                BasisPoints::new(1).expect("rights"),
                BasisPoints::new(1).expect("rights"),
            ))
            .expect("second stake");
        let journal_len = world.money_journal().len();

        world
            .execute_dividend(firm, Money::from_minor_units(1))
            .expect("one-minor-unit distribution");

        assert_eq!(
            world.actor_cash.get(&first),
            Some(&Money::from_minor_units(1))
        );
        assert_eq!(world.actor_cash.get(&second), None);
        let entry = &world.money_journal()[journal_len];
        assert_eq!(entry.postings().len(), 2);
        assert!(entry.postings().iter().any(|posting| {
            posting.account() == MoneyAccountId::ActorLiquid(first)
                && posting.direction() == crate::MoneyPostingDirection::Increase
                && posting.amount() == Money::from_minor_units(1)
        }));
        assert!(
            !entry
                .postings()
                .iter()
                .any(|posting| posting.account() == MoneyAccountId::ActorLiquid(second))
        );
    }

    #[test]
    fn rejected_owner_distribution_leaves_balances_and_journal_unchanged() {
        let mut world = dividend_policy_world();
        let firm = FirmId::new(1);
        let owner = ActorId::new(1);
        world.ownership_stakes.clear();
        world
            .register_ownership_stake(crate::OwnershipStake::new(
                firm,
                owner,
                BasisPoints::new(5_000).expect("rights"),
                BasisPoints::new(5_000).expect("rights"),
            ))
            .expect("incomplete ownership is representable during setup");
        let firm_cash = world.firms[&firm].cash();
        let actor_cash = world.actor_cash.clone();
        let journal_len = world.money_journal().len();

        let error = world
            .execute_dividend(firm, Money::from_minor_units(1))
            .expect_err("incomplete ownership cannot receive a distribution");

        assert!(matches!(error, WorldError::InvalidBoardExecution(_)));
        assert_eq!(world.firms[&firm].cash(), firm_cash);
        assert_eq!(world.actor_cash, actor_cash);
        assert_eq!(world.money_journal().len(), journal_len);
    }

    #[test]
    fn earned_profit_reaches_the_owners_who_carry_the_firm() {
        let mut world = dividend_policy_world();
        let owner = ActorId::new(1);
        let firm = FirmId::new(1);
        assert_eq!(
            world.actor_cash.get(&owner).copied().unwrap_or_default(),
            Money::default(),
            "the owner starts the test with nothing of its own"
        );
        // A month of real trade, worth far more than the work it took. This is
        // profit, not the capital the bakery was founded with.
        world
            .record_firm_sale(firm, Money::from_minor_units(5_000))
            .expect("a month of trade");
        world
            .capture_monthly_firm_observation(firm)
            .expect("close the month");
        world
            .execute_observed_owner_distributions()
            .expect("distributions");
        assert!(
            world
                .actor_cash
                .get(&owner)
                .copied()
                .unwrap_or_default()
                .minor_units()
                > 0,
            "a profitable firm must hand its owner the share its policy claims, \
             otherwise profit piles up inside firms for ever and nobody can ever found another"
        );
    }

    #[test]
    fn a_firm_that_owes_more_than_it_holds_pays_its_owners_nothing() {
        let mut world = dividend_policy_world();
        let owner = ActorId::new(1);
        let firm = FirmId::new(1);
        world.advance_economic_year().expect("year");
        // The till is emptied down to a coin, while the wages, the inputs and
        // the works the firm's own demand justifies are all still owed.
        world
            .firms
            .get_mut(&firm)
            .expect("firm")
            .set_cash(Money::from_minor_units(1));
        let before = world.actor_cash.get(&owner).copied().unwrap_or_default();
        world
            .execute_observed_owner_distributions()
            .expect("distributions");
        assert_eq!(
            world.actor_cash.get(&owner).copied().unwrap_or_default(),
            before,
            "a firm must not pay its owners out of money it already owes"
        );
        assert_eq!(
            world.firms()[&firm].cash(),
            Money::from_minor_units(1),
            "and the till it could not spare must be untouched"
        );
    }

    #[test]
    fn quorum_and_majority_are_both_required() {
        let directors = BTreeSet::from([ActorId::new(1), ActorId::new(2), ActorId::new(3)]);
        let mut r = BoardResolution::new(
            ResolutionId::new(1),
            FirmId::new(1),
            CorporateAction::ApproveMajorInvestment,
            ActorId::new(1),
        );
        r.cast_vote(&directors, ActorId::new(1), BoardVote::For)
            .expect("vote");
        assert_eq!(
            r.close(
                &directors,
                BasisPoints::new(6667).expect("q"),
                BasisPoints::new(5001).expect("a")
            ),
            ResolutionStatus::Rejected
        );
    }
    #[test]
    fn qualified_vote_approves() {
        let directors = BTreeSet::from([ActorId::new(1), ActorId::new(2), ActorId::new(3)]);
        let mut r = BoardResolution::new(
            ResolutionId::new(1),
            FirmId::new(1),
            CorporateAction::DeclareDividend,
            ActorId::new(1),
        );
        for id in [1, 2, 3] {
            r.cast_vote(
                &directors,
                ActorId::new(id),
                if id < 3 {
                    BoardVote::For
                } else {
                    BoardVote::Against
                },
            )
            .expect("vote");
        }
        assert_eq!(
            r.close(
                &directors,
                BasisPoints::new(6667).expect("q"),
                BasisPoints::new(5001).expect("a")
            ),
            ResolutionStatus::Approved
        );
    }
}
