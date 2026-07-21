# Determinism Contract

A.D.A.M promises identical history for identical:

- simulation version;
- world/content schema version;
- seed;
- ordered external inputs;
- target determinism profile.

## Rules

1. All randomness comes from named streams derived from the world seed.
2. Adding random draws to one subsystem must not perturb unrelated subsystems.
3. Simulation time is integer/discrete and never read from the operating system.
4. Behavior cannot depend on thread scheduling or unordered iteration.
5. Canonical state fingerprints use explicit field order and byte encoding.
6. Floating point is not currently part of canonical state. Introducing it requires an ADR defining platform guarantees, quantization, and tests.
7. Event ordering is explicit through a monotonic sequence number.
8. Parallel systems must merge outputs in a deterministic, documented order.

## Test layers

- unit tests for clock, IDs, RNG streams, and event ordering;
- twin-run tests: same seed and inputs must match;
- divergence tests: distinct seeds must affect stochastic outcomes;
- golden fingerprints for accepted long simulations;
- replay tests from commands/events;
- cross-platform CI once additional runners are available.

## Versioning

A change that intentionally alters history must:

1. explain the causal change;
2. update the simulation version;
3. regenerate fingerprints explicitly;
4. preserve migration or declare save incompatibility.
