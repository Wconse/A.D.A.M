use std::collections::BTreeMap;

use crate::{BasisPoints, CountryId, DomainEvent, Money, RegionId, SimDate, World, WorldError};

const PRIORITY_SWITCH_MARGIN_BPS: u16 = 750;
const ACTIVE_PRESSURE_BPS: u16 = 2_500;
const CONFIDENCE_EFFECT_DIVISOR: i32 = 25;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum RegionalPolicyPriority {
    Employment,
    HouseholdSecurity,
    PublicServices,
    Stability,
}

impl RegionalPolicyPriority {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::Employment => 1,
            Self::HouseholdSecurity => 2,
            Self::PublicServices => 3,
            Self::Stability => 4,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum RegionalFiscalPosition {
    NetContributor,
    Balanced,
    NetBeneficiary,
}

impl RegionalFiscalPosition {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::NetContributor => 1,
            Self::Balanced => 2,
            Self::NetBeneficiary => 3,
        }
    }
}

/// Persistent regional policy concern and confidence in national government.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalInterest {
    priority: RegionalPolicyPriority,
    priority_pressure: BasisPoints,
    satisfaction: BasisPoints,
    years_persistent: u8,
    previous_priority_pressure: BasisPoints,
}

impl Default for RegionalInterest {
    fn default() -> Self {
        Self {
            priority: RegionalPolicyPriority::Stability,
            priority_pressure: BasisPoints::ZERO,
            satisfaction: BasisPoints::HALF,
            years_persistent: 0,
            previous_priority_pressure: BasisPoints::ZERO,
        }
    }
}

impl RegionalInterest {
    #[must_use]
    pub const fn priority(self) -> RegionalPolicyPriority {
        self.priority
    }

    #[must_use]
    pub const fn priority_pressure(self) -> BasisPoints {
        self.priority_pressure
    }

    #[must_use]
    pub const fn satisfaction(self) -> BasisPoints {
        self.satisfaction
    }

    #[must_use]
    pub const fn years_persistent(self) -> u8 {
        self.years_persistent
    }

    #[must_use]
    pub const fn previous_priority_pressure(self) -> BasisPoints {
        self.previous_priority_pressure
    }
}

/// Observable fiscal incidence for one region in one annual closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalPolicyOutcome {
    taxes_paid: Money,
    service_allocation: Money,
    net_transfer: Money,
    fiscal_position: RegionalFiscalPosition,
}

impl Default for RegionalPolicyOutcome {
    fn default() -> Self {
        Self {
            taxes_paid: Money::default(),
            service_allocation: Money::default(),
            net_transfer: Money::default(),
            fiscal_position: RegionalFiscalPosition::Balanced,
        }
    }
}

impl RegionalPolicyOutcome {
    #[must_use]
    pub const fn taxes_paid(self) -> Money {
        self.taxes_paid
    }

    #[must_use]
    pub const fn service_allocation(self) -> Money {
        self.service_allocation
    }

    #[must_use]
    pub const fn net_transfer(self) -> Money {
        self.net_transfer
    }

    #[must_use]
    pub const fn fiscal_position(self) -> RegionalFiscalPosition {
        self.fiscal_position
    }
}

impl World {
    pub(crate) fn update_annual_regional_interests(
        &mut self,
        close_date: SimDate,
    ) -> Result<(), WorldError> {
        let outcomes = self.derive_regional_policy_outcomes(close_date)?;
        let regions: Vec<_> = self.regions.keys().copied().collect();
        for region in regions {
            let pressure = self
                .regional_social_pressure
                .get(&region)
                .copied()
                .unwrap_or_default();
            let outcome = outcomes.get(&region).copied().unwrap_or_default();
            let previous = self
                .regional_interests
                .get(&region)
                .copied()
                .unwrap_or_default();
            let mut next = update_interest(previous, pressure, outcome.fiscal_position());
            let program_shift = self
                .regional_program_consequence_shift(region, close_date.year().saturating_sub(1));
            next.satisfaction = next.satisfaction.shifted(program_shift.clamp(-500, 500));
            self.regional_interests.insert(region, next);
            self.regional_policy_outcomes.insert(region, outcome);
            self.events.append(
                close_date,
                DomainEvent::RegionalPolicyOutcomeRecorded {
                    region,
                    taxes_paid: outcome.taxes_paid(),
                    service_allocation: outcome.service_allocation(),
                    net_transfer: outcome.net_transfer(),
                    fiscal_position: outcome.fiscal_position(),
                },
            );
            self.events.append(
                close_date,
                DomainEvent::RegionalInterestUpdated {
                    region,
                    previous_priority: previous.priority(),
                    priority: next.priority(),
                    priority_pressure: next.priority_pressure(),
                    satisfaction: next.satisfaction(),
                    years_persistent: next.years_persistent(),
                },
            );
        }
        self.apply_annual_program_country_consequences(
            close_date,
            close_date.year().saturating_sub(1),
        );
        Ok(())
    }

