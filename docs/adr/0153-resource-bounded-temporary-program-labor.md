# ADR 0153: Resource-bounded temporary program labor

## Status

Accepted for Step 100.3d.

## Decision

Government programs may declare an annual temporary-worker requirement. Requirements are split by regional charter shares. Only unemployed regional population not already committed to another program in the same year is available. Labor availability joins administration, infrastructure, politics, and materials as a binding execution ratio.

A fingerprinted `(year, region)` ledger prevents multiple programs from reusing the same workers. Realized program spending is credited as temporary wages to an eligible unemployed cohort, moving committed program money into household wealth instead of destroying it. Temporary participation does not rewrite the cohort’s durable employment status.

## Invariants

1. Annual program labor use cannot exceed regional unemployed population.
2. The same regional labor capacity cannot be used twice in one year.
3. Wages credited to households equal the corresponding delivered program spending.
4. Missing workers create carryover and delay rather than synthetic labor.
5. Infrastructure can independently bind execution.
6. Labor requirements, usage, wages, events, replay, and fingerprints are deterministic.

## Consequences

Program execution now combines fiscal, administrative, infrastructural, political, material, and labor constraints. `SIMULATION_VERSION` advances 90 -> 91. Explicit interregional movement of public materials remains a future logistics expansion, while local absence already prevents teleportation.
