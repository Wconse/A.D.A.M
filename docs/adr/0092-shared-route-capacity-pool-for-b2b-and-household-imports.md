# ADR 0092: Shared Route Capacity Pool for B2B and Household Imports

## Status

Accepted

## Context

Step 042 introduced monthly route capacity caps for household survival imports
(`market_spot_route_capacity`). Step 043 found and fixed a double-count in that
cap computation. However, firm-to-firm (B2B) procurement imports were still
unconstrained: a bakery could import unlimited grain over a road that was
already saturated with household food imports.

This created two separate, non-competing import flows sharing the same physical
route. The capacity cap was only cosmetic for B2B trade.

## Decision

Compute a single `market_spot_route_capacity` map once at the start of each
commercial cycle, before any import flow runs, and thread it through all three
stages that consume cross-region route capacity:

1. **B2B procurement** (`execute_monthly_firm_procurement`) — consumes first,
   in canonical order (region, good, buyer).
2. **Household demand planning** (`plan_monthly_household_demand_against_offers`)
   — receives a *clone* of the post-procurement map so the first-canonical-buyer
   reservation logic inside the planner works independently without draining the
   pool that clearing will use.
3. **Market clearing** (`clear_market_with_delivery`) — receives the original
   post-procurement map; actual household import fills are bounded by whatever
   route capacity procurement left unused.

The planner clone keeps existing behaviour (first canonical cohort reserves
scarce import slots during budgeting); clearing enforces the physical limit.

## Consequences

- B2B and household imports compete for the same finite monthly route capacity.
  A bakery procuring grain exhausts capacity that would otherwise be available
  for household food imports on the same route in the same month.
- Route capacity 1 000 with 400 milli-units in a spot shipment in transit
  (step 043 gate) still yields pool 600 — unchanged.
- Existing household import tests are unaffected: in those worlds no firm
  procures cross-region intermediates, so the shared pool arrives at clearing
  intact.
- New gate (`b2b_procurement_capacity` integration tests):
  - Ample capacity (10 000): both B2B Grain import and household Bread flow
    coexist freely.
  - Tight capacity (1 000 == B2B demand): B2B fills in full; no route capacity
    remains for any additional household import on the same route.
  - Insufficient capacity (500 < 1 000 needed): B2B import is capped at 500;
    unmet procurement 500 milli-Grain recorded.
  - Year replayability confirmed.
- `direct_market_route_cost` (thin wrapper around `direct_market_route`) removed
  as it became dead code once procurement switched to `direct_market_route`.
- `SIMULATION_VERSION` unchanged (34); release fingerprint unchanged
  (`12100901864703017553`).
