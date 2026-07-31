- Completed Step 100, “The Governing State Becomes Visible”: the authoritative government-program loop now drives a playable Bevy map, inspector, overlays, command desk, and political timeline with end-to-end replay/snapshot acceptance.

- The Bevy observatory now includes a graphical political timeline sourced directly from the shared authoritative chronicle and refreshed only after accepted commands.

- The Bevy observatory now includes a command-backed government-program desk for treasury/debt funding, execution, cancellation, and annual advance, with authoritative snapshot refresh and visible domain rejection.

- The Bevy observatory now provides keyboard-switchable confidence, population, and output overlays plus a canonical region inspector and selected-region highlight.

- Added the first Bevy observatory executable with a deterministic read-only world snapshot and confidence-colored regional map; `adam-core` remains presentation-independent.

- The annual chronicle now narrates government-program promises, funding, physical execution, labor, delay, regional winners and losers, and resulting polarization and legitimacy effects.

- Annual closure now applies government-program memory to regional satisfaction, population-weighted legitimacy, and polarization-sensitive elite cohesion exactly once.

- Government programs now retain persistent regional winners, underfulfilled promises, and explicit exclusions; broken promises carry a stronger negative political memory than making no promise.

- Government programs now use finite temporary regional labor, prevent same-year worker reuse across programs, pay realized spending into household wealth, and respect an independent infrastructure ceiling.

- Government programs may now require concrete goods: local public-reserve availability binds delivery, realized execution consumes physical stock, and missing materials remain visible as carryover and delay.

- Program execution now converts existing political-office influence into attributable bounded support or friction according to whether each actor’s home region is favored or excluded, without granting politics an artificial veto.

- Extended Step 100 through annual treasury/debt program appropriations, explicit zero funding and cancellation, plus administratively bounded regional execution with persistent carryover and delay evidence.

- Began Step 100, **The Governing State Becomes Visible**: persistent replayable government-program charters now separate political promises from later appropriation and physical delivery; the anniversary plan carries this causal arc into the first Bevy world client.

# Changelog

All notable project changes are documented here. The format follows Keep a Changelog and the project uses semantic versioning once public releases begin.

## [Unreleased]

### Added

- Added employment tenure and a three-month switching cooldown so new hires cannot be serially poached.
- Added qualification-specific regional labor evidence and skill-bucket wage pressure.
- Added household-funded three-month workforce training after persistent skill shortages.

- Added bounded voluntary employment switching: workers can accept materially better funded local offers without duplication, same-month chains, or replay ambiguity.

- Added bounded evidence-driven wage adaptation: persistent vacancies raise competitive bids, persistent unemployment restrains them, and replayable matches retain the signed causal adjustment.

- Added persistent regional labor-market evidence with residual unemployment and vacancy counts, funded offers, hires, average offered wages, bounded pressure streaks, fingerprints, typed events, and yearly chronicle attribution.

- Added an opt-in competitive labor market with finite worker allocation, target-derived vacancies, funded wage bidding, education floors, canonical matching, replayable employment events, and legacy-compatible content profiles.

- Added shortage-driven local firm entry: three months of residual survival shortage can produce a real founder-funded, staffed, governed one-batch startup when technology, prices, unemployed labor, and liquid capital are available.

- Persisted regional-good household import dependence from settled survival fills, with local/imported quantities, typed evidence, six-month high-reliance memory, bounded reserve-priority adaptation, and causal chronicle attribution.

- Evidence-driven regional reserve-priority adaptation that raises persistently uncovered configured targets, lowers repeatedly idle-spoiling targets, reuses ordinary political authority, and narrates the resulting institutional learning.

- Replayable regional-good public-reserve priorities that scale coverage targets, rank scarce-budget procurement, preserve explicit zero-priority exclusions, and surface the choice in typed evidence and the chronicle.

- Evidence-driven public-reserve doctrine that slowly raises coverage after recurring unbuffered shortages, expands procurement authority after repeated budget binding, and retrenches after persistent upkeep failure or idle spoilage.

- Monthly public-reserve maintenance with policy-configured carrying cost and baseline spoilage, proportional allocation of scarce treasury upkeep, additional neglect loss when funding fails, replayable evidence, and chronicle explanation.

- Evidence-rich reserve requirement reviews and yearly chronicle narration that explain coverage targets, opening stock, eligible local supply, treasury constraints, procurement spending, and in-kind releases before downstream harm.

- Configurable one-to-twelve-month public-reserve coverage targets with a shared monthly procurement ceiling expressed as a share of opening treasury.

- Evidence-driven monthly government reserve procurement that converts observed survival shortages into treasury- and inventory-bounded local purchases through the existing replayable command.

- Treasury-funded regional government reserves that buy real domestic firm inventory and release physical survival goods against residual shortages through the shared replayable command boundary.

- Persistent firm borrower credit histories for scheduled service, delinquency, successful resolutions, and defaults, with bounded history-based pricing.

- Realized lender track records for principal recovery, interest income, successful resolutions, defaults, and liquidation losses.

- Content schema v8 actor cash portfolios and regional financiers, plus a deterministic demo credit experiment with one repaid working-capital loan and one concentration-driven refusal.

- Autonomous domestic firm-credit search for viable one-month funding gaps, with lender liquidity reserves, portfolio concentration limits, distress- and exposure-priced competition, canonical borrower choice, monthly integration, replay, and chronicle narration.

- Evidence-underwritten firm credit offers with explicit annual interest, two-month expiry, cash-flow and collateral limits, corporate acceptance, concrete forecast financing, interest-first debt service, liquidation carry-through, replay, fingerprinting, and chronicle evidence.

