use crate::{
    ActorId, BasisPoints, CohortId, CountryId, DomainEvent, FirmId, GoodId, Money, NeedTier,
    QuantityMilli, RegionId, World, WorldError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GovernmentReserveProcurement {
    pub actor: ActorId,
    pub country: CountryId,
    pub region: RegionId,
    pub seller: FirmId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub cost: Money,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GovernmentReserveDistribution {
    pub country: CountryId,
    pub region: RegionId,
    pub cohort: CohortId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GovernmentReserveMaintenance {
    pub country: CountryId,
    pub region: RegionId,
    pub good: GoodId,
    pub opening_stock: QuantityMilli,
    pub reference_value: Money,
    pub assessed_cost: Money,
    pub paid_cost: Money,
    pub baseline_spoilage: QuantityMilli,
    pub neglect_spoilage: QuantityMilli,
    pub closing_stock: QuantityMilli,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReservePriorityRevisionReason {
    None,
    UncoveredGap,
    ImportReliance,
    IdleSpoilage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GovernmentReservePolicyReview {
    pub country: CountryId,
    pub observed_shortage: QuantityMilli,
    pub opening_stock: QuantityMilli,
    pub remaining_gap: QuantityMilli,
    pub baseline_spoilage: QuantityMilli,
    pub neglect_spoilage: QuantityMilli,
    pub preparedness_months: u8,
    pub budget_gap_months: u8,
    pub upkeep_stress_months: u8,
    pub waste_months: u8,
    pub previous_coverage_months: u8,
    pub new_coverage_months: u8,
    pub previous_monthly_budget: crate::BasisPoints,
    pub new_monthly_budget: crate::BasisPoints,
}

#[derive(Clone, Copy, Debug, Default)]
struct ReservePolicyEvidence {
    observed_shortage: u64,
    opening_stock: u64,
    remaining_gap: u64,
    baseline_spoilage: u64,
    neglect_spoilage: u64,
    reserve_distributed: u64,
    preparedness_pressure: bool,
    budget_pressure: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct ReservePriorityEvidence {
    remaining_gap: u64,
    baseline_spoilage: u64,
    distributed: u64,
    local_quantity: u64,
    imported_quantity: u64,
    supply_limited: bool,
}

#[derive(Clone, Copy)]
struct ReserveMaintenancePlan {
    country: CountryId,
    region: RegionId,
    good: GoodId,
    opening_stock: QuantityMilli,
    reference_value: Money,
    assessed_cost: Money,
    baseline_spoilage: QuantityMilli,
}

const MAX_UNFUNDED_MAINTENANCE_SPOILAGE_BPS: u64 = 2_500;
const HIGH_HOUSEHOLD_IMPORT_RELIANCE_BPS: u16 = 6_000;

impl World {
    /// Sets the share of the national reserve doctrine assigned to one regional good.
    ///
    /// A full priority applies the complete coverage target, a partial priority scales it down,
    /// and zero explicitly excludes the regional good from automatic stockpiling. Scarce national
    /// procurement budgets are reviewed from highest priority to lowest, then by stable ids.
    /// # Errors
    /// Rejects unknown references, cross-country regions, and unauthorized actors atomically.
    pub fn set_government_reserve_priority(
        &mut self,
        actor: ActorId,
        country: CountryId,
        region: RegionId,
        good: GoodId,
        priority: BasisPoints,
    ) -> Result<(), WorldError> {
        let actual_country = self
            .regions
            .get(&region)
            .ok_or(WorldError::UnknownRegion(region))?
            .country();
        if actual_country != country {
            return Err(WorldError::InvalidEmergencyRelief(
                "reserve priority region must belong to the selected country",
            ));
        }
        if !self.goods.contains_key(&good) {
            return Err(WorldError::UnknownGood(good));
        }
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        self.government_reserve_priorities
            .insert((country, region, good), priority);
        self.events.append(
            self.date,
            DomainEvent::GovernmentReservePrioritySet {
                actor,
                country,
                region,
                good,
                priority,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn government_reserve_priorities(
        &self,
    ) -> &std::collections::BTreeMap<(CountryId, RegionId, GoodId), BasisPoints> {
        &self.government_reserve_priorities
    }

    fn government_reserve_priority(
        &self,
        country: CountryId,
        region: RegionId,
        good: GoodId,
    ) -> BasisPoints {
        self.government_reserve_priorities
            .get(&(country, region, good))
            .copied()
            .unwrap_or(BasisPoints::FULL)
    }

    /// Buys physical inventory from a domestic firm into a regional public reserve.
    /// # Errors
    /// Rejects invalid quantities/prices, foreign or unauthorized purchases, and insufficient stock or treasury atomically.
    pub fn procure_government_reserve(
        &mut self,
        actor: ActorId,
        seller: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
        unit_price: Money,
    ) -> Result<GovernmentReserveProcurement, WorldError> {
        if quantity.get() == 0 || unit_price.minor_units() <= 0 {
            return Err(WorldError::InvalidEmergencyRelief(
                "reserve procurement requires positive quantity and price",
            ));
        }
        if !self.goods.contains_key(&good) {
            return Err(WorldError::UnknownGood(good));
        }
        let firm = self
            .firms
            .get(&seller)
            .ok_or(WorldError::UnknownFirm(seller))?;
        let region = firm.region();
        let country = self
            .regions
            .get(&region)
            .ok_or(WorldError::UnknownRegion(region))?
            .country();
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        let raw_cost = i128::from(quantity.get())
            .checked_mul(i128::from(unit_price.minor_units()))
            .ok_or(WorldError::ArithmeticOverflow("government reserve cost"))?
            / i128::from(QuantityMilli::SCALE);
        let cost_minor = i64::try_from(raw_cost)
            .map_err(|_| WorldError::ArithmeticOverflow("government reserve cost"))?;
        if cost_minor <= 0 {
            return Err(WorldError::InvalidEmergencyRelief(
                "reserve procurement cost must settle above zero",
            ));
        }
        let cost = Money::from_minor_units(cost_minor);
        let treasury = self.countries[&country].indicators().treasury();
        if treasury.minor_units() < cost_minor {
            return Err(WorldError::InsufficientTreasury(country));
        }

        let mut firms = self.firms.clone();
        firms
            .get_mut(&seller)
            .ok_or(WorldError::UnknownFirm(seller))?
            .debit_inventory(good, quantity)?;
        firms
            .get_mut(&seller)
            .ok_or(WorldError::UnknownFirm(seller))?
            .apply_cash_delta(cost)?;
        let mut countries = self.countries.clone();
        countries
            .get_mut(&country)
            .ok_or(WorldError::UnknownCountry(country))?
            .indicators_mut()
            .set_treasury(Money::from_minor_units(treasury.minor_units() - cost_minor));
        let mut reserves = self.government_reserves.clone();
        let current = reserves.get(&(region, good)).copied().unwrap_or_default();
        reserves.insert(
            (region, good),
            QuantityMilli::new(
                current
                    .get()
                    .checked_add(quantity.get())
                    .ok_or(WorldError::ArithmeticOverflow("government reserve stock"))?,
            ),
        );

        self.firms = firms;
        self.countries = countries;
        self.government_reserves = reserves;
        self.events.append(
            self.date,
            DomainEvent::GovernmentReserveProcured {
                actor,
                country,
                region,
                seller,
                good,
                quantity,
                cost,
            },
        );
        Ok(GovernmentReserveProcurement {
            actor,
            country,
            region,
            seller,
            good,
            quantity,
            cost,
        })
    }

    /// Uses observed residual survival demand to buy a bounded local reserve buffer.
    /// # Errors
    /// Rejects duplicate monthly execution and leaves the world unchanged if any accepted purchase fails.
    pub fn execute_observed_government_reserve_procurement(
        &mut self,
    ) -> Result<Vec<GovernmentReserveProcurement>, WorldError> {
        let mut next = self.clone();
        let purchases = next.execute_observed_government_reserve_procurement_in_place()?;
        *self = next;
        Ok(purchases)
    }

    pub(crate) fn execute_observed_government_reserve_procurement_in_place(
        &mut self,
    ) -> Result<Vec<GovernmentReserveProcurement>, WorldError> {
        if self.last_government_reserve_procurement_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed government reserve procurement",
                date: self.date,
            });
        }
        let requirements = self.observed_reserve_requirements()?;
        let mut ordered_requirements: Vec<_> = requirements.into_iter().collect();
        ordered_requirements.sort_by(
            |((left_region, left_good), _), ((right_region, right_good), _)| {
                let left_country = self.regions[left_region].country();
                let right_country = self.regions[right_region].country();
                self.government_reserve_priority(right_country, *right_region, *right_good)
                    .cmp(&self.government_reserve_priority(left_country, *left_region, *left_good))
                    .then_with(|| left_country.cmp(&right_country))
                    .then_with(|| left_region.cmp(right_region))
                    .then_with(|| left_good.cmp(right_good))
            },
        );
        let mut purchases = Vec::new();
        let mut country_budgets = std::collections::BTreeMap::new();
        for ((region, good), observed_need) in ordered_requirements {
            purchases.extend(self.procure_reserve_requirement(
                region,
                good,
                observed_need,
                &mut country_budgets,
            )?);
        }
        self.last_government_reserve_procurement_date = Some(self.date);
        let total_quantity = purchases.iter().try_fold(0_u64, |sum, purchase| {
            sum.checked_add(purchase.quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("reserve procurement total"))
        })?;
        let total_spending = purchases.iter().try_fold(0_i64, |sum, purchase| {
            sum.checked_add(purchase.cost.minor_units())
                .ok_or(WorldError::ArithmeticOverflow(
                    "reserve procurement spending",
                ))
        })?;
        self.events.append(
            self.date,
            DomainEvent::ObservedGovernmentReserveProcurementCompleted {
                purchases: u64::try_from(purchases.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("reserve procurement purchases"))?,
                quantity: QuantityMilli::new(total_quantity),
                spending: Money::from_minor_units(total_spending),
            },
        );
        Ok(purchases)
    }

    fn observed_reserve_requirements(
        &self,
    ) -> Result<std::collections::BTreeMap<(RegionId, GoodId), u64>, WorldError> {
        let mut required = std::collections::BTreeMap::new();
        for ((cohort, good, tier), quantity) in &self.unmet_demand {
            if *tier != NeedTier::Survival || quantity.get() == 0 {
                continue;
            }
            let region = self
                .cohorts
                .get(cohort)
                .ok_or(WorldError::UnknownCohort(*cohort))?
                .region();
            let total = required.entry((region, *good)).or_insert(0_u64);
            *total = total
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("reserve procurement need"))?;
        }
        Ok(required)
    }

    #[allow(clippy::too_many_lines)]
    fn procure_reserve_requirement(
        &mut self,
        region: RegionId,
        good: GoodId,
        observed_need: u64,
        country_budgets: &mut std::collections::BTreeMap<CountryId, i64>,
    ) -> Result<Vec<GovernmentReserveProcurement>, WorldError> {
        let country = self
            .regions
            .get(&region)
            .ok_or(WorldError::UnknownRegion(region))?
            .country();
        let policy = self
            .government_emergency_policies
            .get(&country)
            .copied()
            .unwrap_or_default();
        let Some(actor) = self.emergency_relief_actor(country) else {
            return Ok(Vec::new());
        };
        if policy.physical_shortage_strategy() != crate::PhysicalShortageStrategy::ReserveRelease {
            return Ok(Vec::new());
        }
        let stocked = self
            .government_reserves
            .get(&(region, good))
            .copied()
            .unwrap_or_default()
            .get();
        let priority = self.government_reserve_priority(country, region, good);
        let full_target = observed_need
            .checked_mul(u64::from(policy.reserve_coverage_months()))
            .ok_or(WorldError::ArithmeticOverflow("reserve coverage target"))?;
        let target = scale_quantity_by_priority(full_target, priority)?;
        let mut remaining = target.saturating_sub(stocked);
        let required_purchase = remaining;
        let unit_price = *self
            .regional_prices
            .get(&(region, good))
            .ok_or(WorldError::MissingRegionalPrice { region, good })?;
        let opening_budget = reserve_budget_from_treasury(
            self.countries[&country].indicators().treasury(),
            policy.reserve_monthly_budget(),
        )?;
        let mut budget_remaining = *country_budgets.entry(country).or_insert(opening_budget);
        let budget_before = budget_remaining;
        let sellers: Vec<_> = self
            .firms
            .values()
            .filter(|firm| firm.region() == region && !self.is_firm_insolvent(firm.id()))
            .filter(|firm| {
                self.production_recipes
                    .get(&firm.recipe())
                    .is_some_and(|recipe| recipe.output_good() == good)
            })
            .map(crate::Firm::id)
            .collect();
        let available_supply = sellers.iter().try_fold(0_u64, |sum, seller| {
            let inventory = self
                .firms
                .get(seller)
                .ok_or(WorldError::UnknownFirm(*seller))?
                .inventories()
                .get(&good)
                .copied()
                .unwrap_or_default()
                .get();
            sum.checked_add(inventory)
                .ok_or(WorldError::ArithmeticOverflow("reserve available supply"))
        })?;
        let affordable_quantity = if budget_before <= 0 {
            0
        } else {
            u64::try_from(
                i128::from(budget_before)
                    .checked_mul(i128::from(QuantityMilli::SCALE))
                    .ok_or(WorldError::ArithmeticOverflow(
                        "reserve review affordability",
                    ))?
                    / i128::from(unit_price.minor_units()),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("reserve review affordability"))?
        };
        let mut purchases = Vec::new();
        for seller in sellers {
            let Some(quantity) = self.affordable_reserve_purchase(
                seller,
                good,
                remaining,
                unit_price,
                budget_remaining,
            )?
            else {
                continue;
            };
            let purchase = crate::WorldCommand::ProcureGovernmentReserve {
                actor,
                seller,
                good,
                quantity,
                unit_price,
            };
            purchase.apply(self)?;
            let cost = Money::from_minor_units(reserve_purchase_cost(quantity, unit_price)?);
            purchases.push(GovernmentReserveProcurement {
                actor,
                country,
                region,
                seller,
                good,
                quantity,
                cost,
            });
            budget_remaining = budget_remaining.saturating_sub(cost.minor_units());
            country_budgets.insert(country, budget_remaining);
            remaining -= quantity.get();
            if remaining == 0 {
                break;
            }
        }
        let purchased_quantity = purchases.iter().try_fold(0_u64, |sum, purchase| {
            sum.checked_add(purchase.quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("reserve reviewed purchases"))
        })?;
        let spending = purchases.iter().try_fold(0_i64, |sum, purchase| {
            sum.checked_add(purchase.cost.minor_units())
                .ok_or(WorldError::ArithmeticOverflow("reserve reviewed spending"))
        })?;
        self.events.append(
            self.date,
            DomainEvent::GovernmentReserveRequirementReviewed {
                actor,
                country,
                region,
                good,
                observed_shortage: QuantityMilli::new(observed_need),
                priority,
                target_stock: QuantityMilli::new(target),
                opening_stock: QuantityMilli::new(stocked),
                available_supply: QuantityMilli::new(available_supply),
                budget_available: Money::from_minor_units(budget_before.max(0)),
                purchased: QuantityMilli::new(purchased_quantity),
                spending: Money::from_minor_units(spending),
                remaining_gap: QuantityMilli::new(remaining),
                supply_limited: available_supply < required_purchase,
                budget_limited: affordable_quantity < required_purchase,
            },
        );
        Ok(purchases)
    }

    fn affordable_reserve_purchase(
        &self,
        seller: FirmId,
        good: GoodId,
        remaining: u64,
        unit_price: Money,
        budget_remaining: i64,
    ) -> Result<Option<QuantityMilli>, WorldError> {
        let firm = self
            .firms
            .get(&seller)
            .ok_or(WorldError::UnknownFirm(seller))?;
        let country = self.regions[&firm.region()].country();
        let inventory = firm
            .inventories()
            .get(&good)
            .copied()
            .unwrap_or_default()
            .get();
        let treasury = self.countries[&country]
            .indicators()
            .treasury()
            .minor_units();
        let available_cash = treasury.min(budget_remaining);
        if inventory == 0 || available_cash <= 0 {
            return Ok(None);
        }
        let affordable = u64::try_from(
            i128::from(available_cash)
                .checked_mul(i128::from(QuantityMilli::SCALE))
                .ok_or(WorldError::ArithmeticOverflow(
                    "reserve procurement affordability",
                ))?
                / i128::from(unit_price.minor_units()),
        )
        .map_err(|_| WorldError::ArithmeticOverflow("reserve procurement affordability"))?;
        let quantity = QuantityMilli::new(remaining.min(inventory).min(affordable));
        Ok((quantity.get() > 0).then_some(quantity))
    }

    /// Charges one month of carrying cost and removes physical spoilage from stock held at month opening.
    /// # Errors
    /// Rejects duplicate execution and applies the entire maintenance stage atomically.
    pub fn execute_monthly_government_reserve_maintenance(
        &mut self,
    ) -> Result<Vec<GovernmentReserveMaintenance>, WorldError> {
        let mut next = self.clone();
        let results = next.execute_monthly_government_reserve_maintenance_in_place()?;
        *self = next;
        Ok(results)
    }

    pub(crate) fn execute_monthly_government_reserve_maintenance_in_place(
        &mut self,
    ) -> Result<Vec<GovernmentReserveMaintenance>, WorldError> {
        if self.last_government_reserve_maintenance_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "government reserve maintenance",
                date: self.date,
            });
        }
        let plans = self.plan_government_reserve_maintenance()?;
        let paid_costs = self.allocate_reserve_maintenance_payments(&plans)?;
        self.debit_reserve_maintenance_payments(&plans, &paid_costs)?;
        let results = self.apply_reserve_maintenance_plans(plans, paid_costs)?;
        self.last_government_reserve_maintenance_date = Some(self.date);
        let totals = summarize_reserve_maintenance(&results)?;
        self.events.append(
            self.date,
            DomainEvent::GovernmentReserveMaintenanceCompleted {
                entries: u64::try_from(results.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("reserve maintenance entries"))?,
                assessed_cost: totals.0,
                paid_cost: totals.1,
                baseline_spoilage: totals.2,
                neglect_spoilage: totals.3,
            },
        );
        Ok(results)
    }

    fn plan_government_reserve_maintenance(
        &self,
    ) -> Result<Vec<ReserveMaintenancePlan>, WorldError> {
        let mut plans = Vec::new();
        for ((region, good), opening_stock) in &self.government_reserves {
            if opening_stock.get() == 0 {
                continue;
            }
            let country = self
                .regions
                .get(region)
                .ok_or(WorldError::UnknownRegion(*region))?
                .country();
            let policy = self
                .government_emergency_policies
                .get(&country)
                .copied()
                .unwrap_or_default();
            if policy.reserve_monthly_spoilage().get() == 0
                && policy.reserve_monthly_carrying_cost().get() == 0
            {
                continue;
            }
            let unit_price = *self.regional_prices.get(&(*region, *good)).ok_or(
                WorldError::MissingRegionalPrice {
                    region: *region,
                    good: *good,
                },
            )?;
            let reference_value =
                Money::from_minor_units(reserve_purchase_cost(*opening_stock, unit_price)?);
            plans.push(ReserveMaintenancePlan {
                country,
                region: *region,
                good: *good,
                opening_stock: *opening_stock,
                reference_value,
                assessed_cost: Money::from_minor_units(scale_money_floor(
                    reference_value,
                    policy.reserve_monthly_carrying_cost(),
                )?),
                baseline_spoilage: QuantityMilli::new(scale_quantity_ceil(
                    opening_stock.get(),
                    u64::from(policy.reserve_monthly_spoilage().get()),
                )?),
            });
        }
        Ok(plans)
    }

    fn debit_reserve_maintenance_payments(
        &mut self,
        plans: &[ReserveMaintenancePlan],
        paid_costs: &[i64],
    ) -> Result<(), WorldError> {
        let mut paid_by_country = std::collections::BTreeMap::<CountryId, i64>::new();
        for (plan, paid) in plans.iter().zip(paid_costs) {
            let total = paid_by_country.entry(plan.country).or_default();
            *total = total
                .checked_add(*paid)
                .ok_or(WorldError::ArithmeticOverflow(
                    "reserve maintenance country payment",
                ))?;
        }
        for (country, paid) in paid_by_country {
            let indicators = self
                .countries
                .get_mut(&country)
                .ok_or(WorldError::UnknownCountry(country))?
                .indicators_mut();
            let treasury = indicators.treasury().minor_units();
            indicators.set_treasury(Money::from_minor_units(treasury - paid));
        }
        Ok(())
    }

    fn apply_reserve_maintenance_plans(
        &mut self,
        plans: Vec<ReserveMaintenancePlan>,
        paid_costs: Vec<i64>,
    ) -> Result<Vec<GovernmentReserveMaintenance>, WorldError> {
        let mut results = Vec::new();
        for (plan, paid_minor) in plans.into_iter().zip(paid_costs) {
            let after_baseline = plan
                .opening_stock
                .get()
                .saturating_sub(plan.baseline_spoilage.get());
            let neglect_spoilage = QuantityMilli::new(scale_quantity_ceil(
                after_baseline,
                reserve_neglect_rate(plan.assessed_cost, paid_minor)?,
            )?);
            let closing_stock =
                QuantityMilli::new(after_baseline.saturating_sub(neglect_spoilage.get()));
            if closing_stock.get() == 0 {
                self.government_reserves.remove(&(plan.region, plan.good));
            } else {
                self.government_reserves
                    .insert((plan.region, plan.good), closing_stock);
            }
            let result = GovernmentReserveMaintenance {
                country: plan.country,
                region: plan.region,
                good: plan.good,
                opening_stock: plan.opening_stock,
                reference_value: plan.reference_value,
                assessed_cost: plan.assessed_cost,
                paid_cost: Money::from_minor_units(paid_minor),
                baseline_spoilage: plan.baseline_spoilage,
                neglect_spoilage,
                closing_stock,
            };
            self.events.append(
                self.date,
                DomainEvent::GovernmentReserveMaintained {
                    country: result.country,
                    region: result.region,
                    good: result.good,
                    opening_stock: result.opening_stock,
                    reference_value: result.reference_value,
                    assessed_cost: result.assessed_cost,
                    paid_cost: result.paid_cost,
                    baseline_spoilage: result.baseline_spoilage,
                    neglect_spoilage: result.neglect_spoilage,
                    closing_stock: result.closing_stock,
                },
            );
            results.push(result);
        }
        Ok(results)
    }

    fn allocate_reserve_maintenance_payments(
        &self,
        plans: &[ReserveMaintenancePlan],
    ) -> Result<Vec<i64>, WorldError> {
        let mut paid = vec![0_i64; plans.len()];
        let mut countries = std::collections::BTreeSet::new();
        countries.extend(plans.iter().map(|plan| plan.country));
        for country in countries {
            let indices: Vec<_> = plans
                .iter()
                .enumerate()
                .filter_map(|(index, plan)| (plan.country == country).then_some(index))
                .collect();
            let total_assessed = indices.iter().try_fold(0_i64, |sum, index| {
                sum.checked_add(plans[*index].assessed_cost.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow(
                        "reserve maintenance assessment",
                    ))
            })?;
            if total_assessed <= 0 {
                continue;
            }
            let treasury = self
                .countries
                .get(&country)
                .ok_or(WorldError::UnknownCountry(country))?
                .indicators()
                .treasury()
                .minor_units()
                .max(0);
            let available = treasury.min(total_assessed);
            let mut ranking = Vec::new();
            let mut allocated = 0_i64;
            for index in &indices {
                let assessed = plans[*index].assessed_cost.minor_units();
                let numerator = i128::from(assessed) * i128::from(available);
                let base = i64::try_from(numerator / i128::from(total_assessed)).map_err(|_| {
                    WorldError::ArithmeticOverflow("reserve maintenance allocation")
                })?;
                let remainder = numerator % i128::from(total_assessed);
                paid[*index] = base;
                allocated += base;
                ranking.push((*index, remainder));
            }
            ranking.sort_by(
                |(left_index, left_remainder), (right_index, right_remainder)| {
                    right_remainder
                        .cmp(left_remainder)
                        .then_with(|| left_index.cmp(right_index))
                },
            );
            let remainder_units = usize::try_from(available - allocated)
                .map_err(|_| WorldError::ArithmeticOverflow("reserve maintenance remainder"))?;
            for (index, _) in ranking.into_iter().take(remainder_units) {
                paid[index] += 1;
            }
        }
        Ok(paid)
    }

    /// Reviews current-month reserve evidence and adjusts policy only after persistent pressure.
    /// # Errors
    /// Rejects duplicate review and applies all country decisions atomically.
    pub fn execute_observed_government_reserve_policy_review(
        &mut self,
    ) -> Result<Vec<GovernmentReservePolicyReview>, WorldError> {
        let mut next = self.clone();
        let reviews = next.execute_observed_government_reserve_policy_review_in_place()?;
        *self = next;
        Ok(reviews)
    }

    pub(crate) fn execute_observed_government_reserve_policy_review_in_place(
        &mut self,
    ) -> Result<Vec<GovernmentReservePolicyReview>, WorldError> {
        if self.last_government_reserve_policy_review_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "government reserve policy review",
                date: self.date,
            });
        }
        let evidence = self.current_reserve_policy_evidence();
        let countries: Vec<_> = self
            .government_emergency_policies
            .iter()
            .filter_map(|(country, policy)| {
                (policy.physical_shortage_strategy()
                    == crate::PhysicalShortageStrategy::ReserveRelease)
                    .then_some(*country)
            })
            .collect();
        let mut reviews = Vec::new();
        for country in countries {
            let country_evidence = evidence.get(&country).copied().unwrap_or_default();
            reviews.push(self.review_one_reserve_policy(country, country_evidence)?);
        }
        let (priorities_reviewed, priorities_changed) =
            self.review_configured_reserve_priorities()?;
        self.events.append(
            self.date,
            DomainEvent::GovernmentReservePriorityReviewCompleted {
                priorities_reviewed,
                priorities_changed,
            },
        );
        self.last_government_reserve_policy_review_date = Some(self.date);
        self.events.append(
            self.date,
            DomainEvent::GovernmentReservePolicyReviewCompleted {
                countries_reviewed: u64::try_from(reviews.len()).map_err(|_| {
                    WorldError::ArithmeticOverflow("reserve policy countries reviewed")
                })?,
                policies_changed: u64::try_from(
                    reviews
                        .iter()
                        .filter(|review| {
                            review.previous_coverage_months != review.new_coverage_months
                                || review.previous_monthly_budget != review.new_monthly_budget
                        })
                        .count(),
                )
                .map_err(|_| WorldError::ArithmeticOverflow("reserve policies changed"))?,
            },
        );
        Ok(reviews)
    }

    fn current_reserve_policy_evidence(
        &self,
    ) -> std::collections::BTreeMap<CountryId, ReservePolicyEvidence> {
        let mut evidence = std::collections::BTreeMap::new();
        for envelope in self.events.events() {
            if envelope.date() != self.date {
                continue;
            }
            match envelope.event() {
                DomainEvent::GovernmentReserveRequirementReviewed {
                    country,
                    observed_shortage,
                    opening_stock,
                    remaining_gap,
                    supply_limited,
                    budget_limited,
                    ..
                } => {
                    let entry = evidence
                        .entry(*country)
                        .or_insert_with(ReservePolicyEvidence::default);
                    entry.observed_shortage = entry
                        .observed_shortage
                        .saturating_add(observed_shortage.get());
                    entry.opening_stock = entry.opening_stock.saturating_add(opening_stock.get());
                    entry.remaining_gap = entry.remaining_gap.saturating_add(remaining_gap.get());
                    entry.preparedness_pressure |= observed_shortage.get() > 0
                        && opening_stock.get() < observed_shortage.get()
                        && !supply_limited
                        && !budget_limited;
                    entry.budget_pressure |= *budget_limited;
                }
                DomainEvent::GovernmentReserveMaintained {
                    country,
                    baseline_spoilage,
                    neglect_spoilage,
                    ..
                } => {
                    let entry = evidence
                        .entry(*country)
                        .or_insert_with(ReservePolicyEvidence::default);
                    entry.baseline_spoilage = entry
                        .baseline_spoilage
                        .saturating_add(baseline_spoilage.get());
                    entry.neglect_spoilage = entry
                        .neglect_spoilage
                        .saturating_add(neglect_spoilage.get());
                }
                DomainEvent::GovernmentReserveDistributed {
                    country, quantity, ..
                } => {
                    let entry = evidence
                        .entry(*country)
                        .or_insert_with(ReservePolicyEvidence::default);
                    entry.reserve_distributed =
                        entry.reserve_distributed.saturating_add(quantity.get());
                }
                _ => {}
            }
        }
        evidence
    }

    fn review_one_reserve_policy(
        &mut self,
        country: CountryId,
        evidence: ReservePolicyEvidence,
    ) -> Result<GovernmentReservePolicyReview, WorldError> {
        let previous_policy = *self
            .government_emergency_policies
            .get(&country)
            .ok_or(WorldError::UnknownCountry(country))?;
        let pressure = self.reserve_policy_pressure.entry(country).or_default();
        pressure.preparedness =
            update_pressure_streak(pressure.preparedness, evidence.preparedness_pressure);
        pressure.budget_gap = update_pressure_streak(pressure.budget_gap, evidence.budget_pressure);
        pressure.upkeep_stress =
            update_pressure_streak(pressure.upkeep_stress, evidence.neglect_spoilage > 0);
        pressure.waste = update_pressure_streak(
            pressure.waste,
            evidence.baseline_spoilage > 0
                && evidence.observed_shortage == 0
                && evidence.reserve_distributed == 0,
        );
        let (coverage, budget) = adjusted_reserve_policy(
            previous_policy.reserve_coverage_months(),
            previous_policy.reserve_monthly_budget(),
            pressure,
        );
        let new_policy = previous_policy.with_reserve_procurement(coverage, budget)?;
        self.government_emergency_policies
            .insert(country, new_policy);
        let review = GovernmentReservePolicyReview {
            country,
            observed_shortage: QuantityMilli::new(evidence.observed_shortage),
            opening_stock: QuantityMilli::new(evidence.opening_stock),
            remaining_gap: QuantityMilli::new(evidence.remaining_gap),
            baseline_spoilage: QuantityMilli::new(evidence.baseline_spoilage),
            neglect_spoilage: QuantityMilli::new(evidence.neglect_spoilage),
            preparedness_months: pressure.preparedness,
            budget_gap_months: pressure.budget_gap,
            upkeep_stress_months: pressure.upkeep_stress,
            waste_months: pressure.waste,
            previous_coverage_months: previous_policy.reserve_coverage_months(),
            new_coverage_months: coverage,
            previous_monthly_budget: previous_policy.reserve_monthly_budget(),
            new_monthly_budget: budget,
        };
        self.events.append(
            self.date,
            DomainEvent::GovernmentReservePolicyReviewed {
                country,
                observed_shortage: review.observed_shortage,
                opening_stock: review.opening_stock,
                remaining_gap: review.remaining_gap,
                baseline_spoilage: review.baseline_spoilage,
                neglect_spoilage: review.neglect_spoilage,
                preparedness_months: review.preparedness_months,
                budget_gap_months: review.budget_gap_months,
                upkeep_stress_months: review.upkeep_stress_months,
                waste_months: review.waste_months,
                previous_coverage_months: review.previous_coverage_months,
                new_coverage_months: review.new_coverage_months,
                previous_monthly_budget: review.previous_monthly_budget,
                new_monthly_budget: review.new_monthly_budget,
            },
        );
        Ok(review)
    }

    fn review_configured_reserve_priorities(&mut self) -> Result<(u64, u64), WorldError> {
        let evidence = self.current_reserve_priority_evidence();
        let keys: Vec<_> = self.government_reserve_priorities.keys().copied().collect();
        let mut changed = 0_u64;
        for (country, region, good) in &keys {
            let current = self.government_reserve_priority(*country, *region, *good);
            let observed = evidence
                .get(&(*country, *region, *good))
                .copied()
                .unwrap_or_default();
            let imported_share =
                import_reliance_share(observed.local_quantity, observed.imported_quantity)?;
            let pressure = self
                .reserve_priority_pressure
                .entry((*country, *region, *good))
                .or_default();
            pressure.uncovered = update_pressure_streak(
                pressure.uncovered,
                observed.remaining_gap > 0 && !observed.supply_limited,
            );
            pressure.import_reliance = update_pressure_streak(
                pressure.import_reliance,
                observed.imported_quantity > 0
                    && imported_share.get() >= HIGH_HOUSEHOLD_IMPORT_RELIANCE_BPS,
            );
            pressure.idle_spoilage = update_pressure_streak(
                pressure.idle_spoilage,
                observed.baseline_spoilage > 0
                    && observed.remaining_gap == 0
                    && observed.distributed == 0,
            );
            let previous = current;
            let mut next = current;
            let mut revision_reason = ReservePriorityRevisionReason::None;
            if pressure.uncovered >= 3 && current != BasisPoints::FULL {
                next = current.shifted(500);
                pressure.uncovered = 0;
                revision_reason = ReservePriorityRevisionReason::UncoveredGap;
            } else if pressure.import_reliance >= 6 && current != BasisPoints::FULL {
                next = current.shifted(500);
                pressure.import_reliance = 0;
                revision_reason = ReservePriorityRevisionReason::ImportReliance;
            } else if pressure.idle_spoilage >= 6 && current != BasisPoints::ZERO {
                next = current.shifted(-500);
                pressure.idle_spoilage = 0;
                revision_reason = ReservePriorityRevisionReason::IdleSpoilage;
            }
            let uncovered_months = pressure.uncovered;
            let import_reliance_months = pressure.import_reliance;
            let idle_spoilage_months = pressure.idle_spoilage;
            if next != previous {
                let actor = self.emergency_relief_actor(*country).ok_or(
                    WorldError::InvalidEmergencyRelief(
                        "reserve priority review requires a political office holder",
                    ),
                )?;
                crate::WorldCommand::SetGovernmentReservePriority {
                    actor,
                    country: *country,
                    region: *region,
                    good: *good,
                    priority: next,
                }
                .apply(self)?;
                changed = changed.saturating_add(1);
            }
            self.events.append(
                self.date,
                DomainEvent::GovernmentReservePriorityReviewed {
                    country: *country,
                    region: *region,
                    good: *good,
                    remaining_gap: QuantityMilli::new(observed.remaining_gap),
                    baseline_spoilage: QuantityMilli::new(observed.baseline_spoilage),
                    imported_quantity: QuantityMilli::new(observed.imported_quantity),
                    imported_share,
                    uncovered_months,
                    import_reliance_months,
                    idle_spoilage_months,
                    previous_priority: previous,
                    new_priority: next,
                    revision_reason,
                },
            );
        }
        Ok((
            u64::try_from(keys.len())
                .map_err(|_| WorldError::ArithmeticOverflow("reserve priorities reviewed"))?,
            changed,
        ))
    }

    fn current_reserve_priority_evidence(
        &self,
    ) -> std::collections::BTreeMap<(CountryId, RegionId, GoodId), ReservePriorityEvidence> {
        let mut evidence = std::collections::BTreeMap::<
            (CountryId, RegionId, GoodId),
            ReservePriorityEvidence,
        >::new();
        for envelope in self.events.events() {
            if envelope.date() != self.date {
                continue;
            }
            match envelope.event() {
                DomainEvent::GovernmentReserveRequirementReviewed {
                    country,
                    region,
                    good,
                    remaining_gap,
                    supply_limited,
                    ..
                } => {
                    let row = evidence.entry((*country, *region, *good)).or_default();
                    row.remaining_gap = row.remaining_gap.saturating_add(remaining_gap.get());
                    row.supply_limited |= *supply_limited;
                }
                DomainEvent::GovernmentReserveMaintained {
                    country,
                    region,
                    good,
                    baseline_spoilage,
                    ..
                } => {
                    let row = evidence.entry((*country, *region, *good)).or_default();
                    row.baseline_spoilage = row
                        .baseline_spoilage
                        .saturating_add(baseline_spoilage.get());
                }
                DomainEvent::GovernmentReserveDistributed {
                    country,
                    region,
                    good,
                    quantity,
                    ..
                } => {
                    let row = evidence.entry((*country, *region, *good)).or_default();
                    row.distributed = row.distributed.saturating_add(quantity.get());
                }
                DomainEvent::HouseholdImportDependenceObserved {
                    country,
                    region,
                    good,
                    local_quantity,
                    imported_quantity,
                    ..
                } => {
                    let row = evidence.entry((*country, *region, *good)).or_default();
                    row.local_quantity = row.local_quantity.saturating_add(local_quantity.get());
                    row.imported_quantity = row
                        .imported_quantity
                        .saturating_add(imported_quantity.get());
                }
                _ => {}
            }
        }
        evidence
    }

    pub(crate) fn distribute_government_reserves(
        &mut self,
    ) -> Result<Vec<GovernmentReserveDistribution>, WorldError> {
        let mut distributions = Vec::new();
        let unmet: Vec<_> = self
            .unmet_demand
            .iter()
            .filter(|((_, _, tier), quantity)| *tier == NeedTier::Survival && quantity.get() > 0)
            .map(|(key, quantity)| (*key, *quantity))
            .collect();
        for ((cohort, good, tier), missing) in unmet {
            let region = self
                .cohorts
                .get(&cohort)
                .ok_or(WorldError::UnknownCohort(cohort))?
                .region();
            let country = self
                .regions
                .get(&region)
                .ok_or(WorldError::UnknownRegion(region))?
                .country();
            let policy = self
                .government_emergency_policies
                .get(&country)
                .copied()
                .unwrap_or_default();
            if policy.physical_shortage_strategy()
                != crate::PhysicalShortageStrategy::ReserveRelease
            {
                continue;
            }
            let available = self
                .government_reserves
                .get(&(region, good))
                .copied()
                .unwrap_or_default();
            let supplied = QuantityMilli::new(missing.get().min(available.get()));
            if supplied.get() == 0 {
                continue;
            }
            let remaining = available.get() - supplied.get();
            if remaining == 0 {
                self.government_reserves.remove(&(region, good));
            } else {
                self.government_reserves
                    .insert((region, good), QuantityMilli::new(remaining));
            }
            let consumption = self
                .monthly_consumption
                .get(&(cohort, good, tier))
                .copied()
                .unwrap_or_default();
            self.monthly_consumption.insert(
                (cohort, good, tier),
                QuantityMilli::new(
                    consumption
                        .get()
                        .checked_add(supplied.get())
                        .ok_or(WorldError::ArithmeticOverflow("reserve consumption"))?,
                ),
            );
            let still_unmet = missing.get() - supplied.get();
            if still_unmet == 0 {
                self.unmet_demand.remove(&(cohort, good, tier));
            } else {
                self.unmet_demand
                    .insert((cohort, good, tier), QuantityMilli::new(still_unmet));
            }
            self.recompute_cohort_deprivation_pressure(cohort)?;
            self.events.append(
                self.date,
                DomainEvent::GovernmentReserveDistributed {
                    country,
                    region,
                    cohort,
                    good,
                    quantity: supplied,
                },
            );
            distributions.push(GovernmentReserveDistribution {
                country,
                region,
                cohort,
                good,
                quantity: supplied,
            });
        }
        Ok(distributions)
    }

    fn recompute_cohort_deprivation_pressure(
        &mut self,
        cohort: CohortId,
    ) -> Result<(), WorldError> {
        let weight = |tier: NeedTier| -> u128 {
            match tier {
                NeedTier::Survival => 4,
                NeedTier::Participation => 3,
                NeedTier::Development => 2,
                NeedTier::Discretionary => 1,
            }
        };
        let consumed = self
            .monthly_consumption
            .iter()
            .filter(|((id, _, _), _)| *id == cohort)
            .map(|((_, _, tier), quantity)| u128::from(quantity.get()) * weight(*tier))
            .sum::<u128>();
        let unmet = self
            .unmet_demand
            .iter()
            .filter(|((id, _, _), _)| *id == cohort)
            .map(|((_, _, tier), quantity)| u128::from(quantity.get()) * weight(*tier))
            .sum::<u128>();
        let desired = consumed
            .checked_add(unmet)
            .ok_or(WorldError::ArithmeticOverflow(
                "reserve deprivation pressure",
            ))?;
        let bps = if desired == 0 {
            0
        } else {
            unmet * 10_000 / desired
        };
        self.deprivation_pressure.insert(
            cohort,
            crate::BasisPoints::new(
                u16::try_from(bps)
                    .map_err(|_| WorldError::ArithmeticOverflow("reserve deprivation pressure"))?,
            )
            .map_err(|_| WorldError::ArithmeticOverflow("reserve deprivation bounds"))?,
        );
        Ok(())
    }

    #[must_use]
    pub fn government_reserves(
        &self,
    ) -> &std::collections::BTreeMap<(RegionId, GoodId), QuantityMilli> {
        &self.government_reserves
    }
}

