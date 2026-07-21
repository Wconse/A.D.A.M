use crate::{DomainEvent, FirmId, Money, ProjectId, RegionId, World, WorldError};
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
        self.committed_investments
            .insert(project.firm(), Money::from_minor_units(committed - budget));
        project.status = InvestmentStatus::Building;
        self.events.append(
            self.date,
            DomainEvent::InvestmentProjectLaunched {
                project: project.id(),
                firm: project.firm(),
                budget: project.budget(),
            },
        );
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
