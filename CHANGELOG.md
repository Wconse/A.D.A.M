# Changelog

All notable project changes are documented here. The format follows Keep a Changelog and the project uses semantic versioning once public releases begin.

## [Unreleased]

### Added

- Rust workspace foundation with headless `adam-core` and console `adam-cli`.
- Deterministic clock, typed IDs, RNG streams, event log, and state fingerprint.
- Architecture, determinism contract, ADRs, quality scripts, and CI.
- Versioned TOML world loader with strict validation and a dedicated content adapter.
- Canonical fixed-point value types, 32-bit typed IDs, regions, actors, power nodes, and influence graph.
- Atomic causal yearly loop for demography, output, fiscal closure, legitimacy, and elite cohesion.
- World content schema v2 and simulation rules version 3.
- Documented the full simulation plan, hybrid reputation/propaganda model, ideology/regime framework, and live progress roadmap.
- Added world schema v3 household cohorts and deterministic regional population accounting.
- Added world schema v4 goods, need hierarchies, regional prices, and household demand planning.
- Added deterministic firm production planning constrained by labor, capital capacity, and intermediate inventories.
- Added deterministic mod manifests, layered content patches, provenance history, and canonical merged-content fingerprints.
- Added save/mod-set compatibility metadata and multi-layer registry conflict reports.
- Added complete binary world snapshot round-trips with deterministic continuation tests.