fn import_reliance_share(local: u64, imported: u64) -> Result<BasisPoints, WorldError> {
    let total = local
        .checked_add(imported)
        .ok_or(WorldError::ArithmeticOverflow(
            "household import reliance total",
        ))?;
    if total == 0 {
        return Ok(BasisPoints::ZERO);
    }
    let value = imported
        .checked_mul(10_000)
        .ok_or(WorldError::ArithmeticOverflow(
            "household import reliance share",
        ))?
        / total;
    BasisPoints::new(
        u16::try_from(value)
            .map_err(|_| WorldError::ArithmeticOverflow("household import reliance share"))?,
    )
    .map_err(|_| WorldError::ArithmeticOverflow("household import reliance bounds"))
}

fn scale_quantity_by_priority(value: u64, priority: BasisPoints) -> Result<u64, WorldError> {
    if value == 0 || priority == BasisPoints::ZERO {
        return Ok(0);
    }
    divide_ceil(
        value
            .checked_mul(u64::from(priority.get()))
            .ok_or(WorldError::ArithmeticOverflow("reserve priority target"))?,
        10_000,
    )
}

fn reserve_budget_from_treasury(
    treasury: Money,
    budget: crate::BasisPoints,
) -> Result<i64, WorldError> {
    i64::try_from(
        i128::from(treasury.minor_units().max(0))
            .checked_mul(i128::from(budget.get()))
            .ok_or(WorldError::ArithmeticOverflow("reserve procurement budget"))?
            / 10_000,
    )
    .map_err(|_| WorldError::ArithmeticOverflow("reserve procurement budget"))
}

