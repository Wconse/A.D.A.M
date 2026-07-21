# ADR 0003: Atomic causal yearly pipeline

- Status: Accepted
- Date: 2026-07-21

## Context

Stage 0 needs history to emerge from linked demographic, economic, fiscal, and political state rather than isolated random events. The system order and random-number consumption must be stable enough for replay and regression tests.

## Decision

A year is planned and committed in this order:

1. regional demographic change;
2. regional productive-output change;
3. country fiscal closure;
4. legitimacy and elite-cohesion response;
5. calendar advancement.

All changes use integer fixed-point arithmetic. Region randomness is derived from world seed, subsystem domain, region ID, and year. Political randomness uses the same scheme with country ID. Adding a draw to one region or subsystem therefore cannot perturb unrelated histories.

The year is atomic: every arithmetic result is planned and validated before authoritative state is changed. A failing year emits no partial events. Multi-year execution commits completed years one at a time.

## Initial causal links

- output per person and legitimacy affect demographic growth;
- population growth, legitimacy, and elite cohesion affect output growth;
- output and elite cohesion affect tax collection;
- low legitimacy increases fiscal spending pressure;
- deficits consume treasury and then create public debt;
- output growth and fiscal balance affect legitimacy and elite cohesion.

## Consequences

- The model is intentionally stylized and must be calibrated against distributions, not individual anecdotes.
- System-order changes alter history and require a simulation-version bump.
- Interstate tension, trade, conflict, and actor decisions remain future systems and must enter through the same planned-event boundary.
