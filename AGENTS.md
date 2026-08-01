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

## No unowned constraints

Every limit in the economy must name the party that pays for it. A constraint is
admissible only when some modelled actor bears a concrete cost -- money, health,
reputation, forgone profit, or a broken contract -- for the limit being observed.
If nobody pays, the limit is magic and must be deleted rather than tuned.

- Before adding a bound, answer in a comment: who is holding this line, and what
  does it cost them? A bound whose comment cannot answer that does not ship.
- A magic bound is worse than the instability it hides, because it conceals the
  unfinished loop that produced the instability. A price ceiling hides a missing
  supply response; a population growth cap hides births modelled as a rate rather
  than a process. Fix the loop, do not clamp the symptom.
- Reality is not the test of correctness. When output diverges from life, the
  first hypothesis is an unfinished causal loop, not a wrong rule. Never restore
  plausible-looking numbers by bounding a rule that is itself sound.
- Legitimate exceptions are the domain of a type (a share cannot exceed one), the
  discreteness of a unit (currency and people are integers), and a division guard.
  These are arithmetic, not economics, and must be commented as such.
- Elasticities and sensitivities are behaviour, not bounds. Tuning how strongly an
  actor responds is allowed; forbidding the response from being large is not.

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
