# A.D.A.M Development Roadmap

Status legend: ✅ complete and verified · 🟡 working scaffold · ⬜ planned · 🔬 research/measurement gate

## Stage 0 — Console Chronicle

### 0.1 Repository and deterministic foundation — ✅

- ✅ Rust workspace, pinned toolchain, CI, quality scripts, documentation policy.
- ✅ Headless `adam-core` and console `adam-cli`.
- ✅ Typed IDs, fixed-point values, simulation date, isolated RNG streams.
- ✅ Ordered world state, typed append-only event log, stable fingerprint.
- ✅ Versioned TOML content adapter and strict cross-reference validation.
- ✅ Reproducible 100k-actor/100k-node/100k-edge construction probe.

### 0.2 Minimal social world — ✅

- ✅ Countries, regions, named actors, power nodes, holders, influence edges.
- ✅ Data-driven initial fiscal and political indicators.
- 🟡 Scalar influence edge remains only a deterministic scaffold; it is not the accepted power model.

### 0.3 Causal annual harness — 🟡

- ✅ Atomic year planning and commit.
- ✅ Isolated demographic, economic, and political random streams.
- ✅ Initial links among population, output, fiscal balance, debt, legitimacy, and elite cohesion.
- ✅ Same-seed replay and direct causal tests.
- 🟡 Current aggregate growth equations are a test harness and will be replaced by cohort and market systems.

### 0.4 Demographic cohorts and households — ⬜ NEXT

- ✅ Data-driven household cohort identity, dimensions, and regional population accounting.
- ✅ Deterministic largest-remainder rescaling while the aggregate demographic harness remains.
- ⬜ Adaptive cohort split/merge rules with conserved stocks.
- ⬜ Age progression, births, deaths, household formation, health.
- ⬜ Education and qualification pipelines.
- 🟡 Income, liquid wealth, debt, and household counts are represented; monthly budget flows and housing remain planned.
- 🔬 Cohort compression accuracy and scale benchmark.

### 0.5 Needs and consumption — ⬜

- ✅ Hierarchical survival, participation, development, and discretionary need tiers.
- ⬜ Substitution between goods, quality, habit, culture, and time cost.
- ✅ Monthly demand intentions from cohort population, household counts, income, debt, and regional prices.
- 🟡 Taxes, transfers, savings, wealth drawdown, and intra-household allocation remain planned.
- ⬜ Unmet needs, quality, habits, time cost, and cultural preference.
- ✅ Initial eight-good test taxonomy and four data-driven consumption profiles.
- ⬜ Validate the goods taxonomy against production and substitution behavior before expansion.

### 0.6 Firms and production — ⬜

- 🟡 Production units now have typed recipes, labor, capital capacity, cash, inventories, outputs, and intermediate inputs; ownership and technology remain planned.
- 🟡 Deterministic physical production planning and inventories are implemented; execution, expectations, investment, sales, and failure remain planned.
- ⬜ Qualification constraints and management capacity.

### 0.7 Playable business, ownership, and brands — ⬜

- ✅ Business, brand, governance, and logistics architecture documented.
- 🟡 Core ownership stake and firm-policy value objects implemented.
- ⬜ Legal forms, share classes, voting control, boards, executives, and delegated authority.
- ⬜ Player/AI firm commands: sourcing, pricing, quality, inventory, wages, finance, investment, dividends.
- ⬜ Product offers and audience-specific brand beliefs: awareness, trust, quality, familiarity, status, service reliability.
- ⬜ Advertising through claims, channels, targeting, evidence, repetition, and backlash—not direct popularity points.
- ⬜ Accounting, credit covenants, insolvency, takeovers, nationalization, inheritance, and succession.

### 0.8 Spatial markets and logistics — ⬜

- ⬜ Regional order matching, warehouses, and inventories.
- ⬜ Route graph, shipment orders, fleet/terminal capacity, cost, delay, reliability, spoilage, and congestion.
- ⬜ Logistics companies as ordinary playable firms with transport and warehousing products.
- ⬜ Own-account shipping, spot freight, and long-term carrier contracts.
- ⬜ Price formation from fill rate, cost, stocks, expectations, and market power.
- ⬜ Transport capacity, delay, cost, borders, tariffs, sanctions, spoilage.
- 🔬 Market convergence, shortage behavior, and performance tests.

