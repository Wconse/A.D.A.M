# Changelog

All notable project changes are documented here. The format follows Keep a Changelog and the project uses semantic versioning once public releases begin.

## [Unreleased]

### Added

- Policy-driven autonomous firm reorganization that derives and executes a minimum viable reopening plan from worker claims, payroll, available labor, owner liquidity, and the authorized reinvestment rate.

- Replayable funded firm reorganization that pays preserved wage claims and requires a full month of payroll liquidity before reopening.

- Firm insolvency administration after prolonged zero-workforce distress, freezing ordinary operations while preserving cash, inventory, and worker claims.

- Evidence-driven owner recapitalization after three consecutive months of wage arrears, preserving worker claims and journaling the transfer.
- Governed 25% distress downsizing when owners lack liquidity, with terminated workers retaining payable wage claims.
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
- Added authoritative firm ownership, voting authorization, persisted policies, and replayable management commands.
- Added board-funded investment projects that convert committed cash into constructed production capacity over time.
- Added authoritative multi-leg inventory shipments with shared route capacity and firm stock transfer.
- Added carrier-owned routes and deterministic freight payments between firms.
- Added deterministic mod manifests, layered content patches, provenance history, and canonical merged-content fingerprints.
- Added save/mod-set compatibility metadata and multi-layer registry conflict reports.
- Added complete binary world snapshot round-trips with deterministic continuation tests.
