# ADR 0095: Capacity-Specific Procurement Shortfall Evidence

## Status

Accepted

## Context

Step 046 distinguishes route-capacity-limited firm shortages during settlement
and prevents them from creating bilateral grievance. The event journal still
used the generic `FirmProcurementShortfall` event, however, so the chronicle
could report the missing input but not the material cause already known by the
simulation.

Changing the fields of the existing serialized event would also alter the
contract of historical journal entries.

## Decision

Add `FirmProcurementRouteCapacityShortfall { buyer, good, quantity }` as a new
variant at the end of `DomainEvent`.

- Settlement emits the capacity-specific event instead of the generic event
  when the `(buyer, good)` pair is in the transient `capacity_limited` set.
- All other unmet procurement continues to emit the unchanged
  `FirmProcurementShortfall` event.
- The yearly chronicle aggregates the two causes independently. Generic
  shortages retain `Firm could not procure ...`; capacity shortages render as
  `Route capacity prevented Firm from procuring ...`.
- Both causes retain procurement-shortfall importance tier 75.
- The new variant is appended rather than changing the existing event shape,
  preserving the established serialized representation of earlier variants.

## Consequences

- The event log now exposes the same cause distinction used by diplomacy.
- A 500-unit route cap against 1,000 units of Bakery demand produces one typed
  capacity-shortfall event and named capacity-specific chronicle narration.
- No resources, market ordering, grievance dynamics, authoritative world state,
  persistence schema, or stable fingerprint input changes.
- The event log and chronicle text change only in histories that contain a
  capacity-limited procurement shortfall.
- Acceptance: the full gate passes with 123 tests; the seed-1 50-year release
  fingerprint remains `12100901864703017553` at 34.7 ms per simulated year
  with `SIMULATION_VERSION = 34`.
