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

## Simulation freedom principle

- Reject actions only when they are impossible under authoritative resources, time, physical capacity, contracts, law, or the actor's actual institutional authority.
- Do not add paternalistic engine caps merely to prevent extreme, inefficient, biased, self-destructive, or politically dangerous choices. If a government can legally and materially direct all discretionary spending to one region, the simulation must permit it and model the consequences rather than impose an invisible fairness floor.
- Player and autonomous actors use the same action legality and accounting rules. Never make a feasible action player-only or forbid it globally because the default AI should avoid it.
- Put prudence, fairness, gradualism, risk tolerance, corruption, favoritism, and ideology in actor policy and decision logic, not in universal engine constraints. Different autonomous actors may therefore choose moderate or extreme feasible policies.
- Prefer warnings, opposition, implementation failure, debt, shortages, lost confidence, unrest, and other causal consequences over disabled controls or arbitrary allocation limits.
- Institutional resistance must be represented by actual authority, approval, influence, enforcement, or execution systems. UI and AI may advise against a decision, but they must not silently veto an otherwise executable command.
- Numeric bounds needed for deterministic arithmetic and persistence are valid technical invariants; they must not be disguised gameplay restrictions on agency.

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
