use std::collections::{BTreeMap, BTreeSet};

use crate::{DomainEvent, FirmId, Money, QuantityMilli, RouteId, World, WorldError};

const PRESSURE_MONTHS_REQUIRED: u8 = 3;
const EXPANSION_SHARE_DIVISOR: u64 = 10;
const PAYBACK_MONTHS: i128 = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteCapacityExpansion {
    pub route: RouteId,
    pub carrier: FirmId,
    pub previous_capacity: QuantityMilli,
    pub added_capacity: QuantityMilli,
    pub cost: Money,
}

impl World {
    /// Updates persistent route pressure and funds bounded expansion after
    /// three consecutive revenue-producing constrained months.
    pub(crate) fn respond_to_route_capacity_pressure(
        &mut self,
        constrained_routes: &BTreeSet<RouteId>,
    ) -> Result<Vec<RouteCapacityExpansion>, WorldError> {
        let routes: Vec<RouteId> = self.logistics_routes.keys().copied().collect();
        let mut expansions = Vec::new();
        for route in routes {
            if !constrained_routes.contains(&route)
                || self.current_route_freight_revenue(route) <= 0
            {
                self.route_capacity_pressure.remove(&route);
                continue;
            }
            let pressure = self
                .route_capacity_pressure
                .get(&route)
                .copied()
                .unwrap_or_default()
                .saturating_add(1)
                .min(PRESSURE_MONTHS_REQUIRED);
            self.route_capacity_pressure.insert(route, pressure);
            if pressure < PRESSURE_MONTHS_REQUIRED {
                continue;
            }
            if let Some(expansion) = self.try_fund_route_expansion(route)? {
                self.route_capacity_pressure.remove(&route);
                expansions.push(expansion);
            }
        }
        Ok(expansions)
    }

    fn current_route_freight_revenue(&self, route: RouteId) -> i128 {
        self.events
            .events()
            .iter()
            .rev()
            .take_while(|event| event.date() == self.date)
            .filter_map(|event| match event.event() {
                DomainEvent::MarketFreightPaid {
                    route: paid_route,
                    amount,
                    ..
                }
                | DomainEvent::FirmProcurementFreightPaid {
                    route: paid_route,
                    amount,
                    ..
                } if *paid_route == route => Some(i128::from(amount.minor_units())),
                _ => None,
            })
            .sum()
    }

    fn try_fund_route_expansion(
        &mut self,
        route_id: RouteId,
    ) -> Result<Option<RouteCapacityExpansion>, WorldError> {
        let route = self
            .logistics_routes
            .get(&route_id)
            .ok_or(WorldError::UnknownLogisticsRoute(route_id))?;
        let carrier = route.carrier().ok_or(WorldError::InvalidLogistics(
            "route expansion requires a carrier",
        ))?;
        if self.is_firm_insolvent(carrier) {
            return Ok(None);
        }
        let previous = route.capacity();
        let added_raw = previous.get().div_ceil(EXPANSION_SHARE_DIVISOR).max(1);
        let added = QuantityMilli::new(added_raw);
        let gross = i128::from(route.cost_per_unit().minor_units())
            .checked_mul(i128::from(added_raw))
            .ok_or(WorldError::ArithmeticOverflow(
                "route expansion monthly revenue",
            ))?;
        let scale = i128::from(QuantityMilli::SCALE);
        let monthly_revenue =
            gross
                .checked_add(scale - 1)
                .ok_or(WorldError::ArithmeticOverflow(
                    "route expansion monthly revenue",
                ))?
                / scale;
        let monthly_revenue = monthly_revenue.max(1);
        let cost = monthly_revenue
            .checked_mul(PAYBACK_MONTHS)
            .ok_or(WorldError::ArithmeticOverflow("route expansion cost"))?;
        let cost = Money::from_minor_units(
            i64::try_from(cost)
                .map_err(|_| WorldError::ArithmeticOverflow("route expansion cost"))?,
        );
        let firm = self
            .firms
            .get(&carrier)
            .ok_or(WorldError::UnknownFirm(carrier))?;
        if firm.cash().minor_units() < cost.minor_units() {
            return Ok(None);
        }
        self.firms
            .get_mut(&carrier)
            .ok_or(WorldError::UnknownFirm(carrier))?
            .debit_cash(cost)?;
        self.logistics_routes
            .get_mut(&route_id)
            .ok_or(WorldError::UnknownLogisticsRoute(route_id))?
            .add_capacity(added)?;
        let expansion = RouteCapacityExpansion {
            route: route_id,
            carrier,
            previous_capacity: previous,
            added_capacity: added,
            cost,
        };
        self.events.append(
            self.date,
            DomainEvent::RouteCapacityExpanded {
                route: route_id,
                carrier,
                previous_capacity: previous,
                added_capacity: added,
                cost,
            },
        );
        Ok(Some(expansion))
    }