fn reserve_purchase_cost(quantity: QuantityMilli, unit_price: Money) -> Result<i64, WorldError> {
    i64::try_from(
        i128::from(quantity.get())
            .checked_mul(i128::from(unit_price.minor_units()))
            .ok_or(WorldError::ArithmeticOverflow("reserve procurement result"))?
            / i128::from(QuantityMilli::SCALE),
    )
    .map_err(|_| WorldError::ArithmeticOverflow("reserve procurement result"))
}

fn update_pressure_streak(previous: u8, active: bool) -> u8 {
    if active {
        previous.saturating_add(1)
    } else {
        0
    }
}

fn adjusted_reserve_policy(
    mut coverage: u8,
    mut budget: crate::BasisPoints,
    pressure: &mut crate::relief::ReservePolicyPressure,
) -> (u8, crate::BasisPoints) {
    if pressure.upkeep_stress >= 2 {
        if coverage > 1 {
            coverage -= 1;
        } else {
            budget = budget.shifted(-500);
        }
        pressure.upkeep_stress = 0;
    } else if pressure.waste >= 3 && coverage > 1 {
        coverage -= 1;
        pressure.waste = 0;
    } else if pressure.budget_gap >= 2 && budget != crate::BasisPoints::FULL {
        budget = budget.shifted(500);
        pressure.budget_gap = 0;
    } else if pressure.preparedness >= 3 && coverage < 12 {
        coverage += 1;
        pressure.preparedness = 0;
    }
    (coverage, budget)
}

