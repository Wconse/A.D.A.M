use std::collections::BTreeMap;

use crate::{DomainEvent, World};

/// One deterministic yearly narrative summary derived only from authoritative domain events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChronicleEntry {
    pub year: i32,
    pub importance: u16,
    pub text: String,
}

impl World {
    /// Builds a compact yearly chronicle without changing simulation state.
    #[must_use]
    pub fn chronicle(&self) -> Vec<ChronicleEntry> {
        let mut years = BTreeMap::<i32, YearSummary>::new();
        for envelope in self.events().events() {
            let row = years.entry(envelope.date().year()).or_default();
            row.observe(envelope.event());
        }
        years
            .into_iter()
            .filter_map(|(year, summary)| summary.finish(year))
            .collect()
    }
}

#[derive(Default)]
struct YearSummary {
    months: u64,
    production_milli: u128,
    traded_milli: u128,
    household_borrowing_minor: i128,
    relief_minor: i128,
    relief_debt_minor: i128,
    excess_deaths: u64,
    minimum_survival_bps: Option<u16>,
    rationing_actions: u64,
    rationed_requested_milli: u128,
    rationed_available_milli: u128,
    politics_changes: u64,
    debt_interest_minor: i128,
    indebted_countries: u64,
    measured_regions: u64,
    final_consumption_minor: i128,
    inventory_change_minor: i128,
    measured_output_minor: i128,
    completed: bool,
}

impl YearSummary {
    fn observe(&mut self, event: &DomainEvent) {
        match event {
            DomainEvent::MonthlyEconomicCycleCompleted { .. } => self.months += 1,
            DomainEvent::ProductionCompleted { quantity, .. } => {
                self.production_milli += u128::from(quantity.get());
            }
            DomainEvent::MarketTrade { quantity, .. } => {
                self.traded_milli += u128::from(quantity.get());
            }
            DomainEvent::HouseholdSurvivalBorrowed { amount, .. } => {
                self.household_borrowing_minor += i128::from(amount.minor_units());
            }
            DomainEvent::EmergencyReliefFunded { amount, .. } => {
                self.relief_minor += i128::from(amount.minor_units());
            }
            DomainEvent::EmergencyReliefDebtIssued { amount, .. } => {
                self.relief_debt_minor += i128::from(amount.minor_units());
            }
            DomainEvent::CohortHealthUpdated {
                survival_fulfillment,
                excess_deaths,
                ..
            } => {
                self.excess_deaths = self.excess_deaths.saturating_add(*excess_deaths);
                self.minimum_survival_bps = Some(
                    self.minimum_survival_bps
                        .map_or(survival_fulfillment.get(), |current| {
                            current.min(survival_fulfillment.get())
                        }),
                );
            }
            DomainEvent::SurvivalRationingApplied {
                requested,
                available,
                ..
            } => {
                self.rationing_actions += 1;
                self.rationed_requested_milli += u128::from(requested.get());
                self.rationed_available_milli += u128::from(available.get());
            }
            DomainEvent::RegionalOutputMeasured {
                final_consumption,
                inventory_change,
                annual_output,
                ..
            } => {
                self.measured_regions += 1;
                self.final_consumption_minor += i128::from(final_consumption.minor_units());
                self.inventory_change_minor += i128::from(inventory_change.minor_units());
                self.measured_output_minor += i128::from(annual_output.minor_units());
            }
            DomainEvent::PublicDebtInterestCharged { interest, .. } => {
                self.indebted_countries += 1;
                self.debt_interest_minor += i128::from(interest.minor_units());
            }
            DomainEvent::CountryPoliticsChanged { .. } => self.politics_changes += 1,
            DomainEvent::EconomicYearCompleted { .. } => self.completed = true,
            _ => {}
        }
    }

