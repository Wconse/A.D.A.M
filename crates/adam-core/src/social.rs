use crate::{BasisPoints, CohortId, NeedTier, World, WorldError};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SocialStress {
    health_risk: BasisPoints,
    unrest_pressure: BasisPoints,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SocialStressMemory {
    health_burden: BasisPoints,
    unrest_memory: BasisPoints,
}
impl SocialStressMemory {
    #[must_use]
    pub const fn health_burden(self) -> BasisPoints {
        self.health_burden
    }
    #[must_use]
    pub const fn unrest_memory(self) -> BasisPoints {
        self.unrest_memory
    }
}
impl SocialStress {
    #[must_use]
    pub const fn health_risk(self) -> BasisPoints {
        self.health_risk
    }
    #[must_use]
    pub const fn unrest_pressure(self) -> BasisPoints {
        self.unrest_pressure
    }
}
impl World {
    /// Derives cohort health and unrest pressure from the settled monthly deprivation ledgers.
    /// # Errors
    /// Returns an error on bounded-value conversion failure.
    pub fn derive_monthly_social_stress(&mut self) -> Result<(), WorldError> {
        let mut stress = BTreeMap::new();
        let cohort_ids: Vec<_> = self.cohorts.keys().copied().collect();
        for cohort in cohort_ids {
            let mut survival_met = 0_u128;
            let mut survival_unmet = 0_u128;
            for ((id, _good, tier), q) in self.monthly_consumption() {
                if id == &cohort && *tier == NeedTier::Survival {
                    survival_met += u128::from(q.get());
                }
            }
            for ((id, _good, tier), q) in self.unmet_demand() {
                if id == &cohort && *tier == NeedTier::Survival {
                    survival_unmet += u128::from(q.get());
                }
            }
            let total = survival_met + survival_unmet;
            let health = if total == 0 {
                0
            } else {
                u16::try_from(survival_unmet * 10_000 / total)
                    .map_err(|_| WorldError::ArithmeticOverflow("health stress"))?
            };
            let unrest = self.deprivation_pressure().get(&cohort).copied().unwrap_or(
                BasisPoints::new(0).map_err(|_| WorldError::ArithmeticOverflow("zero pressure"))?,
            );
            let row = SocialStress {
                health_risk: BasisPoints::new(health)
                    .map_err(|_| WorldError::ArithmeticOverflow("health bounds"))?,
                unrest_pressure: unrest,
            };
            stress.insert(cohort, row);
        }
        self.social_stress = stress;
        for (cohort, row) in &self.social_stress {
            self.events.append(
                self.date,
                crate::DomainEvent::SocialStressUpdated {
                    cohort: *cohort,
                    health_risk: row.health_risk(),
                    unrest_pressure: row.unrest_pressure(),
                },
            );
        }
        Ok(())
    }
    /// Accumulates persistent stress memory with 75% prior state and 25% current pressure.
    /// # Errors
    /// Returns an error on bounded conversion failure.
    pub fn accumulate_monthly_social_stress(&mut self) -> Result<(), WorldError> {
        let mut memory = BTreeMap::new();
        for (cohort, current) in &self.social_stress {
            let previous = self.social_stress_memory.get(cohort).copied();
            let health = (u32::from(previous.map_or(0, |v| v.health_burden().get())) * 3
                + u32::from(current.health_risk().get()))
                / 4;
            let unrest = (u32::from(previous.map_or(0, |v| v.unrest_memory().get())) * 3
                + u32::from(current.unrest_pressure().get()))
                / 4;
            memory.insert(
                *cohort,
                SocialStressMemory {
                    health_burden: BasisPoints::new(
                        u16::try_from(health)
                            .map_err(|_| WorldError::ArithmeticOverflow("health memory"))?,
                    )
                    .map_err(|_| WorldError::ArithmeticOverflow("health memory bounds"))?,
                    unrest_memory: BasisPoints::new(
                        u16::try_from(unrest)
                            .map_err(|_| WorldError::ArithmeticOverflow("unrest memory"))?,
                    )
                    .map_err(|_| WorldError::ArithmeticOverflow("unrest memory bounds"))?,
                },
            );
        }
        self.social_stress_memory = memory;
        Ok(())
    }
    #[must_use]
    pub fn social_stress_memory(&self) -> &BTreeMap<CohortId, SocialStressMemory> {
        &self.social_stress_memory
    }
    #[must_use]
    pub fn social_stress(&self) -> &BTreeMap<CohortId, SocialStress> {
        &self.social_stress
    }
}