fn reserve_neglect_rate(assessed_cost: Money, paid_minor: i64) -> Result<u64, WorldError> {
    let assessed_minor = assessed_cost.minor_units();
    if assessed_minor <= paid_minor || assessed_minor <= 0 {
        return Ok(0);
    }
    let unpaid = u64::try_from(assessed_minor - paid_minor)
        .map_err(|_| WorldError::ArithmeticOverflow("reserve unpaid maintenance"))?;
    let assessed = u64::try_from(assessed_minor)
        .map_err(|_| WorldError::ArithmeticOverflow("reserve assessed maintenance"))?;
    divide_ceil(
        unpaid
            .checked_mul(MAX_UNFUNDED_MAINTENANCE_SPOILAGE_BPS)
            .ok_or(WorldError::ArithmeticOverflow("reserve neglect rate"))?,
        assessed,
    )
}

fn scale_money_floor(value: Money, rate: crate::BasisPoints) -> Result<i64, WorldError> {
    i64::try_from(
        i128::from(value.minor_units().max(0))
            .checked_mul(i128::from(rate.get()))
            .ok_or(WorldError::ArithmeticOverflow("reserve carrying cost"))?
            / 10_000,
    )
    .map_err(|_| WorldError::ArithmeticOverflow("reserve carrying cost"))
}

