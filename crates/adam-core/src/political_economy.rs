use std::collections::BTreeMap;

use crate::{BasisPoints, CountryId, EmploymentStatus, RegionId, World, WorldError};

const UNEMPLOYMENT_PRESSURE_PER_MONTH: u32 = 250;
const DEBT_PRESSURE_PER_MONTH: u32 = 125;
const EXPERIENCE_MONTH_CAP: u32 = 24;
const LEGITIMACY_NEUTRAL_PRESSURE_BPS: i32 = 3_000;
const LEGITIMACY_PRESSURE_DIVISOR: i32 = 20;

/// Population-weighted material and institutional pressure observed in one region.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalSocialPressure {
    chronic_unemployment: BasisPoints,
    livelihood_stress: BasisPoints,
    public_service_shortfall: BasisPoints,
    combined: BasisPoints,
}

impl Default for RegionalSocialPressure {
    fn default() -> Self {
        let zero = BasisPoints::new(0).expect("zero pressure is bounded");
        Self {
            chronic_unemployment: zero,
            livelihood_stress: zero,
            public_service_shortfall: zero,
            combined: zero,
        }
    }
}

impl RegionalSocialPressure {
    #[must_use]
    pub const fn chronic_unemployment(self) -> BasisPoints {
        self.chronic_unemployment
    }

    #[must_use]
    pub const fn livelihood_stress(self) -> BasisPoints {
        self.livelihood_stress
    }

    #[must_use]
    pub const fn public_service_shortfall(self) -> BasisPoints {
        self.public_service_shortfall
    }

    #[must_use]
    pub const fn combined(self) -> BasisPoints {
        self.combined
    }

    #[cfg(test)]
    pub(crate) fn from_components_for_test(
        unemployment: u16,
        livelihood: u16,
        services: u16,
        combined: u16,
    ) -> Self {
        Self {
            chronic_unemployment: BasisPoints::new(unemployment).expect("test pressure"),
            livelihood_stress: BasisPoints::new(livelihood).expect("test pressure"),
            public_service_shortfall: BasisPoints::new(services).expect("test pressure"),
            combined: BasisPoints::new(combined).expect("test pressure"),
        }
    }
}

impl World {
    pub(crate) fn plan_annual_regional_social_pressure(
        &self,
    ) -> Result<BTreeMap<RegionId, RegionalSocialPressure>, WorldError> {
        self.regions
            .keys()
            .copied()
            .map(|region| Ok((region, self.derive_regional_social_pressure(region)?)))
            .collect()
    }

    fn derive_regional_social_pressure(
        &self,
        region: RegionId,
    ) -> Result<RegionalSocialPressure, WorldError> {
        let mut people = 0_u128;
        let mut unemployment_weighted = 0_u128;
        let mut livelihood_weighted = 0_u128;
        for cohort in self
            .cohorts
            .values()
            .filter(|cohort| cohort.region() == region)
        {
            let weight = u128::from(cohort.people().people());
            people = people.saturating_add(weight);
            let experience = self
                .cohort_experience
                .get(&cohort.id())
                .copied()
                .unwrap_or_default();
            let unemployment = if cohort.employment() == EmploymentStatus::Unemployed {
                experience
                    .unemployment_months()
                    .min(EXPERIENCE_MONTH_CAP)
                    .saturating_mul(UNEMPLOYMENT_PRESSURE_PER_MONTH)
                    .min(10_000)
            } else {
                0
            };
            let unrest = u32::from(
                self.social_stress_memory
                    .get(&cohort.id())
                    .map_or(0, |memory| memory.unrest_memory().get()),
            );
            let debt = experience
                .debt_distress_months()
                .min(EXPERIENCE_MONTH_CAP)
                .saturating_mul(DEBT_PRESSURE_PER_MONTH)
                .min(10_000);
            let livelihood = (unrest.saturating_mul(3).saturating_add(debt)) / 4;
            unemployment_weighted = unemployment_weighted
                .saturating_add(u128::from(unemployment).saturating_mul(weight));
            livelihood_weighted =
                livelihood_weighted.saturating_add(u128::from(livelihood).saturating_mul(weight));
        }
        let unemployment = weighted_average(unemployment_weighted, people)?;
        let livelihood = weighted_average(livelihood_weighted, people)?;
        let services = self
            .regional_public_services
            .get(&region)
            .copied()
            .unwrap_or_default();
        let delivered = (u32::from(services.healthcare().get())
            + u32::from(services.infrastructure().get())
            + u32::from(services.administration().get()))
            / 3;
        let service_shortfall = u16::try_from(10_000_u32.saturating_sub(delivered))
            .map_err(|_| WorldError::ArithmeticOverflow("regional service shortfall"))?;
        let combined = (u32::from(unemployment) * 2
            + u32::from(livelihood) * 3
            + u32::from(service_shortfall))
            / 6;
        Ok(RegionalSocialPressure {
            chronic_unemployment: BasisPoints::new(unemployment)
                .map_err(|_| WorldError::ArithmeticOverflow("unemployment pressure bounds"))?,
            livelihood_stress: BasisPoints::new(livelihood)
                .map_err(|_| WorldError::ArithmeticOverflow("livelihood pressure bounds"))?,
            public_service_shortfall: BasisPoints::new(service_shortfall)
                .map_err(|_| WorldError::ArithmeticOverflow("service pressure bounds"))?,
            combined: BasisPoints::new(u16::try_from(combined).unwrap_or(10_000))
                .map_err(|_| WorldError::ArithmeticOverflow("combined pressure bounds"))?,
        })
    }

