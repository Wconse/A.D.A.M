# 0088. Delivered-price budgeting for household survival demand

## Status

Accepted (step 039).

## Context

Step 038 made household survival imports settle at an offer's delivered price,
but household demand was still planned using only the local regional reference
price. A household with enough money to pay an available foreign offer plus
its route tariff could therefore reserve too little, buy only a fraction of
its survival need, and appear materially short despite having adequate cash.

The commercial cycle already has the authoritative current offer book before
it plans household orders. It can therefore quote a survival target from the
same reachable supply that the market will consider, without adding mutable
state or speculative future prices.

## Decision

- Keep `plan_monthly_household_demand()` as the public local-reference
  wrapper for callers outside a market cycle. Add the crate-visible
  `plan_monthly_household_demand_against_offers()` for the commercial cycle.
- For each survival target, quote the cheapest current local offer if one
  exists. If no local offer exists, quote the cheapest foreign offer plus the
  lowest peaceful direct route tariff. Ties are resolved by the canonical
  offer order; participation and higher tiers continue using their regional
  reference price.
- The commercial cycle passes its current offer book to the new planner before
  it builds household orders. The existing market still settles the same
  local-first rule and delivered prices, so the reserved spend now matches the
  reachable import in the no-local-supply case.
- No state schema or version change: `SIMULATION_VERSION` remains 34, and the
  seed-1/50-year release baseline remains `12100901864703017553`.

## Consequences

- A household with exactly 11 minor units can reserve all 11 for a survival
  import priced at foreign offer 10 plus route tariff 1, buy the full unit,
  avoid unmet demand, and replay identically. This replaces the prior
  artificial 10-unit reservation and partial fulfillment.
- Existing direct callers retain local-reference behavior through the public
  wrapper, keeping demand-unit tests and non-market calculations stable.
- Deliberate limits: a local offer always wins the current market even when a
  foreign delivered offer would be cheaper; partial local supply is not yet
  quoted as a multi-source blended price; offer availability is observed at
  planning time rather than reserved; and relief-cost calculations still use
  local reference prices.
