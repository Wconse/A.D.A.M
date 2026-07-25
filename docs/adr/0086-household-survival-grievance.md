# 0086. Household survival shortages as bilateral grievance evidence

## Status

Accepted (step 037).

## Context

Bilateral grievance previously represented only an industrial failure: a firm
ended the month unable to procure an intermediate input while a foreign firm
had offered that input over a direct route that would work in peace. This left
the most socially consequential material failure outside the diplomatic ledger:
households could suffer an unmet survival need, lose health, and die while a
reachable foreign food supplier existed, yet their country acquired no
material grievance.

The timing also matters. Firm procurement consumes an offer book before its
settlement result is known; household shortage is known only after the local
market clears. A single late offer snapshot would erase evidence for firms
whose foreign offer had already been consumed in procurement, breaking the
pre-existing escalation and peace gates.

## Decision

- Extend the existing monthly grievance transition with a second evidence
  source: an entry in `MarketClearing::unmet` accrues grievance only when it is
  a `NeedTier::Survival` need and a foreign firm offered the same good over a
  direct route that could deliver it in peace.
- Keep the existing directed country-pair ledger, +500/-250 basis-point
  dynamics, threshold escalation, event shape, replay command boundary, and
  automatic peace logic unchanged. Firm and household evidence union in the
  same canonical `BTreeSet`, so multiple shortages cannot double-accrue a pair
  in one month.
- Run grievance update after household market settlement, the first point at
  which household shortfall is authoritative. Preserve two distinct immutable
  offer snapshots: the procurement book for firm evidence, and the offer book
  presented to the household market for household evidence.
- No world-state schema or simulation-rule version change is required:
  `SIMULATION_VERSION` remains 34 and the seed-1/50-year baseline fingerprint
  remains `8818694516742230572`.

## Consequences

- Famine can now become a visible, deterministic source of cross-border
  grievance when a foreign supplier and peaceful logistics made supply
  materially reachable; a shortage without a route still blames nobody.
- Gate coverage adds a replay-equality scenario with a suspended local farm,
  an offering foreign farm, an unmet household survival need, and a direct
  route (grievance reaches 500), plus the same world without the route (no
  grievance). Existing firm-grievance and hostility-deescalation gates confirm
  that retaining the procurement snapshot preserves the earlier conflict arc.
- Deliberate limits: household markets still settle locally and do not yet
  perform cross-border retail imports. The foreign offer is evidence of
  supply that peaceful logistics could make available, not an instant retail
  transaction; delivered household pricing, route capacity, tariffs, and
  inventory shipment remain the next economic slice. Affordability alone does
  not assign foreign blame without a matching foreign offer and route.
