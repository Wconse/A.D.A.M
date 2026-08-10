use crate::money_ledger::MoneyAccountBalances;
use crate::{
    ActorId, CorporateAction, DomainEvent, FirmId, Money, MoneyAccountId, MoneyPosting,
    MoneyTransaction, MoneyTransactionPurpose, ProjectId, RegionId, World, WorldError,
};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum InvestmentStatus {
    Planned,
    Building,
    Completed,
    Cancelled,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InvestmentProject {
    id: ProjectId,
    firm: FirmId,
    region: RegionId,
    budget: Money,
    spent: Money,
    duration_months: u32,
    elapsed_months: u32,
    capacity_batches: u64,
    status: InvestmentStatus,
}
impl InvestmentProject {
    /// Creates a funded construction project definition.
    /// # Errors
    /// Returns [`WorldError::InvalidInvestmentProject`] for non-positive budget, duration, or capacity.
    pub fn new(
        id: ProjectId,
        firm: FirmId,
        region: RegionId,
        budget: Money,
        duration_months: u32,
        capacity_batches: u64,
    ) -> Result<Self, WorldError> {
        if budget.minor_units() <= 0 || duration_months == 0 || capacity_batches == 0 {
            return Err(WorldError::InvalidInvestmentProject(
                "budget, duration, and capacity must be positive",
            ));
        }
        Ok(Self {
            id,
            firm,
            region,
            budget,
            spent: Money::default(),
            duration_months,
            elapsed_months: 0,
            capacity_batches,
            status: InvestmentStatus::Planned,
        })
    }
    #[must_use]
    pub const fn id(&self) -> ProjectId {
        self.id
    }
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn region(&self) -> RegionId {
        self.region
    }
    #[must_use]
    pub const fn budget(&self) -> Money {
        self.budget
    }
    #[must_use]
    pub const fn spent(&self) -> Money {
        self.spent
    }
    #[must_use]
    pub const fn status(&self) -> InvestmentStatus {
        self.status
    }
    #[must_use]
    pub const fn capacity_batches(&self) -> u64 {
        self.capacity_batches
    }
    #[must_use]
    pub const fn duration_months(&self) -> u32 {
        self.duration_months
    }
    #[must_use]
    pub const fn elapsed_months(&self) -> u32 {
        self.elapsed_months
    }
    fn advance(&mut self) -> Result<bool, WorldError> {
        if matches!(
            self.status,
            InvestmentStatus::Completed | InvestmentStatus::Cancelled
        ) {
            return Err(WorldError::InvalidInvestmentProject("project is closed"));
        }
        self.status = InvestmentStatus::Building;
        self.elapsed_months += 1;
        let total = self.budget.minor_units();
        let target = if self.elapsed_months >= self.duration_months {
            total
        } else {
            i64::try_from(
                i128::from(total) * i128::from(self.elapsed_months)
                    / i128::from(self.duration_months),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("project spending"))?
        };
        self.spent = Money::from_minor_units(target);
        if self.elapsed_months >= self.duration_months {
            self.status = InvestmentStatus::Completed;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
impl World {
    /// Commits firm cash to future capacity under investment-scoped authority.
    ///
    /// The money leaves the firm's balance the moment the works are ordered,
    /// which is what makes an expansion a real decision with a real cost.
    ///
    /// # Errors
    /// Returns an authority, unknown-firm, or insufficient-cash error without mutation.
    pub fn commit_firm_investment(
        &mut self,
        actor: ActorId,
        firm: FirmId,
        amount: Money,
    ) -> Result<(), WorldError> {
        if !self.firms().contains_key(&firm) {
            return Err(WorldError::UnknownFirm(firm));
        }
        if !self.can_perform_corporate_action(actor, firm, CorporateAction::ProposeMajorInvestment)
        {
            return Err(WorldError::UnauthorizedFirmControl { actor, firm });
        }
        self.execute_investment_commitment(firm, amount)
    }

    /// Launches a project using previously committed firm investment funds.
    /// # Errors
    /// Returns an error for duplicates, unknown references, or insufficient commitment.
    pub fn launch_investment_project(
        &mut self,
        mut project: InvestmentProject,
    ) -> Result<(), WorldError> {
        if self.investment_projects.contains_key(&project.id()) {
            return Err(WorldError::DuplicateInvestmentProject(project.id()));
        }
        if !self.firms().contains_key(&project.firm()) {
            return Err(WorldError::UnknownFirm(project.firm()));
        }
        if !self.regions().contains_key(&project.region()) {
            return Err(WorldError::UnknownRegion(project.region()));
        }
        let committed = self
            .committed_investments()
            .get(&project.firm())
            .copied()
            .unwrap_or_default()
            .minor_units();
        let budget = project.budget().minor_units();
        if committed < budget {
            return Err(WorldError::InsufficientCommittedInvestment(project.firm()));
        }
        let shares = self.plan_regional_capital_shares(project.region(), project.budget());
        if shares.is_empty() {
            return Err(WorldError::InvalidProduction(
                "capital spending needs local households to build the works",
            ));
        }
        let mut postings = vec![MoneyPosting::decrease(
            MoneyAccountId::FirmInvestmentEscrow(project.firm()),
            project.budget(),
        )];
        for &(cohort, share) in &shares {
            postings.push(MoneyPosting::increase(
                MoneyAccountId::HouseholdLiquid(cohort),
                share,
            ));
        }
        let transaction = MoneyTransaction::new(
            MoneyTransactionPurpose::InvestmentOutlay {
                firm: project.firm(),
                project: project.id(),
                region: project.region(),
            },
            postings,
        )?;
        let journal_entries = self.prepare_money_journal_entries(&[transaction.clone()])?;
        MoneyAccountBalances::InvestmentEscrowAndHouseholds {
            committed_investments: &mut self.committed_investments,
            cohorts: &mut self.cohorts,
        }
        .settle(&transaction)?;
        project.status = InvestmentStatus::Building;
        self.append_money_journal_entries(journal_entries);
        self.events.append(
            self.date,
            DomainEvent::InvestmentProjectLaunched {
                project: project.id(),
                firm: project.firm(),
                budget: project.budget(),
            },
        );
        for (cohort, amount) in shares {
            self.events.append(
                self.date,
                DomainEvent::CapitalOutlayDistributed {
                    firm: project.firm(),
                    cohort,
                    amount,
                },
            );
        }
        self.investment_projects.insert(project.id(), project);
        Ok(())
    }
    /// Advances one construction month and adds capacity on completion.
    /// # Errors
    /// Returns an error for an unknown or closed project.
    pub fn advance_investment_project(&mut self, id: ProjectId) -> Result<(), WorldError> {
        let completed = self
            .investment_projects
            .get_mut(&id)
            .ok_or(WorldError::UnknownInvestmentProject(id))?
            .advance()?;
        self.events.append(
            self.date,
            DomainEvent::InvestmentProjectAdvanced { project: id },
        );
        if completed {
            let project = self
                .investment_projects
                .get(&id)
                .ok_or(WorldError::UnknownInvestmentProject(id))?;
            self.firms
                .get_mut(&project.firm())
                .ok_or(WorldError::UnknownFirm(project.firm()))?
                .add_capacity(project.capacity_batches())?;
            self.events.append(
                self.date,
                DomainEvent::InvestmentProjectCompleted {
                    project: id,
                    firm: project.firm(),
                },
            );
        }
        Ok(())
    }
    #[must_use]
    pub fn investment_projects(&self) -> &BTreeMap<ProjectId, InvestmentProject> {
        &self.investment_projects
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn investment_escrow_reserves_then_pays_builders_without_moving_total_money() {
        let mut world = crate::capacity_investment::tests::hungry_market_world();
        let actor = ActorId::new(1);
        let firm = FirmId::new(1);
        let amount = Money::from_minor_units(100);
        let opening_stock = world.observed_money_stock().expect("opening stock");
        let opening_cash = world.firms()[&firm].cash();

        world
            .commit_firm_investment(actor, firm, amount)
            .expect("reserve investment");

        assert_eq!(
            world.firms()[&firm].cash(),
            Money::from_minor_units(opening_cash.minor_units() - amount.minor_units())
        );
        assert_eq!(world.committed_investments()[&firm], amount);
        assert_eq!(
            world
                .observed_money_stock()
                .expect("reserved stock")
                .total()
                .expect("reserved total"),
            opening_stock.total().expect("opening total")
        );
        assert!(matches!(
            world.money_journal().last().map(crate::MoneyJournalEntry::purpose),
            Some(MoneyTransactionPurpose::InvestmentReservation { firm: reserved })
                if reserved == firm
        ));

        let household_before = world
            .observed_money_stock()
            .expect("stock before launch")
            .households;
        world
            .launch_investment_project(
                InvestmentProject::new(ProjectId::new(999), firm, RegionId::new(1), amount, 3, 1)
                    .expect("project"),
            )
            .expect("launch project");

        assert_eq!(
            world.committed_investments()[&firm],
            Money::default(),
            "launch spends the reserved balance rather than deleting a promise"
        );
        assert_eq!(
            world
                .observed_money_stock()
                .expect("stock after launch")
                .households
                .minor_units()
                - household_before.minor_units(),
            amount.minor_units()
        );
        assert_eq!(
            world
                .observed_money_stock()
                .expect("closing stock")
                .total()
                .expect("closing total"),
            opening_stock.total().expect("opening total")
        );
        let outlay = world.money_journal().last().expect("outlay entry");
        assert!(matches!(
            outlay.purpose(),
            MoneyTransactionPurpose::InvestmentOutlay {
                firm: paid_firm,
                project,
                region,
            } if paid_firm == firm
                && project == ProjectId::new(999)
                && region == RegionId::new(1)
        ));
        assert!(outlay.postings().iter().any(|posting| {
            posting.account() == MoneyAccountId::FirmInvestmentEscrow(firm)
                && posting.direction() == crate::MoneyPostingDirection::Decrease
                && posting.amount() == amount
        }));
    }

    #[test]
    fn rejected_investment_reservation_or_release_is_atomic() {
        let mut world = crate::capacity_investment::tests::hungry_market_world();
        let actor = ActorId::new(1);
        let firm = FirmId::new(1);
        let cash = world.firms()[&firm].cash();
        let before_reservation = world.clone();

        let error = world
            .commit_firm_investment(actor, firm, Money::from_minor_units(cash.minor_units() + 1))
            .expect_err("an overdrawn reservation must fail");
        assert!(matches!(error, WorldError::InsufficientFirmCash(failed) if failed == firm));
        assert_eq!(world, before_reservation);

        world
            .commit_firm_investment(actor, firm, Money::from_minor_units(100))
            .expect("reserve investment");
        let before_release = world.clone();
        let error = world
            .launch_investment_project(
                InvestmentProject::new(
                    ProjectId::new(999),
                    firm,
                    RegionId::new(1),
                    Money::from_minor_units(101),
                    3,
                    1,
                )
                .expect("project"),
            )
            .expect_err("a project cannot spend more than its escrow");
        assert!(matches!(
            error,
            WorldError::InsufficientCommittedInvestment(failed) if failed == firm
        ));
        assert_eq!(world, before_release);
    }

    #[test]
    fn project_spending_finishes_exactly() {
        let mut p = InvestmentProject::new(
            ProjectId::new(1),
            FirmId::new(1),
            RegionId::new(1),
            Money::from_minor_units(100),
            3,
            5,
        )
        .expect("project");
        assert!(!p.advance().expect("month"));
        assert!(!p.advance().expect("month"));
        assert!(p.advance().expect("month"));
        assert_eq!(p.spent(), Money::from_minor_units(100));
        assert_eq!(p.status(), InvestmentStatus::Completed);
    }
}