    #[must_use]
    pub fn regional_social_pressure(&self) -> &BTreeMap<RegionId, RegionalSocialPressure> {
        &self.regional_social_pressure
    }

    pub(crate) fn country_social_pressure(
        &self,
        country: CountryId,
        regional: &BTreeMap<RegionId, RegionalSocialPressure>,
    ) -> BasisPoints {
        let mut population = 0_u128;
        let mut weighted = 0_u128;
        for region in self
            .regions
            .values()
            .filter(|region| region.country() == country)
        {
            let people = u128::from(region.population().people());
            population = population.saturating_add(people);
            weighted = weighted.saturating_add(
                u128::from(
                    regional
                        .get(&region.id())
                        .copied()
                        .unwrap_or_default()
                        .combined()
                        .get(),
                )
                .saturating_mul(people),
            );
        }
        BasisPoints::new(weighted_average(weighted, population).unwrap_or(0))
            .expect("weighted pressure is bounded")
    }
}

pub(crate) fn legitimacy_effect(pressure: BasisPoints) -> i32 {
    (LEGITIMACY_NEUTRAL_PRESSURE_BPS - i32::from(pressure.get())) / LEGITIMACY_PRESSURE_DIVISOR
}

fn weighted_average(weighted: u128, population: u128) -> Result<u16, WorldError> {
    if population == 0 {
        return Ok(0);
    }
    u16::try_from((weighted / population).min(10_000))
        .map_err(|_| WorldError::ArithmeticOverflow("population-weighted social pressure"))
}

#[cfg(test)]
mod tests {
    use crate::{Country, Money, Population, Region, SimDate, WorldSeed};

    use super::*;

    #[test]
    fn legitimacy_rewards_stability_and_penalizes_severe_pressure() {
        assert_eq!(
            legitimacy_effect(BasisPoints::new(0).expect("pressure")),
            150
        );
        assert_eq!(
            legitimacy_effect(BasisPoints::new(3_000).expect("pressure")),
            0
        );
        assert_eq!(legitimacy_effect(BasisPoints::FULL), -350);
    }

    fn world_with_two_regions() -> World {
        let mut world = World::new(WorldSeed::new(9), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "Republic").expect("country"))
            .expect("country");
        for (id, people) in [(1, 100), (2, 300)] {
            world
                .register_region(
                    Region::new(
                        RegionId::new(id),
                        CountryId::new(1),
                        format!("Region {id}"),
                        Population::new(people),
                        Money::from_minor_units(10_000),
                    )
                    .expect("region"),
                )
                .expect("region");
        }
        world
    }

    #[test]
    fn country_pressure_is_population_weighted_not_region_averaged() {
        let world = world_with_two_regions();
        let value = |combined| RegionalSocialPressure {
            chronic_unemployment: BasisPoints::new(0).expect("pressure"),
            livelihood_stress: BasisPoints::new(0).expect("pressure"),
            public_service_shortfall: BasisPoints::new(0).expect("pressure"),
            combined: BasisPoints::new(combined).expect("pressure"),
        };
        let regional = BTreeMap::from([
            (RegionId::new(1), value(1_000)),
            (RegionId::new(2), value(5_000)),
        ]);
        assert_eq!(
            world
                .country_social_pressure(CountryId::new(1), &regional)
                .get(),
            4_000
        );
    }

    #[test]
    fn service_shortfall_is_visible_without_invented_household_stress() {
        let world = world_with_two_regions();
        let pressure = world
            .plan_annual_regional_social_pressure()
            .expect("pressure");
        let first = pressure[&RegionId::new(1)];
        assert_eq!(first.chronic_unemployment().get(), 0);
        assert_eq!(first.livelihood_stress().get(), 0);
        assert_eq!(first.public_service_shortfall().get(), 5_000);
        assert_eq!(first.combined().get(), 833);
    }
}