### 0.9 Labor, migration, and human capital — ⬜

- ⬜ Job matching, wages, unemployment, bargaining, working conditions.
- ⬜ Internal and international migration based on expected life outcomes.
- ⬜ Skill formation, mismatch, automation, and brain drain.

### 0.10 Government, banking, and finance — ⬜

- ⬜ Taxes, budgets, procurement, debt, assets, and implementation capacity.
- ⬜ Banks, credit, collateral, liquidity, insolvency, and guarantees.
- ⬜ Inflation emerging from goods, money, production, and expectations.

### 0.11 Institutions, ideologies, and political process — ⬜

- ✅ Design framework documented.
- ⬜ Multidimensional private beliefs, public platforms, salience, compromise.
- ⬜ Data-driven ideology traditions and classifications.
- ⬜ De jure/de facto institutions, offices, veto points, succession.
- ⬜ Coalition bargaining and policy implementation.
- ⬜ Elections, protest, repression, coups, reform, and state decay.

### 0.12 Information, reputation, media, and propaganda — ⬜

- ✅ Hybrid audience-traits plus event-memory design selected.
- ✅ Absurd propaganda, conformity, private/public belief split documented.
- ⬜ Claims, evidence, frames, narratives, channels, source credibility.
- ⬜ Awareness, attention, confidence, trait beliefs, issue beliefs.
- ⬜ Rumor mutation, censorship, counter-messaging, polling uncertainty.
- ⬜ Material events feeding information and political behavior.

### 0.13 Contextual power network — ⬜

- ⬜ Replace universal influence weights with typed relationships.
- ⬜ Formal authority, organizational control, assets, access, trust, obligation, fear.
- ⬜ Actor knowledge and uncertainty about the network.
- ⬜ Action-specific coalition and veto resolution.

### 0.14 Diplomacy, trade, crisis, and war — ⬜

- ⬜ Bilateral beliefs, interests, trade exposure, commitments, and tension.
- ⬜ Mobilization, sanctions, negotiation, escalation, and conflict.
- ⬜ Minimal unified combat model for auto-resolution; no tactical UI yet.

### 0.15 Chronicle experiment — ⬜

- ⬜ Importance scoring and causal-chain extraction.
- ⬜ Competing contemporary narratives versus retrospective history.
- ⬜ Chapters, biographies, turning points, economic and demographic series.
- ⬜ Fifty-year seed panel and human evaluation rubric.
- 🔬 Stage 0 acceptance gate: the chronicle is coherent, surprising, causal, and memorable.

## Later stages

- Stage 1: Bevy 2D strategic map and observation UI.
- Stage 2: playable mortal lives and power ladders.
- Stage 3: expanded economy and money-based life path.
- Stage 4: unified 2D tactical battle participation.
- Stage 5: chronicle maps, timelines, biographies, charts, HTML/PDF textbook.
- Stage 6: UX, performance, content, and public demo.

## Progress log

### 2026-07-21

- Created deterministic Rust workspace and repository policy.
- Added versioned world content, actors, regions, power nodes, and event archive.
- Fixed canonical numeric representations and 32-bit typed identity contract.
- Measured 100k actors + 100k nodes + 100k edges construction at 82 ms on the initial machine.
- Added and verified the atomic annual causal harness.
- Selected hybrid reputation model B+C.
- Documented material/institutional/social/information layers, propaganda, ideology, regimes, and depth criteria.
- Added schema-v3 household cohorts with age, household, education, employment, income, wealth, debt, and exact regional accounting.
- Added schema-v4 goods, need profiles, regional prices, and budget-constrained monthly household demand intentions.
- Added the core firm/recipe model and physical monthly production constraints for labor, capacity, and intermediate inputs.
- Documented playable firm governance, consumer brand choice, advertising, route logistics, and logistics companies.

## Progress update rule

Every accepted slice updates this file with status, tests, measurements, known approximations, and the next gate. New ideas are added only after passing the depth rule in `docs/design/simulation-plan.md`.
