# A.D.A.M

**Ambition. Dynasty. Ascension. Memory.**

A.D.A.M is a deterministic, headless-first simulation of a living twenty-first-century society. The long-term game combines grand strategy, mortal characters, 2D tactical battles, and a generated history textbook. The current milestone is deliberately smaller: prove that a causal simulation can produce history worth reading.

> Design law: **“Because this exists in life.”** No player privilege, rubber-banding, scripted rescues, or separate rules for AI actors.

## Current milestone — Stage 0: Console Chronicle

The first playable artifact will simulate several fictional countries for fifty years and emit a readable chronicle. Rendering, Bevy, tactical combat, and full UI remain out of scope until this hypothesis is validated.

### Foundation included

- `adam-core`: engine-independent simulation library;
- `adam-cli`: console runner and future chronicle exporter;
- typed identifiers and discrete simulation time;
- deterministic, explicitly seeded random streams;
- ordered world storage and append-only typed event log;
- household cohort ledger with exact regional population conservation;
- stable state fingerprint for regression checks;
- determinism tests and cross-platform quality scripts;
- architecture and decision records.

## Repository layout

```text
ADAM/
├── apps/adam-cli/          # Console composition root; no domain rules
├── crates/adam-core/       # Headless deterministic simulation
├── config/                 # Data-driven world definitions
├── docs/                   # Architecture, determinism contract, ADRs
├── scripts/                # Reproducible local quality gates
└── .github/workflows/      # CI quality gate
```

## Requirements

- Rust 1.85.0 (pinned by `rust-toolchain.toml`)
- Git

Install Rust with [rustup](https://rustup.rs/), then restart the terminal.

## Quick start

```bash
cargo run -- --seed 1 --years 50
```

The initial scaffold runs a minimal deterministic clock and prints a stable state fingerprint. Simulation systems and prose generation will be added as small tested slices.

## Quality gate

Linux/macOS:

```bash
./scripts/check.sh
```

Windows PowerShell:

```powershell
./scripts/check.ps1
```

The gate checks formatting, compilation, Clippy warnings, tests, and generated documentation.

A reproducible foundation-scale probe is also available:

```bash
cargo run --release -p adam-core --example foundation_scale -- 100000
```

## Non-negotiable invariants

1. Same seed + same ordered inputs + same simulation version = identical history.
2. The renderer never owns domain state.
3. State changes are represented by auditable domain events.
4. Player and AI decisions enter through the same command boundary.
5. Snapshot changes require an explicit design decision; they are not auto-accepted.
6. Wall-clock time, hash-map iteration order, and ambient randomness must not affect the world.

See the project documentation:

- [`docs/roadmap.md`](docs/roadmap.md) — live progress and gates;
- [`docs/design/simulation-plan.md`](docs/design/simulation-plan.md) — full simulation architecture;
- [`docs/design/information-reputation-propaganda.md`](docs/design/information-reputation-propaganda.md) — reputation, media, and propaganda;
- [`docs/design/ideologies-and-regimes.md`](docs/design/ideologies-and-regimes.md) — ideologies, institutions, and forms of rule;
- [`docs/architecture.md`](docs/architecture.md) and [`docs/determinism.md`](docs/determinism.md) — technical contracts.

## Status

Foundation scaffold. No gameplay claim is made yet.

## License

Copyright © 2026 A.D.A.M author. All rights reserved. See [`LICENSE.md`](LICENSE.md).