    fn regional_program_consequence_shift(&self, region: RegionId, execution_year: i32) -> i32 {
        self.government_programs
            .values()
            .filter(|program| program.last_execution_year() == Some(execution_year))
            .filter_map(|program| program.regional_memory().get(&region))
            .map(|memory| {
                if memory.outcome() == crate::ProgramRegionalOutcomeKind::Excluded {
                    -100
                } else {
                    let fulfillment = i32::from(memory.fulfillment().get());
                    fulfillment * 150 / 10_000 - (10_000 - fulfillment) * 300 / 10_000
                }
            })
            .fold(0_i32, i32::saturating_add)
    }

    #[allow(clippy::too_many_lines)]
    fn apply_annual_program_country_consequences(
        &mut self,
        close_date: SimDate,
        execution_year: i32,
    ) {
        let countries: Vec<_> = self.countries.keys().copied().collect();
        for country in countries {
            let mut population = 0_u128;
            let mut weighted = 0_i128;
            let mut minimum = i32::MAX;
            let mut maximum = i32::MIN;
            let mut affected = false;
            for region in self
                .regions
                .values()
                .filter(|region| region.country() == country)
            {
                let shift = self.regional_program_consequence_shift(region.id(), execution_year);
                let has_outcome = self.government_programs.values().any(|program| {
                    program.country() == country
                        && program.last_execution_year() == Some(execution_year)
                        && program.regional_memory().contains_key(&region.id())
                });
                if !has_outcome {
                    continue;
                }
                affected = true;
                let people = u128::from(region.population().people());
                population = population.saturating_add(people);
                weighted = weighted.saturating_add(
                    i128::from(shift) * i128::try_from(people).unwrap_or(i128::MAX),
                );
                minimum = minimum.min(shift);
                maximum = maximum.max(shift);
            }
            if !affected || population == 0 {
                continue;
            }
            let average = i32::try_from(weighted / i128::try_from(population).unwrap_or(i128::MAX))
                .unwrap_or(if weighted.is_negative() {
                    i32::MIN
                } else {
                    i32::MAX
                });
            let legitimacy_shift = (average / 2).clamp(-300, 150);
            let polarization = maximum.saturating_sub(minimum);
            let cohesion_shift = (average / 6 - polarization / 4).clamp(-300, 100);
            let indicators = self
                .countries
                .get_mut(&country)
                .expect("country exists")
                .indicators_mut();
            indicators.set_legitimacy(indicators.legitimacy().shifted(legitimacy_shift));
            indicators.set_elite_cohesion(indicators.elite_cohesion().shifted(cohesion_shift));
            let legitimacy = indicators.legitimacy();
            let elite_cohesion = indicators.elite_cohesion();
            self.events.append(
                close_date,
                DomainEvent::GovernmentProgramPoliticalConsequencesApplied {
                    country,
                    execution_year,
                    regional_average_shift: average,
                    polarization,
                    legitimacy_shift,
                    elite_cohesion_shift: cohesion_shift,
                    legitimacy,
                    elite_cohesion,
                },
            );
        }
    }

