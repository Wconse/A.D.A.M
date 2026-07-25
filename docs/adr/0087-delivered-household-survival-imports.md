# 0087. Delivered household survival imports

## Status

Accepted (step 038).

## Context

Step 037 allowed an unmet household survival need to demonstrate a material
cause for bilateral grievance when foreign supply and peaceful logistics
existed, but household markets still cleared only within their own region.
Foreign food could explain a famine diplomatically without actually reaching
the household. Firm procurement already had a deterministic local-first,
delivered-price import rule, but copying offers into destination markets would
risk selling the same foreign inventory more than once and would lose the
seller's source-region accounting.

## Decision

- Generalize market clearing into `clear_market_with_delivery`: it keeps one
  canonical remaining quantity for each source offer, accepts a delivery-cost
  lookup, and preserves each offer's source region in outcomes and settlement.
  `clear_local_market` remains a documented wrapper supplying no import route.
- For each household order, local offers retain strict priority. Only
  `NeedTier::Survival` may then consider foreign offers, ordered by delivered
  price (offer price plus the lowest direct route tariff), source region, and
  seller. The same source inventory is therefore shared across local and
  imported sales.
- The commercial cycle provides `World::direct_market_route_cost`, which
  rejects hostile country pairs and selects the lowest-tariff direct route with
  stable route-ID tie-breaking. The household pays the delivered amount; the
  foreign seller receives it through the existing market settlement and typed
  `MarketTrade` event.
- This changes authoritative outcomes in worlds with reachable household
  imports, so the seed-1/50-year baseline changes to `12100901864703017553`.
  No state schema changes: `SIMULATION_VERSION` remains 34.

## Consequences

- Reachable foreign food can now avert a survival shortage rather than merely
  supply evidence for grievance. The new gate test pins a full import of one
  unit at delivered price 9 + 1 = 10, zero unmet survival demand, no grievance,
  and bit-identical replay.
- Existing local-only callers remain deterministic through the wrapper; firm
  procurement continues to use the same direct-route selector. Full quality
  gates pass with 114 tests.
- Deliberate limits: imports are immediate accounting transfers, not physical
  shipments; route capacity, terminal queues, tariffs beyond the route cost,
  and price-aware household demand planning are not yet modeled. In
  particular, a delivered price above the local reference price can leave a
  household partially unfunded because its demand budget is still planned from
  the regional reference price.
