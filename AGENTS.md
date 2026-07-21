# A.D.A.M Agent Guide

## Mission

Build Stage 0 first: a deterministic headless world whose fifty-year chronicle is interesting to read.

## Required workflow

1. Work in one small vertical slice.
2. State invariants and acceptance criteria before code.
3. Keep `adam-core` independent from rendering and UI.
4. Add tests for determinism and causal effects.
5. Run formatting, Clippy, tests, and docs before claiming completion.
6. Never update regression fingerprints merely to make tests green.
7. Update `docs/roadmap.md` after every accepted slice with progress, tests, measurements, approximations, and next gate.
8. Record depth ideas only when they add a decision, feedback loop, historical pattern, failure mode, or conflict; test them before acceptance.

## Architecture boundaries

- `crates/adam-core`: domain state, commands, systems, events, simulation time, deterministic RNG.
- `apps/adam-cli`: argument parsing, configuration loading, invoking the core, rendering text.
- `config`: versioned content data only.
- `docs/adr`: decisions that constrain future work.

## Forbidden shortcuts

- ambient randomness or wall-clock input;
- player-only bonuses;
- simulation state owned by Bevy/UI;
- unordered collections in deterministic paths without canonical sorting;
- floating-point equality used as a persistence contract;
- `unsafe` code without an accepted ADR and dedicated tests.
