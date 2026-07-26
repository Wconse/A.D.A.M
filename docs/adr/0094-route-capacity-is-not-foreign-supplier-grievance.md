# ADR 0094: Route Capacity Is Not Foreign-Supplier Grievance

## Status

Accepted

## Context

Bilateral grievance accrues when a firm has unmet input demand while a foreign
firm offered that input over a direct route that could deliver it in peace.
Step 044 made route capacity a real constraint on B2B imports. Without a
separate cause classification, a foreign offer left unfilled solely because the
shared monthly route pool was exhausted looked identical to a foreign supplier
withholding a reachable good.

That attributes a logistics bottleneck to another country and can create
hostility from domestic infrastructure scarcity.

## Decision

Classify an unmet `(buyer, good)` as `capacity_limited` during procurement
settlement only when all of these hold:

1. the order remains partially unmet;
2. an eligible foreign offer still has positive inventory; and
3. its selected direct route has exhausted the monthly shared capacity pool.

The classification is transient settlement evidence, returned with the monthly
procurement result. It is not authoritative world state and is not serialized
or fingerprinted.

During grievance update, capacity-limited firm shortages are excluded from new
foreign grievance evidence. Existing grievances for that pair still take their
ordinary monthly decay path. Other evidence retains prior behavior: no route
creates no grievance, while a hostile or unavailable foreign offer can still
support the existing material-shortage logic where applicable.

## Consequences

- Cross-border route capacity of 400 against a Bakery demand of 1,000 produces
  a 400-unit fill and explicit 600-unit unmet procurement, but no grievance
  toward the Farm's country.
- Deterministic replay is preserved through the ordinary monthly command
  boundary.
- The condition is intentionally narrow: it does not erase grievance merely
  because a route exists; it requires a still-stocked eligible foreign offer
  and a route pool actually exhausted in the settlement.
- This corrects attribution, not market mechanics: B2B route caps, cash,
  inventory movements, and the priority of local supply are unchanged.