fn scale_quantity_ceil(value: u64, rate_bps: u64) -> Result<u64, WorldError> {
    if value == 0 || rate_bps == 0 {
        return Ok(0);
    }
    let numerator = value
        .checked_mul(rate_bps)
        .ok_or(WorldError::ArithmeticOverflow("reserve spoilage"))?;
    Ok(divide_ceil(numerator, 10_000)?.min(value))
}

fn divide_ceil(numerator: u64, denominator: u64) -> Result<u64, WorldError> {
    if denominator == 0 {
        return Err(WorldError::ArithmeticOverflow("reserve division by zero"));
    }
    numerator
        .checked_add(denominator - 1)
        .ok_or(WorldError::ArithmeticOverflow("reserve rounded division"))
        .map(|value| value / denominator)
}

fn summarize_reserve_maintenance(
    results: &[GovernmentReserveMaintenance],
) -> Result<(Money, Money, QuantityMilli, QuantityMilli), WorldError> {
    let mut assessed = 0_i64;
    let mut paid = 0_i64;
    let mut baseline = 0_u64;
    let mut neglect = 0_u64;
    for result in results {
        assessed = assessed
            .checked_add(result.assessed_cost.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("reserve assessed total"))?;
        paid = paid
            .checked_add(result.paid_cost.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("reserve paid total"))?;
        baseline = baseline.checked_add(result.baseline_spoilage.get()).ok_or(
            WorldError::ArithmeticOverflow("reserve baseline spoilage total"),
        )?;
        neglect = neglect.checked_add(result.neglect_spoilage.get()).ok_or(
            WorldError::ArithmeticOverflow("reserve neglect spoilage total"),
        )?;
    }
    Ok((
        Money::from_minor_units(assessed),
        Money::from_minor_units(paid),
        QuantityMilli::new(baseline),
        QuantityMilli::new(neglect),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, AgeBand, BasisPoints, ConsumptionProfile, Country, CountryIndicators, DemandBasis,
        EducationLevel, EmergencyReliefStrategy, EmploymentStatus, Firm, Good,
        GovernmentEmergencyPolicy, HouseholdCohort, HouseholdType, NeedProfileId, Population,
        PowerNode, PowerNodeId, PowerNodeKind, ProductionRecipe, RecipeId, Region, SimDate,
        WorldCommand, WorldSeed,
    };
    use std::collections::BTreeMap;

    #[allow(clippy::too_many_lines)]
    fn reserve_world() -> World {
        let mut world = World::new(WorldSeed::new(19), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(
                Country::new(CountryId::new(1), "A")
                    .expect("country")
                    .with_indicators(CountryIndicators::new(
                        Money::from_minor_units(100),
                        Money::default(),
                        BasisPoints::HALF,
                        BasisPoints::HALF,
                    )),
            )
            .expect("country");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(1),
                    Money::from_minor_units(100),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Minister", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world
            .register_power_node(
                PowerNode::new(
                    PowerNodeId::new(1),
                    CountryId::new(1),
                    "Cabinet",
                    PowerNodeKind::PoliticalOffice,
                    Some(ActorId::new(1)),
                )
                .expect("office"),
            )
            .expect("office");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(10),
            )
            .expect("price");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Food recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe");
        let mut inventory = BTreeMap::new();
        inventory.insert(GoodId::new(1), QuantityMilli::new(1_000));
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Farm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::default(),
                    inventory,
                )
                .expect("firm"),
            )
            .expect("firm");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Households",
                    vec![crate::ConsumptionTarget::new(
                        GoodId::new(1),
                        crate::NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(1),
                    1,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Unemployed,
                    Money::default(),
                    Money::default(),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world
            .set_government_emergency_policy(
                ActorId::new(1),
                CountryId::new(1),
                GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
                    .with_physical_shortage_strategy(
                        crate::PhysicalShortageStrategy::ReserveRelease,
                    ),
            )
            .expect("reserve policy");
        world.unmet_demand.insert(
            (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
            QuantityMilli::new(1_000),
        );
        world.deprivation_pressure.insert(
            CohortId::new(1),
            BasisPoints::new(10_000).expect("full basis points"),
        );
        world
    }

    #[test]
    fn treasury_procurement_moves_stock_then_reserve_release_closes_shortage() {
        let mut direct = reserve_world();
        let mut replayed = direct.clone();
        let command = WorldCommand::ProcureGovernmentReserve {
            actor: ActorId::new(1),
            seller: FirmId::new(1),
            good: GoodId::new(1),
            quantity: QuantityMilli::new(1_000),
            unit_price: Money::from_minor_units(10),
        };
        command.apply(&mut direct).expect("procurement");
        command.apply(&mut replayed).expect("replayed procurement");
        assert_eq!(direct, replayed);
        assert_eq!(
            direct.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::from_minor_units(90)
        );
        assert_eq!(
            direct.firms[&FirmId::new(1)].cash(),
            Money::from_minor_units(10)
        );
        assert_eq!(
            direct.government_reserves()[&(RegionId::new(1), GoodId::new(1))],
            QuantityMilli::new(1_000)
        );

        let distributions = direct
            .distribute_government_reserves()
            .expect("reserve distribution");
        assert_eq!(distributions.len(), 1);
        assert_eq!(distributions[0].quantity, QuantityMilli::new(1_000));
        assert!(direct.government_reserves().is_empty());
        assert!(direct.unmet_demand().is_empty());
        assert_eq!(
            direct.monthly_consumption()[&(CohortId::new(1), GoodId::new(1), NeedTier::Survival)],
            QuantityMilli::new(1_000)
        );
        assert_eq!(direct.deprivation_pressure()[&CohortId::new(1)].get(), 0);
    }

    #[test]
    fn observed_shortage_procures_available_stock_once_and_replays() {
        let mut direct = reserve_world();
        let mut replayed = direct.clone();

        let purchases = direct
            .execute_observed_government_reserve_procurement()
            .expect("observed procurement");
        WorldCommand::ExecuteObservedGovernmentReserveProcurement
            .apply(&mut replayed)
            .expect("replayed observed procurement");

        assert_eq!(purchases.len(), 1);
        assert_eq!(purchases[0].quantity, QuantityMilli::new(1_000));
        assert_eq!(purchases[0].cost, Money::from_minor_units(10));
        assert_eq!(direct, replayed);
        assert_eq!(
            direct.government_reserves()[&(RegionId::new(1), GoodId::new(1))],
            QuantityMilli::new(1_000)
        );
        assert_eq!(
            direct.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::from_minor_units(90)
        );

        let before_duplicate = direct.clone();
        let duplicate = direct.execute_observed_government_reserve_procurement();
        assert!(matches!(
            duplicate,
            Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed government reserve procurement",
                ..
            })
        ));
        assert_eq!(direct, before_duplicate);
    }

    #[test]
    fn reserve_coverage_target_retains_future_month_after_current_release() {
        let mut world = reserve_world();
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("farm")
            .credit_inventory(GoodId::new(1), QuantityMilli::new(1_000))
            .expect("extra inventory");
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(2, BasisPoints::FULL)
            .expect("two-month policy");
        world
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("policy");

        let purchases = world
            .execute_observed_government_reserve_procurement()
            .expect("procurement");
        assert_eq!(purchases[0].quantity, QuantityMilli::new(2_000));
        assert_eq!(purchases[0].cost, Money::from_minor_units(20));
        world
            .distribute_government_reserves()
            .expect("distribution");
        assert_eq!(
            world.government_reserves()[&(RegionId::new(1), GoodId::new(1))],
            QuantityMilli::new(1_000)
        );
        assert!(world.unmet_demand().is_empty());
    }

    #[test]
    fn one_country_budget_is_shared_across_competing_reserve_goods() {
        let mut world = reserve_world();
        world
            .register_good(Good::new(GoodId::new(2), "Medicine").expect("medicine"))
            .expect("medicine");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(2),
                Money::from_minor_units(10),
            )
            .expect("medicine price");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(2),
                    "Medicine recipe",
                    GoodId::new(2),
                    QuantityMilli::new(1_000),
                    1,
                    vec![],
                )
                .expect("medicine recipe"),
            )
            .expect("medicine recipe");
        let mut medicine = BTreeMap::new();
        medicine.insert(GoodId::new(2), QuantityMilli::new(1_000));
        world
            .register_firm(
                Firm::new(
                    FirmId::new(2),
                    "Pharmacy",
                    RegionId::new(1),
                    RecipeId::new(2),
                    1,
                    1,
                    Money::default(),
                    medicine,
                )
                .expect("pharmacy"),
            )
            .expect("pharmacy");
        world.unmet_demand.insert(
            (CohortId::new(1), GoodId::new(2), NeedTier::Survival),
            QuantityMilli::new(1_000),
        );
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(1, BasisPoints::new(1_000).expect("ten percent budget"))
            .expect("budget policy");
        world
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("policy");

        let purchases = world
            .execute_observed_government_reserve_procurement()
            .expect("procurement");
        assert_eq!(purchases.len(), 1);
        assert_eq!(purchases[0].good, GoodId::new(1));
        assert_eq!(purchases[0].cost, Money::from_minor_units(10));
        assert!(
            !world
                .government_reserves()
                .contains_key(&(RegionId::new(1), GoodId::new(2)))
        );
        assert_eq!(
            world.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::from_minor_units(90)
        );
    }

    #[test]
    fn reserve_maintenance_charges_storage_and_neglect_loss_and_replays() {
        let build = |treasury: i64| {
            let mut world = reserve_world();
            world
                .countries
                .get_mut(&CountryId::new(1))
                .expect("country")
                .indicators_mut()
                .set_treasury(Money::from_minor_units(treasury));
            world.government_reserves.insert(
                (RegionId::new(1), GoodId::new(1)),
                QuantityMilli::new(1_000),
            );
            let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
                .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
                .with_reserve_storage(
                    BasisPoints::new(1_000).expect("ten percent spoilage"),
                    BasisPoints::new(1_000).expect("ten percent carrying cost"),
                );
            world
                .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
                .expect("storage policy");
            world
        };

        let mut direct = build(100);
        let mut replayed = direct.clone();
        let maintenance = direct
            .execute_monthly_government_reserve_maintenance()
            .expect("maintenance");
        WorldCommand::ExecuteMonthlyGovernmentReserveMaintenance
            .apply(&mut replayed)
            .expect("replayed maintenance");
        assert_eq!(direct, replayed);
        assert_eq!(maintenance.len(), 1);
        assert_eq!(maintenance[0].reference_value, Money::from_minor_units(10));
        assert_eq!(maintenance[0].assessed_cost, Money::from_minor_units(1));
        assert_eq!(maintenance[0].paid_cost, Money::from_minor_units(1));
        assert_eq!(maintenance[0].baseline_spoilage, QuantityMilli::new(100));
        assert_eq!(maintenance[0].neglect_spoilage, QuantityMilli::new(0));
        assert_eq!(maintenance[0].closing_stock, QuantityMilli::new(900));
        assert_eq!(
            direct.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::from_minor_units(99)
        );
        let before_duplicate = direct.clone();
        assert!(matches!(
            direct.execute_monthly_government_reserve_maintenance(),
            Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "government reserve maintenance",
                ..
            })
        ));
        assert_eq!(direct, before_duplicate);

        let mut unfunded = build(0);
        let maintenance = unfunded
            .execute_monthly_government_reserve_maintenance()
            .expect("unfunded maintenance");
        assert_eq!(maintenance[0].paid_cost, Money::default());
        assert_eq!(maintenance[0].baseline_spoilage, QuantityMilli::new(100));
        assert_eq!(maintenance[0].neglect_spoilage, QuantityMilli::new(225));
        assert_eq!(maintenance[0].closing_stock, QuantityMilli::new(675));
        assert_eq!(
            unfunded.government_reserves()[&(RegionId::new(1), GoodId::new(1))],
            QuantityMilli::new(675)
        );
    }

    #[test]
    fn new_reserve_stock_is_first_maintained_in_the_following_month() {
        let mut world = reserve_world();
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("farm")
            .credit_inventory(GoodId::new(1), QuantityMilli::new(1_000))
            .expect("second coverage month");
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(2, BasisPoints::FULL)
            .expect("coverage policy")
            .with_reserve_storage(
                BasisPoints::new(1_000).expect("spoilage"),
                BasisPoints::new(1_000).expect("carrying cost"),
            );
        world
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("storage policy");

        let first = world
            .execute_monthly_economic_cycle()
            .expect("purchase month");
        assert!(first.reserve_maintenance.is_empty());
        assert_eq!(
            world.government_reserves()[&(RegionId::new(1), GoodId::new(1))],
            QuantityMilli::new(1_000)
        );

        let second = world
            .execute_monthly_economic_cycle()
            .expect("maintenance month");
        assert_eq!(second.reserve_maintenance.len(), 1);
        assert_eq!(
            second.reserve_maintenance[0].opening_stock,
            QuantityMilli::new(1_000)
        );
        assert_eq!(
            second.reserve_maintenance[0].baseline_spoilage,
            QuantityMilli::new(100)
        );
        assert_eq!(
            second.reserve_maintenance[0].paid_cost,
            Money::from_minor_units(1)
        );
    }

    #[test]
    fn scarce_storage_budget_is_shared_proportionally_across_reserve_goods() {
        let mut world = reserve_world();
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(10));
        world
            .register_good(Good::new(GoodId::new(2), "Medicine").expect("medicine"))
            .expect("medicine");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(2),
                Money::from_minor_units(10),
            )
            .expect("medicine price");
        world.government_reserves.insert(
            (RegionId::new(1), GoodId::new(1)),
            QuantityMilli::new(1_000),
        );
        world.government_reserves.insert(
            (RegionId::new(1), GoodId::new(2)),
            QuantityMilli::new(1_000),
        );
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_storage(BasisPoints::ZERO, BasisPoints::FULL);
        world
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("storage policy");

        let maintenance = world
            .execute_monthly_government_reserve_maintenance()
            .expect("maintenance");
        assert_eq!(maintenance.len(), 2);
        for result in maintenance {
            assert_eq!(result.assessed_cost, Money::from_minor_units(10));
            assert_eq!(result.paid_cost, Money::from_minor_units(5));
            assert_eq!(result.baseline_spoilage, QuantityMilli::new(0));
            assert_eq!(result.neglect_spoilage, QuantityMilli::new(125));
            assert_eq!(result.closing_stock, QuantityMilli::new(875));
        }
        assert_eq!(
            world.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::default()
        );
    }

    #[test]
    fn recurring_unbuffered_shortage_gradually_raises_reserve_coverage_and_replays() {
        let mut world = reserve_world();
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(1, BasisPoints::new(5_000).expect("budget"))
            .expect("policy");
        world
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("reserve policy");
        for month in 0..3 {
            append_policy_requirement(&mut world, false, false);
            if month < 2 {
                let review = world
                    .execute_observed_government_reserve_policy_review()
                    .expect("policy review");
                assert_eq!(review[0].new_coverage_months, 1);
                world.advance_month().expect("next month");
            }
        }
        let mut replayed = world.clone();
        let review = world
            .execute_observed_government_reserve_policy_review()
            .expect("third review");
        WorldCommand::ExecuteObservedGovernmentReservePolicyReview
            .apply(&mut replayed)
            .expect("replayed review");
        assert_eq!(world, replayed);
        assert_eq!(review[0].previous_coverage_months, 1);
        assert_eq!(review[0].new_coverage_months, 2);
        assert_eq!(review[0].preparedness_months, 0);
        let before_duplicate = world.clone();
        assert!(matches!(
            world.execute_observed_government_reserve_policy_review(),
            Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "government reserve policy review",
                ..
            })
        ));
        assert_eq!(world, before_duplicate);
    }

    #[test]
    fn persistent_budget_gap_expands_authority_but_upkeep_stress_retranches_coverage() {
        let mut budget_world = reserve_world();
        let budget_policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(2, BasisPoints::new(4_000).expect("budget"))
            .expect("policy");
        budget_world
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), budget_policy)
            .expect("reserve policy");
        budget_world
            .reserve_policy_pressure
            .entry(CountryId::new(1))
            .or_default()
            .budget_gap = 1;
        append_policy_requirement(&mut budget_world, true, false);
        let review = budget_world
            .execute_observed_government_reserve_policy_review()
            .expect("budget review");
        assert_eq!(review[0].previous_monthly_budget.get(), 4_000);
        assert_eq!(review[0].new_monthly_budget.get(), 4_500);
        assert_eq!(review[0].budget_gap_months, 0);

        let mut stressed = reserve_world();
        let stressed_policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(3, BasisPoints::new(4_000).expect("budget"))
            .expect("policy");
        stressed
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), stressed_policy)
            .expect("reserve policy");
        stressed
            .reserve_policy_pressure
            .entry(CountryId::new(1))
            .or_default()
            .upkeep_stress = 1;
        stressed.events.append(
            stressed.date,
            DomainEvent::GovernmentReserveMaintained {
                country: CountryId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                opening_stock: QuantityMilli::new(1_000),
                reference_value: Money::from_minor_units(10),
                assessed_cost: Money::from_minor_units(2),
                paid_cost: Money::from_minor_units(1),
                baseline_spoilage: QuantityMilli::new(10),
                neglect_spoilage: QuantityMilli::new(100),
                closing_stock: QuantityMilli::new(890),
            },
        );
        let review = stressed
            .execute_observed_government_reserve_policy_review()
            .expect("upkeep review");
        assert_eq!(review[0].previous_coverage_months, 3);
        assert_eq!(review[0].new_coverage_months, 2);
        assert_eq!(review[0].upkeep_stress_months, 0);
    }

    fn append_policy_requirement(world: &mut World, budget_limited: bool, supply_limited: bool) {
        world.events.append(
            world.date,
            DomainEvent::GovernmentReserveRequirementReviewed {
                actor: ActorId::new(1),
                country: CountryId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                observed_shortage: QuantityMilli::new(1_000),
                priority: BasisPoints::FULL,
                target_stock: QuantityMilli::new(1_000),
                opening_stock: QuantityMilli::new(0),
                available_supply: QuantityMilli::new(1_000),
                budget_available: Money::from_minor_units(10),
                purchased: QuantityMilli::new(0),
                spending: Money::default(),
                remaining_gap: QuantityMilli::new(1_000),
                supply_limited,
                budget_limited,
            },
        );
    }

    #[test]
    fn reserve_review_explains_supply_and_budget_constraints() {
        let mut supply_limited = reserve_world();
        supply_limited
            .firms
            .get_mut(&FirmId::new(1))
            .expect("farm")
            .debit_inventory(GoodId::new(1), QuantityMilli::new(1_000))
            .expect("remove supply");
        assert!(
            supply_limited
                .execute_observed_government_reserve_procurement()
                .expect("supply review")
                .is_empty()
        );
        assert!(supply_limited.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::GovernmentReserveRequirementReviewed { good, purchased, remaining_gap, supply_limited: true, budget_limited: false, .. }
                if *good == GoodId::new(1) && purchased.get() == 0 && remaining_gap.get() == 1_000
        )));

        let mut budget_limited = reserve_world();
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(1, BasisPoints::new(0).expect("zero budget"))
            .expect("budget policy");
        budget_limited
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("policy");
        assert!(
            budget_limited
                .execute_observed_government_reserve_procurement()
                .expect("budget review")
                .is_empty()
        );
        assert!(budget_limited.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::GovernmentReserveRequirementReviewed { good, budget_available, purchased, remaining_gap, supply_limited: false, budget_limited: true, .. }
                if *good == GoodId::new(1) && budget_available.minor_units() == 0 && purchased.get() == 0 && remaining_gap.get() == 1_000
        )));
    }

    #[test]
    fn differentiated_priorities_scale_targets_and_allocate_scarce_budget_first() {
        let mut direct = reserve_world();
        direct
            .register_good(Good::new(GoodId::new(2), "Medicine").expect("medicine"))
            .expect("medicine");
        direct
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(2),
                Money::from_minor_units(10),
            )
            .expect("medicine price");
        direct
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(2),
                    "Medicine recipe",
                    GoodId::new(2),
                    QuantityMilli::new(1_000),
                    1,
                    vec![],
                )
                .expect("medicine recipe"),
            )
            .expect("medicine recipe");
        let mut medicine = BTreeMap::new();
        medicine.insert(GoodId::new(2), QuantityMilli::new(1_000));
        direct
            .register_firm(
                Firm::new(
                    FirmId::new(2),
                    "Pharmacy",
                    RegionId::new(1),
                    RecipeId::new(2),
                    1,
                    1,
                    Money::default(),
                    medicine,
                )
                .expect("pharmacy"),
            )
            .expect("pharmacy");
        direct.unmet_demand.insert(
            (CohortId::new(1), GoodId::new(2), NeedTier::Survival),
            QuantityMilli::new(1_000),
        );
        let policy = GovernmentEmergencyPolicy::new(EmergencyReliefStrategy::TreasuryOnly)
            .with_physical_shortage_strategy(crate::PhysicalShortageStrategy::ReserveRelease)
            .with_reserve_procurement(1, BasisPoints::new(1_000).expect("ten percent budget"))
            .expect("budget policy");
        direct
            .set_government_emergency_policy(ActorId::new(1), CountryId::new(1), policy)
            .expect("policy");
        WorldCommand::SetGovernmentReservePriority {
            actor: ActorId::new(1),
            country: CountryId::new(1),
            region: RegionId::new(1),
            good: GoodId::new(1),
            priority: BasisPoints::HALF,
        }
        .apply(&mut direct)
        .expect("food priority");
        WorldCommand::SetGovernmentReservePriority {
            actor: ActorId::new(1),
            country: CountryId::new(1),
            region: RegionId::new(1),
            good: GoodId::new(2),
            priority: BasisPoints::FULL,
        }
        .apply(&mut direct)
        .expect("medicine priority");
        let mut replayed = direct.clone();

        let purchases = direct
            .execute_observed_government_reserve_procurement()
            .expect("priority procurement");
        WorldCommand::ExecuteObservedGovernmentReserveProcurement
            .apply(&mut replayed)
            .expect("replayed procurement");
        assert_eq!(direct, replayed);
        assert_eq!(purchases.len(), 1);
        assert_eq!(purchases[0].good, GoodId::new(2));
        assert_eq!(purchases[0].quantity, QuantityMilli::new(1_000));
        assert!(
            !direct
                .government_reserves()
                .contains_key(&(RegionId::new(1), GoodId::new(1)))
        );
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::GovernmentReserveRequirementReviewed {
                good,
                priority,
                target_stock,
                ..
            } if *good == GoodId::new(1)
                && *priority == BasisPoints::HALF
                && target_stock.get() == 500
        )));
    }

    #[test]
    fn repeated_uncovered_priority_target_recovers_through_shared_command() {
        let mut world = reserve_world();
        WorldCommand::SetGovernmentReservePriority {
            actor: ActorId::new(1),
            country: CountryId::new(1),
            region: RegionId::new(1),
            good: GoodId::new(1),
            priority: BasisPoints::HALF,
        }
        .apply(&mut world)
        .expect("initial priority");
        for month in 0..3 {
            world.events.append(
                world.date,
                DomainEvent::GovernmentReserveRequirementReviewed {
                    actor: ActorId::new(1),
                    country: CountryId::new(1),
                    region: RegionId::new(1),
                    good: GoodId::new(1),
                    observed_shortage: QuantityMilli::new(1_000),
                    priority: BasisPoints::HALF,
                    target_stock: QuantityMilli::new(500),
                    opening_stock: QuantityMilli::new(0),
                    available_supply: QuantityMilli::new(1_000),
                    budget_available: Money::from_minor_units(0),
                    purchased: QuantityMilli::new(0),
                    spending: Money::default(),
                    remaining_gap: QuantityMilli::new(500),
                    supply_limited: false,
                    budget_limited: true,
                },
            );
            if month < 2 {
                world
                    .execute_observed_government_reserve_policy_review()
                    .expect("review");
                world.advance_month().expect("month");
            }
        }
        let mut replayed = world.clone();
        WorldCommand::ExecuteObservedGovernmentReservePolicyReview
            .apply(&mut world)
            .expect("third review");
        WorldCommand::ExecuteObservedGovernmentReservePolicyReview
            .apply(&mut replayed)
            .expect("replayed third review");
        assert_eq!(world, replayed);
        assert_eq!(
            world.government_reserve_priorities()
                [&(CountryId::new(1), RegionId::new(1), GoodId::new(1))]
                .get(),
            5_500
        );
        assert!(world.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::GovernmentReservePriorityReviewed {
                previous_priority,
                new_priority,
                ..
            } if previous_priority.get() == 5_000 && new_priority.get() == 5_500
        )));
    }

    #[test]
    fn sustained_high_import_reliance_gradually_raises_configured_priority() {
        let mut world = reserve_world();
        WorldCommand::SetGovernmentReservePriority {
            actor: ActorId::new(1),
            country: CountryId::new(1),
            region: RegionId::new(1),
            good: GoodId::new(1),
            priority: BasisPoints::HALF,
        }
        .apply(&mut world)
        .expect("initial priority");
        for month in 0..6 {
            world.events.append(
                world.date,
                DomainEvent::HouseholdImportDependenceObserved {
                    country: CountryId::new(1),
                    region: RegionId::new(1),
                    good: GoodId::new(1),
                    local_quantity: QuantityMilli::new(0),
                    imported_quantity: QuantityMilli::new(1_000),
                    imported_share: BasisPoints::FULL,
                },
            );
            if month < 5 {
                let before = world.government_reserve_priorities()
                    [&(CountryId::new(1), RegionId::new(1), GoodId::new(1))];
                world
                    .execute_observed_government_reserve_policy_review()
                    .expect("priority review");
                assert_eq!(before, BasisPoints::HALF);
                assert_eq!(
                    world.government_reserve_priorities()
                        [&(CountryId::new(1), RegionId::new(1), GoodId::new(1))],
                    BasisPoints::HALF
                );
                world.advance_month().expect("month");
            }
        }
        let mut replayed = world.clone();
        WorldCommand::ExecuteObservedGovernmentReservePolicyReview
            .apply(&mut world)
            .expect("sixth review");
        WorldCommand::ExecuteObservedGovernmentReservePolicyReview
            .apply(&mut replayed)
            .expect("replayed sixth review");
        assert_eq!(world, replayed);
        assert_eq!(
            world.government_reserve_priorities()
                [&(CountryId::new(1), RegionId::new(1), GoodId::new(1))]
                .get(),
            5_500
        );
        assert!(world.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::GovernmentReservePriorityReviewed {
                imported_share,
                revision_reason: ReservePriorityRevisionReason::ImportReliance,
                previous_priority,
                new_priority,
                ..
            } if *imported_share == BasisPoints::FULL
                && previous_priority.get() == 5_000
                && new_priority.get() == 5_500
        )));
    }

    #[test]
    fn rejected_reserve_purchase_is_atomic() {
        let mut world = reserve_world();
        let before = world.clone();
        let result = world.procure_government_reserve(
            ActorId::new(1),
            FirmId::new(1),
            GoodId::new(1),
            QuantityMilli::new(1_000),
            Money::from_minor_units(101),
        );
        assert!(matches!(
            result,
            Err(WorldError::InsufficientTreasury(country)) if country == CountryId::new(1)
        ));
        assert_eq!(world, before);
    }
}
