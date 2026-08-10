use std::collections::BTreeMap;

use crate::{
    ActorId, CohortId, Country, CountryId, Firm, FirmId, GoodId, GovernmentProgram,
    HouseholdCohort, Money, ProgramId, SimDate, World, WorldError,
};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum MoneyAccountOwner {
    Household(CohortId),
    Firm(FirmId),
    Country(CountryId),
    Program(ProgramId),
    Actor(ActorId),
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum MoneyAccountId {
    HouseholdLiquid(CohortId),
    FirmOperating(FirmId),
    FirmInvestmentEscrow(FirmId),
    CountryTreasury(CountryId),
    GovernmentProgramEscrow(ProgramId),
    ActorLiquid(ActorId),
}

impl MoneyAccountId {
    #[must_use]
    pub const fn owner(self) -> MoneyAccountOwner {
        match self {
            Self::HouseholdLiquid(cohort) => MoneyAccountOwner::Household(cohort),
            Self::FirmOperating(firm) | Self::FirmInvestmentEscrow(firm) => {
                MoneyAccountOwner::Firm(firm)
            }
            Self::CountryTreasury(country) => MoneyAccountOwner::Country(country),
            Self::GovernmentProgramEscrow(program) => MoneyAccountOwner::Program(program),
            Self::ActorLiquid(actor) => MoneyAccountOwner::Actor(actor),
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum MoneyPostingDirection {
    Decrease,
    Increase,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct MoneyPosting {
    account: MoneyAccountId,
    direction: MoneyPostingDirection,
    amount: Money,
}

impl MoneyPosting {
    #[must_use]
    pub const fn decrease(account: MoneyAccountId, amount: Money) -> Self {
        Self {
            account,
            direction: MoneyPostingDirection::Decrease,
            amount,
        }
    }

    #[must_use]
    pub const fn increase(account: MoneyAccountId, amount: Money) -> Self {
        Self {
            account,
            direction: MoneyPostingDirection::Increase,
            amount,
        }
    }

    #[must_use]
    pub const fn account(self) -> MoneyAccountId {
        self.account
    }

    #[must_use]
    pub const fn direction(self) -> MoneyPostingDirection {
        self.direction
    }

    #[must_use]
    pub const fn amount(self) -> Money {
        self.amount
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum MoneyTransactionKind {
    Transfer,
    Issue,
    Retire,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MoneyTransactionPurpose {
    Payroll {
        firm: FirmId,
        cohort: CohortId,
    },
    HouseholdPurchase {
        buyer: CohortId,
        seller: FirmId,
        good: GoodId,
        carrier: Option<FirmId>,
    },
    FirmPurchase {
        buyer: FirmId,
        seller: FirmId,
        good: GoodId,
        carrier: Option<FirmId>,
    },
    SalesTax {
        firm: FirmId,
        country: CountryId,
    },
    OwnerDistribution {
        firm: FirmId,
    },
    InvestmentReservation {
        firm: FirmId,
    },
    InvestmentOutlay {
        firm: FirmId,
        project: crate::ProjectId,
        region: crate::RegionId,
    },
    GovernmentProgramAppropriation {
        program: ProgramId,
        country: CountryId,
    },
    GovernmentProgramOutlay {
        program: ProgramId,
        country: CountryId,
    },
    EmergencyRelief {
        country: CountryId,
        cohort: CohortId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MoneyTransaction {
    kind: MoneyTransactionKind,
    purpose: MoneyTransactionPurpose,
    postings: Vec<MoneyPosting>,
}

impl MoneyTransaction {
    /// Builds and canonicalizes one balanced transfer.
    ///
    /// # Errors
    /// Rejects empty, non-positive, overflowing, or unbalanced postings.
    pub fn new(
        purpose: MoneyTransactionPurpose,
        postings: Vec<MoneyPosting>,
    ) -> Result<Self, WorldError> {
        Self::canonicalize(MoneyTransactionKind::Transfer, purpose, postings)
    }

    fn canonicalize(
        kind: MoneyTransactionKind,
        purpose: MoneyTransactionPurpose,
        postings: Vec<MoneyPosting>,
    ) -> Result<Self, WorldError> {
        let mut net = BTreeMap::<MoneyAccountId, i128>::new();
        for posting in postings {
            if posting.amount.minor_units() <= 0 {
                return Err(WorldError::InvalidMoneyTransaction(
                    "posting amount must be positive",
                ));
            }
            let signed = match posting.direction {
                MoneyPostingDirection::Decrease => -i128::from(posting.amount.minor_units()),
                MoneyPostingDirection::Increase => i128::from(posting.amount.minor_units()),
            };
            let current = net.get(&posting.account).copied().unwrap_or(0);
            net.insert(
                posting.account,
                current
                    .checked_add(signed)
                    .ok_or(WorldError::ArithmeticOverflow("money posting aggregation"))?,
            );
        }
        let mut decreases = 0_i128;
        let mut increases = 0_i128;
        let mut canonical = Vec::new();
        for (account, signed) in net {
            if signed == 0 {
                continue;
            }
            let (direction, amount) = if signed < 0 {
                decreases =
                    decreases
                        .checked_add(-signed)
                        .ok_or(WorldError::ArithmeticOverflow(
                            "money transaction decreases",
                        ))?;
                (MoneyPostingDirection::Decrease, -signed)
            } else {
                increases = increases
                    .checked_add(signed)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "money transaction increases",
                    ))?;
                (MoneyPostingDirection::Increase, signed)
            };
            canonical.push(MoneyPosting {
                account,
                direction,
                amount: Money::from_minor_units(
                    i64::try_from(amount)
                        .map_err(|_| WorldError::ArithmeticOverflow("canonical money posting"))?,
                ),
            });
        }
        match kind {
            MoneyTransactionKind::Transfer if decreases == 0 || increases == 0 => {
                return Err(WorldError::InvalidMoneyTransaction(
                    "transfer must decrease and increase named accounts",
                ));
            }
            MoneyTransactionKind::Transfer if decreases != increases => {
                return Err(WorldError::InvalidMoneyTransaction(
                    "total decreases must equal total increases",
                ));
            }
            MoneyTransactionKind::Issue if decreases != 0 || increases == 0 => {
                return Err(WorldError::InvalidMoneyTransaction(
                    "issue must only increase named accounts",
                ));
            }
            MoneyTransactionKind::Retire if decreases == 0 || increases != 0 => {
                return Err(WorldError::InvalidMoneyTransaction(
                    "retirement must only decrease named accounts",
                ));
            }
            _ => {}
        }
        Ok(Self {
            kind,
            purpose,
            postings: canonical,
        })
    }

    /// Builds a two-account transfer.
    ///
    /// # Errors
    /// Rejects a non-positive amount or a transfer that nets to no account movement.
    pub fn transfer(
        purpose: MoneyTransactionPurpose,
        from: MoneyAccountId,
        to: MoneyAccountId,
        amount: Money,
    ) -> Result<Self, WorldError> {
        Self::new(
            purpose,
            vec![
                MoneyPosting::decrease(from, amount),
                MoneyPosting::increase(to, amount),
            ],
        )
    }

    /// Issues new money into one named account.
    ///
    /// # Errors
    /// Rejects a non-positive amount.
    pub(crate) fn issue(
        purpose: MoneyTransactionPurpose,
        to: MoneyAccountId,
        amount: Money,
    ) -> Result<Self, WorldError> {
        Self::canonicalize(
            MoneyTransactionKind::Issue,
            purpose,
            vec![MoneyPosting::increase(to, amount)],
        )
    }

    /// Retires money from one named account.
    ///
    /// # Errors
    /// Rejects a non-positive amount.
    pub fn retire(
        purpose: MoneyTransactionPurpose,
        from: MoneyAccountId,
        amount: Money,
    ) -> Result<Self, WorldError> {
        Self::canonicalize(
            MoneyTransactionKind::Retire,
            purpose,
            vec![MoneyPosting::decrease(from, amount)],
        )
    }

    #[must_use]
    pub const fn kind(&self) -> MoneyTransactionKind {
        self.kind
    }

    #[must_use]
    pub const fn purpose(&self) -> MoneyTransactionPurpose {
        self.purpose
    }

    #[must_use]
    pub fn postings(&self) -> &[MoneyPosting] {
        &self.postings
    }

    fn preflight<F>(&self, mut balance_of: F) -> Result<BTreeMap<MoneyAccountId, Money>, WorldError>
    where
        F: FnMut(MoneyAccountId) -> Result<Money, WorldError>,
    {
        let mut closing = BTreeMap::new();
        for posting in &self.postings {
            let opening = balance_of(posting.account)?;
            if opening.minor_units() < 0 {
                return Err(WorldError::InvalidMoneyTransaction(
                    "opening account balance cannot be negative",
                ));
            }
            let signed = match posting.direction {
                MoneyPostingDirection::Decrease => -posting.amount.minor_units(),
                MoneyPostingDirection::Increase => posting.amount.minor_units(),
            };
            let value =
                opening
                    .minor_units()
                    .checked_add(signed)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "money account closing balance",
                    ))?;
            if value < 0 {
                return Err(WorldError::InvalidMoneyTransaction(
                    "transaction would overdraw an account",
                ));
            }
            closing.insert(posting.account, Money::from_minor_units(value));
        }
        Ok(closing)
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct MoneyJournalEntryId(u64);

impl MoneyJournalEntryId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MoneyJournalEntry {
    id: MoneyJournalEntryId,
    date: SimDate,
    kind: MoneyTransactionKind,
    purpose: MoneyTransactionPurpose,
    postings: Vec<MoneyPosting>,
}

impl MoneyJournalEntry {
    #[must_use]
    pub const fn id(&self) -> MoneyJournalEntryId {
        self.id
    }

    #[must_use]
    pub const fn date(&self) -> SimDate {
        self.date
    }

    #[must_use]
    pub const fn kind(&self) -> MoneyTransactionKind {
        self.kind
    }

    #[must_use]
    pub const fn purpose(&self) -> MoneyTransactionPurpose {
        self.purpose
    }

    #[must_use]
    pub fn postings(&self) -> &[MoneyPosting] {
        &self.postings
    }
}

pub(crate) enum MoneyAccountBalances<'a> {
    Firms {
        firms: &'a mut BTreeMap<FirmId, Firm>,
    },
    FirmsAndHouseholds {
        firms: &'a mut BTreeMap<FirmId, Firm>,
        cohorts: &'a mut BTreeMap<CohortId, HouseholdCohort>,
    },
    FirmsAndTreasuries {
        firms: &'a mut BTreeMap<FirmId, Firm>,
        countries: &'a mut BTreeMap<CountryId, Country>,
    },
    FirmsAndActors {
        firms: &'a mut BTreeMap<FirmId, Firm>,
        actor_cash: &'a mut BTreeMap<ActorId, Money>,
    },
    FirmsAndInvestmentEscrow {
        firms: &'a mut BTreeMap<FirmId, Firm>,
        committed_investments: &'a mut BTreeMap<FirmId, Money>,
    },
    InvestmentEscrowAndHouseholds {
        committed_investments: &'a mut BTreeMap<FirmId, Money>,
        cohorts: &'a mut BTreeMap<CohortId, HouseholdCohort>,
    },
    Programs {
        programs: &'a mut BTreeMap<ProgramId, GovernmentProgram>,
    },
    TreasuriesAndPrograms {
        countries: &'a mut BTreeMap<CountryId, Country>,
        programs: &'a mut BTreeMap<ProgramId, GovernmentProgram>,
    },
    ProgramsAndHouseholds {
        programs: &'a mut BTreeMap<ProgramId, GovernmentProgram>,
        cohorts: &'a mut BTreeMap<CohortId, HouseholdCohort>,
    },
    TreasuriesAndHouseholds {
        countries: &'a mut BTreeMap<CountryId, Country>,
        cohorts: &'a mut BTreeMap<CohortId, HouseholdCohort>,
    },
}

impl MoneyAccountBalances<'_> {
    pub(crate) fn settle(&mut self, transaction: &MoneyTransaction) -> Result<(), WorldError> {
        let closing = transaction.preflight(|account| self.balance(account))?;
        for (account, balance) in closing {
            self.set_balance(account, balance)?;
        }
        Ok(())
    }

    fn balance(&self, account: MoneyAccountId) -> Result<Money, WorldError> {
        match (self, account) {
            (
                Self::Firms { firms }
                | Self::FirmsAndHouseholds { firms, .. }
                | Self::FirmsAndTreasuries { firms, .. }
                | Self::FirmsAndActors { firms, .. }
                | Self::FirmsAndInvestmentEscrow { firms, .. },
                MoneyAccountId::FirmOperating(firm),
            ) => firms
                .get(&firm)
                .map(Firm::cash)
                .ok_or(WorldError::UnknownFirm(firm)),
            (
                Self::FirmsAndHouseholds { cohorts, .. }
                | Self::InvestmentEscrowAndHouseholds { cohorts, .. }
                | Self::ProgramsAndHouseholds { cohorts, .. }
                | Self::TreasuriesAndHouseholds { cohorts, .. },
                MoneyAccountId::HouseholdLiquid(cohort),
            ) => cohorts
                .get(&cohort)
                .map(HouseholdCohort::liquid_wealth)
                .ok_or(WorldError::UnknownCohort(cohort)),
            (
                Self::FirmsAndInvestmentEscrow {
                    committed_investments,
                    ..
                }
                | Self::InvestmentEscrowAndHouseholds {
                    committed_investments,
                    ..
                },
                MoneyAccountId::FirmInvestmentEscrow(firm),
            ) => Ok(committed_investments
                .get(&firm)
                .copied()
                .unwrap_or_default()),
            (
                Self::Programs { programs }
                | Self::TreasuriesAndPrograms { programs, .. }
                | Self::ProgramsAndHouseholds { programs, .. },
                MoneyAccountId::GovernmentProgramEscrow(program),
            ) => programs
                .get(&program)
                .map(GovernmentProgram::carryover_funding)
                .ok_or(WorldError::UnknownGovernmentProgram(program)),
            (
                Self::FirmsAndTreasuries { countries, .. }
                | Self::TreasuriesAndPrograms { countries, .. }
                | Self::TreasuriesAndHouseholds { countries, .. },
                MoneyAccountId::CountryTreasury(country),
            ) => countries
                .get(&country)
                .map(|row| row.indicators().treasury())
                .ok_or(WorldError::UnknownCountry(country)),
            (Self::FirmsAndActors { actor_cash, .. }, MoneyAccountId::ActorLiquid(actor)) => {
                Ok(actor_cash.get(&actor).copied().unwrap_or_default())
            }
            _ => Err(WorldError::InvalidMoneyTransaction(
                "account is unavailable in this settlement store",
            )),
        }
    }

    fn set_balance(&mut self, account: MoneyAccountId, balance: Money) -> Result<(), WorldError> {
        match (self, account) {
            (
                Self::Firms { firms }
                | Self::FirmsAndHouseholds { firms, .. }
                | Self::FirmsAndTreasuries { firms, .. }
                | Self::FirmsAndActors { firms, .. }
                | Self::FirmsAndInvestmentEscrow { firms, .. },
                MoneyAccountId::FirmOperating(firm),
            ) => {
                firms
                    .get_mut(&firm)
                    .ok_or(WorldError::UnknownFirm(firm))?
                    .set_cash(balance);
            }
            (
                Self::FirmsAndHouseholds { cohorts, .. }
                | Self::InvestmentEscrowAndHouseholds { cohorts, .. }
                | Self::ProgramsAndHouseholds { cohorts, .. }
                | Self::TreasuriesAndHouseholds { cohorts, .. },
                MoneyAccountId::HouseholdLiquid(cohort),
            ) => {
                cohorts
                    .get_mut(&cohort)
                    .ok_or(WorldError::UnknownCohort(cohort))?
                    .set_liquid_wealth(balance);
            }
            (
                Self::FirmsAndInvestmentEscrow {
                    committed_investments,
                    ..
                }
                | Self::InvestmentEscrowAndHouseholds {
                    committed_investments,
                    ..
                },
                MoneyAccountId::FirmInvestmentEscrow(firm),
            ) => {
                committed_investments.insert(firm, balance);
            }
            (
                Self::Programs { programs }
                | Self::TreasuriesAndPrograms { programs, .. }
                | Self::ProgramsAndHouseholds { programs, .. },
                MoneyAccountId::GovernmentProgramEscrow(program),
            ) => {
                programs
                    .get_mut(&program)
                    .ok_or(WorldError::UnknownGovernmentProgram(program))?
                    .set_carryover_funding(balance);
            }
            (
                Self::FirmsAndTreasuries { countries, .. }
                | Self::TreasuriesAndPrograms { countries, .. }
                | Self::TreasuriesAndHouseholds { countries, .. },
                MoneyAccountId::CountryTreasury(country),
            ) => {
                countries
                    .get_mut(&country)
                    .ok_or(WorldError::UnknownCountry(country))?
                    .indicators_mut()
                    .set_treasury(balance);
            }
            (Self::FirmsAndActors { actor_cash, .. }, MoneyAccountId::ActorLiquid(actor)) => {
                actor_cash.insert(actor, balance);
            }
            _ => {
                return Err(WorldError::InvalidMoneyTransaction(
                    "account is unavailable in this settlement store",
                ));
            }
        }
        Ok(())
    }
}

impl World {
    #[must_use]
    pub fn money_journal(&self) -> &[MoneyJournalEntry] {
        &self.money_journal
    }

    pub(crate) fn prepare_money_journal_entries(
        &self,
        transactions: &[MoneyTransaction],
    ) -> Result<Vec<MoneyJournalEntry>, WorldError> {
        let opening = u64::try_from(self.money_journal.len())
            .map_err(|_| WorldError::ArithmeticOverflow("money journal length"))?;
        transactions
            .iter()
            .enumerate()
            .map(|(offset, transaction)| {
                let offset = u64::try_from(offset)
                    .map_err(|_| WorldError::ArithmeticOverflow("money journal offset"))?;
                let sequence = opening
                    .checked_add(offset)
                    .and_then(|value| value.checked_add(1))
                    .ok_or(WorldError::ArithmeticOverflow("money journal sequence"))?;
                Ok(MoneyJournalEntry {
                    id: MoneyJournalEntryId(sequence),
                    date: self.date,
                    kind: transaction.kind,
                    purpose: transaction.purpose,
                    postings: transaction.postings.clone(),
                })
            })
            .collect()
    }

    pub(crate) fn append_money_journal_entries(&mut self, entries: Vec<MoneyJournalEntry>) {
        self.money_journal.extend(entries);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payroll(amount: i64) -> Result<MoneyTransaction, WorldError> {
        MoneyTransaction::transfer(
            MoneyTransactionPurpose::Payroll {
                firm: FirmId::new(1),
                cohort: CohortId::new(1),
            },
            MoneyAccountId::FirmOperating(FirmId::new(1)),
            MoneyAccountId::HouseholdLiquid(CohortId::new(1)),
            Money::from_minor_units(amount),
        )
    }

    #[test]
    fn unbalanced_transaction_is_rejected_at_construction() {
        let result = MoneyTransaction::new(
            MoneyTransactionPurpose::Payroll {
                firm: FirmId::new(1),
                cohort: CohortId::new(1),
            },
            vec![
                MoneyPosting::decrease(
                    MoneyAccountId::FirmOperating(FirmId::new(1)),
                    Money::from_minor_units(10),
                ),
                MoneyPosting::increase(
                    MoneyAccountId::HouseholdLiquid(CohortId::new(1)),
                    Money::from_minor_units(9),
                ),
            ],
        );
        assert!(matches!(
            result,
            Err(WorldError::InvalidMoneyTransaction(_))
        ));
    }

    #[test]
    fn insufficient_funds_fail_before_external_balances_change() {
        let transaction = payroll(11).expect("balanced transaction");
        let balances = BTreeMap::from([
            (
                MoneyAccountId::FirmOperating(FirmId::new(1)),
                Money::from_minor_units(10),
            ),
            (
                MoneyAccountId::HouseholdLiquid(CohortId::new(1)),
                Money::from_minor_units(2),
            ),
        ]);
        let unchanged = balances.clone();
        let result = transaction.preflight(|account| {
            balances
                .get(&account)
                .copied()
                .ok_or(WorldError::InvalidMoneyTransaction("missing test account"))
        });
        assert!(matches!(
            result,
            Err(WorldError::InvalidMoneyTransaction(_))
        ));
        assert_eq!(balances, unchanged);
    }

    #[test]
    fn issue_and_retire_have_explicit_non_transfer_identity() {
        let purpose = MoneyTransactionPurpose::GovernmentProgramAppropriation {
            program: ProgramId::new(1),
            country: CountryId::new(1),
        };
        let account = MoneyAccountId::GovernmentProgramEscrow(ProgramId::new(1));
        let amount = Money::from_minor_units(10);
        let issue = MoneyTransaction::issue(purpose, account, amount).expect("issue");
        assert_eq!(issue.kind(), MoneyTransactionKind::Issue);
        assert_eq!(issue.postings().len(), 1);
        assert_eq!(
            issue.postings()[0].direction(),
            MoneyPostingDirection::Increase
        );

        let retire = MoneyTransaction::retire(purpose, account, amount).expect("retire");
        assert_eq!(retire.kind(), MoneyTransactionKind::Retire);
        assert_eq!(retire.postings().len(), 1);
        assert_eq!(
            retire.postings()[0].direction(),
            MoneyPostingDirection::Decrease
        );
    }

    #[test]
    fn balanced_preflight_preserves_total_and_names_closing_balances() {
        let transaction = payroll(4).expect("balanced transaction");
        let balances = BTreeMap::from([
            (
                MoneyAccountId::FirmOperating(FirmId::new(1)),
                Money::from_minor_units(10),
            ),
            (
                MoneyAccountId::HouseholdLiquid(CohortId::new(1)),
                Money::from_minor_units(2),
            ),
        ]);
        let closing = transaction
            .preflight(|account| {
                balances
                    .get(&account)
                    .copied()
                    .ok_or(WorldError::InvalidMoneyTransaction("missing test account"))
            })
            .expect("preflight");
        assert_eq!(
            closing[&MoneyAccountId::FirmOperating(FirmId::new(1))],
            Money::from_minor_units(6)
        );
        assert_eq!(
            closing[&MoneyAccountId::HouseholdLiquid(CohortId::new(1))],
            Money::from_minor_units(6)
        );
        let opening_total: i64 = balances.values().map(|value| value.minor_units()).sum();
        let closing_total: i64 = closing.values().map(|value| value.minor_units()).sum();
        assert_eq!(opening_total, closing_total);
    }
}