- Scheduled firm principal repayment with 1-120 month terms, payroll seniority, partial and overdue balances, insolvency freeze, liquidation carry-through, typed events, replay, fingerprinting, and chronicle narration.

- Physical liquidation sales of installed capacity to funded compatible local successors, with payroll reserves, bounded integration, estate proceeds, and explicit retirement of unsold capacity.

- Actor-funded firm credit with secured and unsecured principal claims, worker-first liquidation priority, deterministic pro-rata creditor recovery, and explicit write-offs.

- Physical liquidation inventory auctions that transfer usable stock to solvent local producers, route proceeds through worker-priority claims, and journal each sale.

- Bounded twelve-month insolvency administration followed by replayable worker-priority liquidation, explicit claim and inventory write-offs, and residual owner distribution when no viable reorganization emerges.

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

### Changed

- Autonomous firm-credit applications are now batched and scarce lender headroom is allocated by deterministic risk-adjusted return instead of canonical borrower order.

- Autonomous lender rates and portfolio limits now adapt to successful repayment and realized loss history while preserving liquidity, concentration, and underwriting caps.

### Fixed

- Autonomous funding now separates recurring payroll from legacy wage arrears, reserves the first loan payment, removes inadequate offers, and closes debt-only matured defaults through insolvency.
## Steps 084-087 — retention, targeted training, sponsorship, and occupations

- Added forecast-solvent incumbent counteroffers with typed retention commands/events.
- Training now partitions one deterministic household instead of upgrading an aggregate cohort.
- Solvent local employers can fund tuition when households cannot, without guaranteed hiring.
- Added typed occupation skills layered over education; legacy recipes remain compatible.
- Advanced the simulation contract to version 74.
- Accepted ADRs 0132-0135 and passed 182 workspace tests plus Clippy/docs/release audit.
- Seed-1 50-year audit fingerprint: `18242524565653015855`; runtime: `99.6 ms/year`.
## Steps 088-090 — lifecycle, supplier loyalty, and public services

- Cohorts now advance through explicit age-band clocks; adulthood changes labor status and retirement closes active employment without moving financial stocks.
- Household cohorts remember dominant suppliers and accept at most a ten-percent loyalty premium before switching.
- Regional healthcare, infrastructure, and administrative capacity now evolve from an explicitly reported share of fiscal spending, with slow institutional absorption.
- Advanced the simulation contract from version 74 to 77 and accepted ADRs 0136-0138.

## Step 091 — resource-bounded internal migration

- Added deterministic household-scale migration from persistent unemployment toward persistent vacancies and stronger public services.
- Conserved population, household counts, income, liquid wealth, debt, health, experience, and skills across cohort partitioning.
- Added typed migration evidence and advanced the simulation contract to version 78.

## Step 092 — housing-constrained paid migration

- Added persistent regional dwelling capacity and baseline housing costs.
- Made projected housing pressure constrain migration and price relocation.
- Transferred relocation fees from migrant wealth to destination treasury without creating money.
- Advanced the simulation contract to version 79 and accepted ADR 0140.

## Step 093 — pressure-funded delayed housing construction

- Added treasury-funded public housing projects triggered by persistent physical occupancy pressure.
- Added a two-year construction delay, bounded capacity increments, committed cost, and public housing capital.
- Added typed project start/completion evidence and advanced the simulation contract to version 80.

## Step 094 — settlement arc chronicle

- Added named yearly narration for household migration, housing pressure, and relocation fees.
- Added separate public housing authorization and completion narration with importance ranking.
- Kept simulation rules, version 80, and fingerprint semantics unchanged.

## Step 095 — material social pressure and legitimacy

- Added population-weighted regional social pressure from unemployment duration, livelihood distress, and public-service shortfall.
- Connected bounded material pressure to annual country legitimacy alongside existing growth, fiscal, and crisis signals.
- Added persistent state, stable fingerprinting, typed evidence, explanatory chronicle narration, and focused weighting tests.
- Advanced the simulation contract to version 81 and accepted ADR 0143.

## Step 096 — persistent regional interests and fiscal incidence

- Added persistent regional employment, household-security, public-service, and stability priorities with switching hysteresis.
- Added regional satisfaction and physically attributable sales-tax/service-allocation winners and losers.
- Added authoritative fingerprinted state, typed evidence, chronicle narration, and ADR 0144.

## Step 097 — delayed regional confidence and legitimacy

- Added population-weighted prior-year regional confidence as a bounded national legitimacy signal.
- Kept current policy outcomes delayed until the following political closure to preserve causal memory.
- Added typed evidence, chronicle narration, deterministic tests, ADR 0145, and simulation version 83.

## Step 098 — unrestricted authorized service allocation

- Added replayable political-office commands for exact regional public-service budget shares.
- Explicitly allowed zero-funded regions and 100% concentration in one region under shared player/AI legality.
- Added prudent need-sensitive autonomous allocation as behavior rather than an engine cap.
- Added exact integer budget conservation, regional delivery effects, typed evidence, chronicle narration, ADR 0146, and simulation version 84.

## Step 099 — power-network-biased autonomous service allocation

- Connected political-office holders, actor home regions, and established influence edges to autonomous regional service allocation.
- Kept all political weighting in AI decision policy; explicit authorized player allocations bypass it completely.
- Added typed actor-to-region influence evidence, actor-attributed chronicle narration, ADR 0147, and simulation version 85.