    fn derive_regional_policy_outcomes(
        &self,
        close_date: SimDate,
    ) -> Result<BTreeMap<RegionId, RegionalPolicyOutcome>, WorldError> {
        let mut taxes = BTreeMap::<RegionId, i128>::new();
        let mut services = BTreeMap::<RegionId, i128>::new();
        for envelope in self
            .events
            .events()
            .iter()
            .filter(|event| event.date() == close_date)
        {
            match envelope.event() {
                DomainEvent::FirmSalesTaxPaid { firm, paid, .. } => {
                    let Some(region) = self.firms.get(firm).map(crate::Firm::region) else {
                        continue;
                    };
                    let total = taxes.entry(region).or_default();
                    *total = total
                        .checked_add(i128::from(paid.minor_units()))
                        .ok_or(WorldError::ArithmeticOverflow("regional tax incidence"))?;
                }
                DomainEvent::RegionalPublicServicesUpdated {
                    region,
                    service_budget,
                    ..
                } => {
                    let total = services.entry(*region).or_default();
                    *total = total
                        .checked_add(i128::from(service_budget.minor_units()))
                        .ok_or(WorldError::ArithmeticOverflow(
                            "regional service allocation",
                        ))?;
                }
                _ => {}
            }
        }
        self.regions
            .keys()
            .copied()
            .map(|region| {
                let paid = *taxes.get(&region).unwrap_or(&0);
                let allocated = *services.get(&region).unwrap_or(&0);
                let net = allocated
                    .checked_sub(paid)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "regional net fiscal transfer",
                    ))?;
                let fiscal_position = match net.cmp(&0) {
                    std::cmp::Ordering::Less => RegionalFiscalPosition::NetContributor,
                    std::cmp::Ordering::Equal => RegionalFiscalPosition::Balanced,
                    std::cmp::Ordering::Greater => RegionalFiscalPosition::NetBeneficiary,
                };
                Ok((
                    region,
                    RegionalPolicyOutcome {
                        taxes_paid: money_from_i128(paid, "regional taxes")?,
                        service_allocation: money_from_i128(allocated, "regional services")?,
                        net_transfer: money_from_i128(net, "regional net transfer")?,
                        fiscal_position,
                    },
                ))
            })
            .collect()
    }

    #[must_use]
    pub fn regional_interests(&self) -> &BTreeMap<RegionId, RegionalInterest> {
        &self.regional_interests
    }

    #[must_use]
    pub fn regional_policy_outcomes(&self) -> &BTreeMap<RegionId, RegionalPolicyOutcome> {
        &self.regional_policy_outcomes
    }

    pub(crate) fn country_regional_confidence(&self, country: CountryId) -> BasisPoints {
        let mut population = 0_u128;
        let mut weighted = 0_u128;
        for region in self
            .regions
            .values()
            .filter(|region| region.country() == country)
        {
            let people = u128::from(region.population().people());
            let satisfaction = self
                .regional_interests
                .get(&region.id())
                .copied()
                .unwrap_or_default()
                .satisfaction()
                .get();
            population = population.saturating_add(people);
            weighted = weighted.saturating_add(u128::from(satisfaction).saturating_mul(people));
        }
        if population == 0 {
            return BasisPoints::HALF;
        }
        BasisPoints::new(u16::try_from((weighted / population).min(10_000)).unwrap_or(10_000))
            .expect("regional confidence is bounded")
    }
}

pub(crate) fn regional_confidence_effect(confidence: BasisPoints) -> i32 {
    (i32::from(confidence.get()) - 5_000) / CONFIDENCE_EFFECT_DIVISOR
}

fn update_interest(
    previous: RegionalInterest,
    pressure: crate::RegionalSocialPressure,
    fiscal_position: RegionalFiscalPosition,
) -> RegionalInterest {
    let (candidate, candidate_strength) = leading_priority(pressure);
    let current_strength = priority_pressure(previous.priority(), pressure);
    let switches = candidate != previous.priority()
        && (candidate_strength >= current_strength.saturating_add(PRIORITY_SWITCH_MARGIN_BPS)
            || (previous.priority() == RegionalPolicyPriority::Stability
                && pressure.combined().get() >= ACTIVE_PRESSURE_BPS));
    let priority = if switches {
        candidate
    } else {
        previous.priority()
    };
    let strength = priority_pressure(priority, pressure);
    let improvement = if switches {
        0
    } else {
        i32::from(previous.previous_priority_pressure().get()) - i32::from(strength)
    };
    let fiscal_signal = match fiscal_position {
        RegionalFiscalPosition::NetContributor => -25,
        RegionalFiscalPosition::Balanced => 0,
        RegionalFiscalPosition::NetBeneficiary => 25,
    };
    let satisfaction_shift = (improvement / 20 + fiscal_signal).clamp(-250, 250);
    RegionalInterest {
        priority,
        priority_pressure: BasisPoints::new(strength).expect("priority pressure is bounded"),
        satisfaction: previous.satisfaction().shifted(satisfaction_shift),
        years_persistent: if switches {
            1
        } else {
            previous.years_persistent().saturating_add(1)
        },
        previous_priority_pressure: BasisPoints::new(strength)
            .expect("previous pressure is bounded"),
    }
}

fn leading_priority(pressure: crate::RegionalSocialPressure) -> (RegionalPolicyPriority, u16) {
    let candidates = [
        (
            RegionalPolicyPriority::Employment,
            pressure.chronic_unemployment().get(),
        ),
        (
            RegionalPolicyPriority::HouseholdSecurity,
            pressure.livelihood_stress().get(),
        ),
        (
            RegionalPolicyPriority::PublicServices,
            pressure.public_service_shortfall().get(),
        ),
    ];
    let (priority, strength) = candidates
        .into_iter()
        .max_by_key(|(priority, strength)| (*strength, std::cmp::Reverse(*priority)))
        .expect("priority candidates are non-empty");
    if strength < ACTIVE_PRESSURE_BPS {
        (RegionalPolicyPriority::Stability, pressure.combined().get())
    } else {
        (priority, strength)
    }
}

fn priority_pressure(
    priority: RegionalPolicyPriority,
    pressure: crate::RegionalSocialPressure,
) -> u16 {
    match priority {
        RegionalPolicyPriority::Employment => pressure.chronic_unemployment().get(),
        RegionalPolicyPriority::HouseholdSecurity => pressure.livelihood_stress().get(),
        RegionalPolicyPriority::PublicServices => pressure.public_service_shortfall().get(),
        RegionalPolicyPriority::Stability => pressure.combined().get(),
    }
}