    fn finish(self, year: i32) -> Option<ChronicleEntry> {
        if self.months == 0 && !self.completed {
            return None;
        }
        let mut sentences = Vec::new();
        if let Some(fulfillment) = self.minimum_survival_bps {
            if fulfillment < 10_000 {
                sentences.push(format!(
                    "Survival consumption fell as low as {}.{:02}%.",
                    fulfillment / 100,
                    fulfillment % 100
                ));
            }
        }
        if self.excess_deaths > 0 {
            sentences.push(format!(
                "Insufficient survival consumption was followed by {} excess deaths.",
                self.excess_deaths
            ));
        }
        if self.household_borrowing_minor > 0 {
            sentences.push(format!(
                "Households borrowed {} minor currency units for survival purchases.",
                self.household_borrowing_minor
            ));
        }
        if self.rationing_actions > 0 {
            sentences.push(format!(
                "Officials rationed {} of {} requested milli-units across {} local shortages.",
                self.rationed_available_milli,
                self.rationed_requested_milli,
                self.rationing_actions
            ));
        }
        if self.relief_minor > 0 {
            sentences.push(format!(
                "Political offices transferred {} minor currency units in emergency relief, backed by {} of new public debt.",
                self.relief_minor, self.relief_debt_minor
            ));
        }
        if self.production_milli > 0 || self.traded_milli > 0 {
            sentences.push(format!(
                "Firms produced {} and households bought {} milli-units through local markets.",
                self.production_milli, self.traded_milli
            ));
        }
        if self.measured_regions > 0 {
            sentences.push(format!(
                "Annual accounts measured {} output from {} final consumption and {} inventory change across {} regions.",
                self.measured_output_minor,
                self.final_consumption_minor,
                self.inventory_change_minor,
                self.measured_regions
            ));
        }
        if self.debt_interest_minor > 0 {
            sentences.push(format!(
                "Public debt service charged {} minor currency units of interest across {} indebted countries.",
                self.debt_interest_minor, self.indebted_countries
            ));
        }
        if self.politics_changes > 0 {
            sentences.push(format!(
                "Political indicators changed in {} countries during annual closure.",
                self.politics_changes
            ));
        }
        if sentences.is_empty() {
            sentences.push(format!(
                "{} monthly cycles closed without a material event.",
                self.months
            ));
        }
        Some(ChronicleEntry {
            year,
            importance: self.importance(),
            text: sentences.join(" "),
        })
    }

    fn importance(&self) -> u16 {
        if self.excess_deaths > 0 {
            100
        } else if self.minimum_survival_bps.is_some_and(|value| value < 5_000) {
            90
        } else if self.rationing_actions > 0 {
            80
        } else if self.relief_debt_minor > 0 || self.relief_minor > 0 {
            70
        } else if self.household_borrowing_minor > 0 {
            60
        } else if self.production_milli > 0 || self.traded_milli > 0 || self.measured_regions > 0 {
            40
        } else {
            10
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasisPoints, CohortId, CountryId, Money, Population, SimDate, WorldSeed};

    use super::*;

    #[test]
    fn chronicle_connects_material_shortage_coping_relief_and_deaths() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::HouseholdSurvivalBorrowed {
                cohort: CohortId::new(1),
                amount: Money::from_minor_units(100),
                ending_wealth: Money::from_minor_units(100),
                ending_debt: Money::from_minor_units(100),
            },
        );
        world.events.append(
            date,
            DomainEvent::EmergencyReliefFunded {
                actor: crate::ActorId::new(1),
                country: CountryId::new(1),
                cohort: CohortId::new(1),
                amount: Money::from_minor_units(50),
            },
        );
        world.events.append(
            date,
            DomainEvent::CohortHealthUpdated {
                cohort: CohortId::new(1),
                survival_fulfillment: BasisPoints::new(2_500).expect("basis points"),
                functional_capacity: BasisPoints::new(9_000).expect("basis points"),
                excess_deaths: 3,
                survivors: Population::new(97),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert_eq!(chronicle[0].year, 2025);
        assert_eq!(chronicle[0].importance, 100);
        assert!(chronicle[0].text.contains("25.00%"));
        assert!(chronicle[0].text.contains("3 excess deaths"));
        assert!(chronicle[0].text.contains("borrowed 100"));
        assert!(chronicle[0].text.contains("transferred 50"));
    }

    #[test]
    fn chronicle_reports_public_debt_service() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::PublicDebtInterestCharged {
                country: CountryId::new(1),
                opening_debt: Money::from_minor_units(1_000_000),
                interest: Money::from_minor_units(30_000),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert!(chronicle[0].text.contains("interest"));
        assert!(chronicle[0].text.contains("30000"));
        assert!(chronicle[0].text.contains("1 indebted"));
    }
}
