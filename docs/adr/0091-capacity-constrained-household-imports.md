# 0091: Capacity-constrained household imports

## Status

Accepted.

## Context

Since ADR 0087, household survival demand can be filled by direct foreign
offers at delivered prices. Both the demand planner (ADR 0088-0090) and the
market clearing treated the chosen logistics route as if it had unlimited
monthly throughput: a single cheap foreign offer could satisfy any volume of
household imports even when the route's monthly capacity was already consumed
by freight contracts or in-transit shipments. Physical shipments (`RouteCapacityLedger`)
and contracted freight (`FreightCapacityLedger`) already enforced capacity, so
household market imports were the one flow that ignored the physical network.

## Decision

- `World::market_spot_route_capacity` derives, per route, the uncontracted
  spot capacity available to immediate market imports this month:
  monthly route capacity minus active freight-contract reservations, minus
  spot freight already used, minus in-transit shipment reservations.
- `World::direct_market_route` returns the selected route identity together
  with its tariff (lowest tariff, ties by stable route ID);
  `direct_market_route_cost` delegates to it.
- `clear_market_with_delivery` takes a mutable per-route capacity map. Import
  fills are capped by the remaining capacity of their route and consume it
  across all orders of the month. Routes missing from the map stay
  unconstrained, and local fills never touch the map.
- The demand planner mirrors the same cap: `SurvivalSupplyLedger` carries a
  shared route-capacity copy, and import quotes both respect and (under
  market allocation) consume it in canonical market order, so reserved
  budgets anticipate the capped fills plus reference-priced fallback.

## Consequences

- Household survival imports can no longer exceed the physical monthly
  throughput of the connecting route; excess need becomes explicit unmet
  demand that feeds relief, health, and grievance channels.
- Planner reservations and clearing agree under market allocation, keeping
  reserved budgets consistent with capped fills.
- The demo world keeps its history (fingerprint 12100901864703017553,
  SIMULATION_VERSION 34) because its routes have ample capacity.
- Known limits: firm-to-firm procurement imports are not yet capacity-capped;
  the proportional-rationing planning path caps each cohort's quote but keeps
  independent per-cohort copies; relief purchasing still uses reference
  prices.
