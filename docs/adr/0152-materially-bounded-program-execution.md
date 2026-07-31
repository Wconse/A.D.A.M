# ADR 0152: Materially bounded government-program execution

## Status

Accepted for Step 100.3c.

## Context

Money, administrative capacity, and political support are insufficient to deliver a physical program. A program requiring construction or medical goods must stall when those goods do not exist at the receiving region. Existing regional public reserves provide authoritative physical stock that can be consumed without inventing suppliers or double-spending treasury cash.

## Decision

- A government-program charter may declare annual physical requirements by good and quantity. Programs without requirements retain the previous service-only behavior.
- Every required good must exist at declaration time and every quantity must be positive.
- Split each annual requirement across beneficiary regions by the exact charter shares.
- Derive a regional material-availability ratio from local public-reserve stock. The scarcest required good binds execution.
- Final delivery capacity is the minimum of institutional absorption and material availability.
- Consume the proportional physical quantity from the receiving region’s public reserves atomically. No interregional teleportation is allowed.
- Emit typed material-consumption evidence for program, country, region, good, and quantity. Regional delivery evidence also retains the binding material absorption.

## Invariants

1. Program execution cannot consume more stock than the region physically holds.
2. Consumed material disappears from public reserves exactly once.
3. Money cannot substitute for missing material during the same execution.
4. Stock in another region does not satisfy a local requirement without a future logistics transition.
5. Multiple required goods are complements: the lowest fulfillment ratio binds.
6. Programs with no material requirements remain compatible.
7. Requirements, consumption, outcomes, save state, replay, and fingerprints are deterministic.

## Consequences

The execution chain now includes promise, appropriation, administration, political support/resistance, and consumed physical stock. Procurement and transport remain separate decisions; a government that failed to stock or deliver materials receives delay rather than free implementation. `SIMULATION_VERSION` advances 89 -> 90. Temporary project labor and interregional public logistics remain the next physical gates.
