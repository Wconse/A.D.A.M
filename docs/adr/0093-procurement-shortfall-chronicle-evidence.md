# ADR 0093: Procurement Shortfall Chronicle Evidence

## Status

Accepted

## Context

Firm procurement already returned a canonical `unmet` map after each monthly
settlement. That value informed bilateral-grievance mechanics, but it was not
written to the domain event log. As a result, the console chronicle could
narrate the consequences of a shortage (for example, grievance or survival
loss) but not the immediate material fact: which firm could not obtain which
input and in what quantity.

Step 044 made route capacity a direct source of such B2B shortfalls, increasing
the need for an auditable and readable explanation.

## Decision

Add a `FirmProcurementShortfall { buyer, good, quantity }` domain event.

- `record_procurement_outcomes` writes exactly one event for each canonical
  `(buyer, good)` entry in the settled unmet map.
- The event is evidence only: it adds no authoritative world state and does not
  change resource movement, grievance rules, market allocation, or the stable
  fingerprint.
- The yearly chronicle aggregates repeated monthly shortfalls per `(buyer,
  good)`, resolves current firm and good identities to names, and narrates the
  total as: `Bakery could not procure 500 milli-units of Grain.`
- A B2B-only shortfall gets importance tier 75. Existing stronger signals retain
  priority: survival loss (<50% fulfillment) remains 90 and excess deaths remain
  100.

## Consequences

- Route-capacity scarcity is now visible as a material causal link rather than
  only as an internal quantity or a later diplomatic side effect.
- Repeated monthly shortages are compacted into one yearly sentence per firm and
  input, preserving deterministic ordering and avoiding journal prose spam.
- The B2B capacity gate now proves that a 500 milli-unit route cap produces the
  named Bakery/Grain/500 chronicle sentence and remains replay-identical.
- This changes the serialized event-log vocabulary but not world state. Existing
  snapshot/version compatibility is not affected because the new variant is only
  emitted by new simulation runs.
- Acceptance: the full quality gate passes with 122 tests, and the seed-1
  50-year release run keeps the authoritative fingerprint unchanged at
  `12100901864703017553` (34.7 ms per simulated year, SIMULATION_VERSION 34).