fn money_from_i128(value: i128, context: &'static str) -> Result<Money, WorldError> {
    i64::try_from(value)
        .map(Money::from_minor_units)
        .map_err(|_| WorldError::ArithmeticOverflow(context))
}

#[cfg(test)]
mod tests {
    use crate::{Country, Population, Region, WorldSeed};

    use super::*;

    fn pressure(
        unemployment: u16,
        livelihood: u16,
        services: u16,
        combined: u16,
    ) -> crate::RegionalSocialPressure {
        crate::RegionalSocialPressure::from_components_for_test(
            unemployment,
            livelihood,
            services,
            combined,
        )
    }

    #[test]
    fn priority_changes_only_for_materially_stronger_pressure() {
        let employment = update_interest(
            RegionalInterest::default(),
            pressure(6_000, 3_000, 4_000, 4_500),
            RegionalFiscalPosition::Balanced,
        );
        assert_eq!(employment.priority(), RegionalPolicyPriority::Employment);
        let retained = update_interest(
            employment,
            pressure(5_000, 5_500, 4_000, 4_800),
            RegionalFiscalPosition::Balanced,
        );
        assert_eq!(retained.priority(), RegionalPolicyPriority::Employment);
        let switched = update_interest(
            retained,
            pressure(4_000, 6_000, 4_000, 5_000),
            RegionalFiscalPosition::Balanced,
        );
        assert_eq!(
            switched.priority(),
            RegionalPolicyPriority::HouseholdSecurity
        );
    }

    #[test]
    fn fiscal_winners_and_losers_move_satisfaction_in_opposite_directions() {
        let base = RegionalInterest::default();
        let evidence = pressure(0, 0, 5_000, 833);
        let beneficiary = update_interest(base, evidence, RegionalFiscalPosition::NetBeneficiary);
        let contributor = update_interest(base, evidence, RegionalFiscalPosition::NetContributor);
        assert!(beneficiary.satisfaction().get() > BasisPoints::HALF.get());
        assert!(contributor.satisfaction().get() < BasisPoints::HALF.get());
    }

    #[test]
    fn regional_confidence_effect_is_bounded_and_centered() {
        assert_eq!(regional_confidence_effect(BasisPoints::ZERO), -200);
        assert_eq!(regional_confidence_effect(BasisPoints::HALF), 0);
        assert_eq!(regional_confidence_effect(BasisPoints::FULL), 200);
    }

    fn one_region_world(population: u64) -> World {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(11), date);
        world
            .register_country(Country::new(CountryId::new(1), "Union").expect("country"))
            .expect("country");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Harbor",
                    Population::new(population),
                    Money::from_minor_units(10_000),
                )
                .expect("region"),
            )
            .expect("region");
        world
    }

    #[test]
    fn service_allocation_records_a_real_regional_beneficiary() {
        let mut world = one_region_world(100);
        let close_date = SimDate::new(2026, 1).expect("date");
        world.events.append(
            close_date,
            DomainEvent::RegionalPublicServicesUpdated {
                region: RegionId::new(1),
                service_budget: Money::from_minor_units(100),
                funding_target: BasisPoints::HALF,
                healthcare: BasisPoints::HALF,
                infrastructure: BasisPoints::HALF,
                administration: BasisPoints::HALF,
            },
        );
        world
            .update_annual_regional_interests(close_date)
            .expect("regional interests");
        let outcome = world.regional_policy_outcomes()[&RegionId::new(1)];
        assert_eq!(outcome.net_transfer(), Money::from_minor_units(100));
        assert_eq!(
            outcome.fiscal_position(),
            RegionalFiscalPosition::NetBeneficiary
        );
        assert_eq!(
            world.regional_interests()[&RegionId::new(1)]
                .satisfaction()
                .get(),
            5_025
        );
    }

    #[test]
    fn country_confidence_is_population_weighted() {
        let mut world = one_region_world(100);
        world
            .register_region(
                Region::new(
                    RegionId::new(2),
                    CountryId::new(1),
                    "Interior",
                    Population::new(300),
                    Money::from_minor_units(10_000),
                )
                .expect("region"),
            )
            .expect("region");
        world.regional_interests.insert(
            RegionId::new(1),
            RegionalInterest {
                satisfaction: BasisPoints::new(2_000).expect("satisfaction"),
                ..RegionalInterest::default()
            },
        );
        world.regional_interests.insert(
            RegionId::new(2),
            RegionalInterest {
                satisfaction: BasisPoints::new(6_000).expect("satisfaction"),
                ..RegionalInterest::default()
            },
        );
        assert_eq!(
            world.country_regional_confidence(CountryId::new(1)).get(),
            5_000
        );
    }
}
