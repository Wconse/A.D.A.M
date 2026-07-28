use crate::{CohortId, FirmId, Money, World, WorldError};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmploymentAgreement {
    firm: FirmId,
    cohort: CohortId,
    workers: u64,
    monthly_wage_per_worker: Money,
    arrears: Money,
    active: bool,
}
impl EmploymentAgreement {
    /// Creates a firm-to-cohort wage agreement.
    /// # Errors
    /// Returns an error for zero workers or non-positive wage.
    pub fn new(
        firm: FirmId,
        cohort: CohortId,
        workers: u64,
        wage: Money,
    ) -> Result<Self, WorldError> {
        if workers == 0 || wage.minor_units() <= 0 {
            return Err(WorldError::InvalidEmployment(
                "workers and wage must be positive",
            ));
        }
        Ok(Self {
            firm,
            cohort,
            workers,
            monthly_wage_per_worker: wage,
            arrears: Money::default(),
            active: true,
        })
    }
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn cohort(&self) -> CohortId {
        self.cohort
    }
    #[must_use]
    pub const fn workers(&self) -> u64 {
        self.workers
    }
    #[must_use]
    pub const fn wage(&self) -> Money {
        self.monthly_wage_per_worker
    }
    #[must_use]
    pub const fn arrears(&self) -> Money {
        self.arrears
    }
    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }
    pub(crate) fn set_workers(&mut self, workers: u64) {
        self.workers = workers;
        self.active = workers > 0;
    }

    pub(crate) fn settle_arrears(&mut self) -> Money {
        let settled = self.arrears;
        self.arrears = Money::default();
        settled
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PayrollRecord {
    pub firm: FirmId,
    pub cohort: CohortId,
    pub owed: Money,
    pub paid: Money,
    pub arrears: Money,
}
impl World {
    /// Registers a local employment agreement within firm and cohort worker limits.
    /// # Errors
    /// Returns an error for unknown references, region mismatch, duplicate pair, or over-allocation.
    pub fn register_employment_agreement(
        &mut self,
        agreement: EmploymentAgreement,
    ) -> Result<(), WorldError> {
        let firm = self
            .firms()
            .get(&agreement.firm())
            .ok_or(WorldError::UnknownFirm(agreement.firm()))?;
        let cohort = self
            .cohorts
            .get(&agreement.cohort())
            .ok_or(WorldError::UnknownCohort(agreement.cohort()))?;
        if firm.region() != cohort.region() {
            return Err(WorldError::InvalidEmployment(
                "firm and cohort must share a region",
            ));
        }
        let key = (agreement.firm(), agreement.cohort());
        if self.employment_agreements.contains_key(&key) {
            return Err(WorldError::InvalidEmployment(
                "duplicate employment agreement",
            ));
        }
        let firm_allocated: u64 = self
            .employment_agreements
            .values()
            .filter(|a| a.firm() == agreement.firm())
            .map(EmploymentAgreement::workers)
            .sum();
        let cohort_allocated: u64 = self
            .employment_agreements
            .values()
            .filter(|a| a.cohort() == agreement.cohort())
            .map(EmploymentAgreement::workers)
            .sum();
        if firm_allocated + agreement.workers() > firm.workers()
            || cohort_allocated + agreement.workers() > cohort.people().people()
        {
            return Err(WorldError::InvalidEmployment(
                "employment exceeds workers or cohort population",
            ));
        }
        self.employment_agreements.insert(key, agreement);
        Ok(())
    }
    /// Changes staffed workers in an existing agreement; zero workers terminates active work while preserving arrears.
    /// # Errors
    /// Returns an error for unknown references or worker over-allocation.
    pub fn change_employment_workers(
        &mut self,
        firm: FirmId,
        cohort: CohortId,
        workers: u64,
    ) -> Result<(), WorldError> {
        let key = (firm, cohort);
        if workers > 0 && self.is_firm_insolvent(firm) {
            return Err(WorldError::InvalidFirmReorganization(
                "insolvent employment can resume only through an approved reorganization",
            ));
        }
        let definition = self
            .firms()
            .get(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?;
        let population = self
            .cohorts
            .get(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?
            .people()
            .people();
        let firm_other: u64 = self
            .employment_agreements
            .iter()
            .filter(|(key, row)| key.0 == firm && key.1 != cohort && row.active())
            .map(|(_, row)| row.workers())
            .sum();
        let cohort_other: u64 = self
            .employment_agreements
            .iter()
            .filter(|(key, row)| key.1 == cohort && key.0 != firm && row.active())
            .map(|(_, row)| row.workers())
            .sum();
        if firm_other + workers > definition.workers() || cohort_other + workers > population {
            return Err(WorldError::InvalidEmployment(
                "worker change exceeds firm or cohort limit",
            ));
        }
        let agreement =
            self.employment_agreements
                .get_mut(&key)
                .ok_or(WorldError::InvalidEmployment(
                    "employment agreement is missing",
                ))?;
        let previous = agreement.workers();
        agreement.set_workers(workers);
        self.events.append(
            self.date,
            crate::DomainEvent::EmploymentChanged {
                firm,
                cohort,
                previous_workers: previous,
                current_workers: workers,
            },
        );
        Ok(())
    }
    /// Pays wages from firm cash to cohort wealth; unpaid obligations become arrears.
    /// # Errors
    /// Returns an error on arithmetic overflow.
    pub fn execute_monthly_payroll(&mut self) -> Result<Vec<PayrollRecord>, WorldError> {
        if self.last_payroll_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "payroll",
                date: self.date,
            });
        }
        let mut firms = self.firms.clone();
        let mut cohorts = self.cohorts.clone();
        let mut agreements = self.employment_agreements.clone();
        let mut records = Vec::new();
        for agreement in agreements
            .values_mut()
            .filter(|agreement| agreement.active || agreement.arrears().minor_units() > 0)
        {
            let current =
                i128::from(agreement.wage().minor_units()) * i128::from(agreement.workers());
            let current = i64::try_from(current)
                .map_err(|_| WorldError::ArithmeticOverflow("payroll owed"))?;
            let owed = current
                .checked_add(agreement.arrears().minor_units())
                .ok_or(WorldError::ArithmeticOverflow("payroll arrears"))?;
            let firm = firms
                .get_mut(&agreement.firm())
                .ok_or(WorldError::UnknownFirm(agreement.firm()))?;
            let paid = firm.cash().minor_units().min(owed);
            firm.debit_cash(Money::from_minor_units(paid))?;
            cohorts
                .get_mut(&agreement.cohort())
                .ok_or(WorldError::UnknownCohort(agreement.cohort()))?
                .credit_wealth(Money::from_minor_units(paid))?;
            agreement.arrears = Money::from_minor_units(owed - paid);
            records.push(PayrollRecord {
                firm: agreement.firm(),
                cohort: agreement.cohort(),
                owed: Money::from_minor_units(owed),
                paid: Money::from_minor_units(paid),
                arrears: agreement.arrears(),
            });
        }
        self.firms = firms;
        self.cohorts = cohorts;
        self.employment_agreements = agreements;
        for row in &records {
            self.record_firm_payroll(row.firm, row.owed, row.paid, row.arrears)?;
        }
        self.last_payroll_date = Some(self.date);
        for row in &records {
            self.events.append(
                self.date,
                crate::DomainEvent::PayrollSettled {
                    firm: row.firm,
                    cohort: row.cohort,
                    owed: row.owed,
                    paid: row.paid,
                    arrears: row.arrears,
                },
            );
        }
        Ok(records)
    }
    #[must_use]
    pub fn employment_agreements(&self) -> &BTreeMap<(FirmId, CohortId), EmploymentAgreement> {
        &self.employment_agreements
    }
}