    #[must_use]
    pub fn route_capacity_pressure(&self) -> &BTreeMap<RouteId, u8> {
        &self.route_capacity_pressure
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Country, CountryId, Firm, Good, GoodId, LogisticsRoute, Population, ProductionRecipe,
        RecipeId, Region, RegionId, SimDate, TransportMode, WorldSeed,
    };

    fn expansion_world(cash: i64) -> World {
        let mut world = World::new(WorldSeed::new(50), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Freight proxy").expect("good"))
            .expect("good");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Idle carrier recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe");
        for (id, name) in [(1, "Origin"), (2, "Destination")] {
            world
                .register_region(
                    Region::new(
                        RegionId::new(id),
                        CountryId::new(1),
                        name,
                        Population::new(0),
                        Money::default(),
                    )
                    .expect("region"),
                )
                .expect("region");
        }
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Carrier",
                    RegionId::new(1),
                    RecipeId::new(1),
                    0,
                    0,
                    Money::from_minor_units(cash),
                    BTreeMap::new(),
                )
                .expect("carrier"),
            )
            .expect("carrier");
        world
            .register_logistics_route(
                LogisticsRoute::new(
                    RouteId::new(1),
                    RegionId::new(1),
                    RegionId::new(2),
                    TransportMode::Road,
                    QuantityMilli::new(1_000),
                    Money::from_minor_units(10),
                    1,
                    10_000,
                )
                .expect("route")
                .with_carrier(FirmId::new(1)),
            )
            .expect("route");
        world.events.append(
            world.date,
            DomainEvent::FirmProcurementFreightPaid {
                buyer: FirmId::new(2),
                seller: FirmId::new(3),
                carrier: FirmId::new(1),
                route: RouteId::new(1),
                amount: Money::from_minor_units(10),
            },
        );
        world
    }

    #[test]
    fn three_revenue_producing_constrained_months_fund_bounded_expansion() {
        let mut direct = expansion_world(100);
        let mut replayed = direct.clone();
        let constrained = BTreeSet::from([RouteId::new(1)]);
        for expected in [1, 2] {
            assert!(
                direct
                    .respond_to_route_capacity_pressure(&constrained)
                    .expect("pressure")
                    .is_empty()
            );
            assert!(
                replayed
                    .respond_to_route_capacity_pressure(&constrained)
                    .expect("replayed pressure")
                    .is_empty()
            );
            assert_eq!(direct.route_capacity_pressure()[&RouteId::new(1)], expected);
        }
        let expansions = direct
            .respond_to_route_capacity_pressure(&constrained)
            .expect("expansion");
        let replayed_expansions = replayed
            .respond_to_route_capacity_pressure(&constrained)
            .expect("replayed expansion");
        assert_eq!(expansions, replayed_expansions);
        assert_eq!(expansions.len(), 1);
        assert_eq!(expansions[0].previous_capacity, QuantityMilli::new(1_000));
        assert_eq!(expansions[0].added_capacity, QuantityMilli::new(100));
        assert_eq!(expansions[0].cost, Money::from_minor_units(12));
        assert_eq!(
            direct.logistics_routes()[&RouteId::new(1)].capacity(),
            QuantityMilli::new(1_100)
        );
        assert_eq!(
            direct.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(88)
        );
        assert!(direct.route_capacity_pressure().is_empty());
        assert!(direct.events().events().iter().any(|event| matches!(
            event.event(),
            DomainEvent::RouteCapacityExpanded {
                route,
                carrier,
                previous_capacity,
                added_capacity,
                cost,
            } if *route == RouteId::new(1)
                && *carrier == FirmId::new(1)
                && *previous_capacity == QuantityMilli::new(1_000)
                && *added_capacity == QuantityMilli::new(100)
                && *cost == Money::from_minor_units(12)
        )));
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }

    #[test]
    fn pressure_requires_revenue_and_available_carrier_cash() {
        let constrained = BTreeSet::from([RouteId::new(1)]);
        let mut no_revenue = expansion_world(100);
        no_revenue.events = crate::EventLog::default();
        assert!(
            no_revenue
                .respond_to_route_capacity_pressure(&constrained)
                .expect("no revenue")
                .is_empty()
        );
        assert!(no_revenue.route_capacity_pressure().is_empty());

        let mut no_cash = expansion_world(0);
        for _ in 0..3 {
            assert!(
                no_cash
                    .respond_to_route_capacity_pressure(&constrained)
                    .expect("no cash")
                    .is_empty()
            );
        }
        assert_eq!(no_cash.route_capacity_pressure()[&RouteId::new(1)], 3);
        assert_eq!(
            no_cash.logistics_routes()[&RouteId::new(1)].capacity(),
            QuantityMilli::new(1_000)
        );
    }
}
