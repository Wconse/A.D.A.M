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
- 🟡 Deterministic production execution, realized sales/payroll accounting, and explicit management cash-flow expectations are implemented; technology, broader investment behavior, and failure remain planned.
- ⬜ Qualification constraints and management capacity.

### 0.7 Playable business, ownership, and brands — ⬜

- ✅ Business, brand, governance, and logistics architecture documented.
- ✅ Ownership stakes are authoritative world state with separate economic/voting rights, aggregate 100% validation, majority-control checks, stored firm policies, and replayable policy commands.
- 🟡 Corporate appointments now represent board, CEO, operations, and marketing roles; CEO delegated policy authority works, while legal forms, share classes, board votes, and narrow role scopes remain.
- 🟡 Action-specific authority separates all major domains; serialized marketing-budget and inventory-buffer commands now enforce narrow role scopes, while investment/dividend/appointment commands remain.
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

- 🟡 Concrete employment agreements, firm-funded payroll, wage arrears, layoffs, production labor constraints, and advisory expectation-aware staffing are implemented; job matching, bargaining, and working conditions remain planned.
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

## Cross-cutting modding platform — 🟡

- ✅ Strict mod manifests, namespaced identities, dependencies, load hints, deterministic ordering, and diagnostics foundation.
- ✅ Generic layered namespaced registry with provenance and explicit set/remove/append patch operations.
- 🟡 Generic post-merge typed decoding and provenance diagnostics are implemented; strict goods/recipe schemas and cross-registry namespaced reference validation are now working, with more content types remaining.
- 🟡 Per-entry change provenance and canonical merged-content fingerprint are implemented; save/replay embedding, conflict reports, compatibility ranges, and migrations remain.
- 🟡 `adam-validate-mod` now validates multiple folders and deterministic dependency order plus manifests, goods, recipes, files, and cross-references; merged patches and source spans remain.
- ⬜ Declarative conditions, formulas, effects, and AI weights.
- ⬜ Sandboxed deterministic WebAssembly extension API after the core stabilizes.

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
- Added authoritative ownership, majority-vote authorization, firm policies, and shared player/AI replay commands.
- Added persisted corporate appointments and CEO delegated authority through the same command resolver.
- Replaced universal corporate control with an action-specific owner/executive/manager/board authority matrix.
- Added replayable marketing and inventory commands with field-level immutable policy updates.
- Added deterministic board resolutions with director eligibility, quorum, voting thresholds, and closure.
- Integrated typed board appointment/removal mandates, one-time execution, events, commands, save/replay, and fingerprinting.
- Added one-time dividend and investment mandates, cash constraints, deterministic owner payouts, actor cash, committed-investment balances, events, save/replay, and fingerprinting.
- Added funded regional investment projects with monthly construction progress, deterministic spending, completion events, capacity creation, commands, save/replay, and full fingerprinting.
- Added typed directed logistics routes and deterministic shipment planning constrained by direction, capacity, cost, time, reliability, and budget.
- Added deterministic multi-leg routing with cycle avoidance, leg limits, per-leg capacity, total cost, transit time, and stable tie-breaking.
- Added atomic shared-capacity reservations and a serializable planned/in-transit/delivered shipment lifecycle with release on delivery.
- Integrated routes, capacity, shipment commands/events, and atomic firm inventory debit/credit into authoritative World; added end-to-end delivery test.
- Added carrier-owned routes and atomic fixed-point freight payments from shippers to carriers with liquidity checks and shipment cost fingerprinting.
- Added long-term freight contract values, reserved capacity, term/status, discounts, route fuel/labor/maintenance costs, and deterministic carrier margin evaluation.
- Integrated freight contracts and route operating-cost profiles into World with validation, commands, lifecycle events, save/replay, expiry, and fingerprinting.
- Applied active contract discounts and route operating costs to shipment routing and atomic shipper/carrier cash settlement.
- Added separate guaranteed contract and residual spot capacity ledgers with protected reservations and releases.
- Integrated contract/spot pool selection, atomic reservation, shipment provenance, delivery release, save/replay, and fingerprinting into authoritative World shipments.
- Added renewable monthly contract/spot capacity periods with isolated reservations, exact release, and deterministic pruning.
- Added deterministic per-leg shipment progression with current-route state, exact transit-day consumption, and multi-leg completion reporting.
- Migrated authoritative inventory shipments to per-leg progression with immediate general and contract/spot capacity release on each completed route.
- Added explicit intermodal terminal-handling phases and causal route/transfer completion transitions between shipment legs.
- Added authoritative firm-operated regional terminals with handling duration, storage tariffs, shared throughput reservation, save/replay, and fingerprinting.
- Added deterministic FIFO terminal queues with unique shipments, head-of-line blocking, and shared-capacity admission.
- Integrated terminal queues into World with shipment validation, FIFO admission commands, events, save/replay, and fingerprinting.
- Added an explicit waiting-for-terminal phase so queue delay cannot consume handling time before terminal admission.
- Migrated authoritative shipments to intermodal phases with deterministic terminal selection, automatic queueing, FIFO admission, handling, and throughput release.
- Documented playable firm governance, consumer brand choice, advertising, route logistics, and logistics companies.
- Added strict namespaced mod manifests and deterministic dependency/load-order resolution.
- Added the layered content registry with explicit non-silent patch operations and provenance.
- Added per-entry patch history and an insertion-order-independent canonical fingerprint for merged mod content.
- Added strict post-merge typed decoding and provenance-rich validation reports.
- Added typed goods/recipe mod schemas and accumulated cross-registry reference validation.
- Added folder-level `adam-validate-mod` for manifests, files, schemas, and cross-references.
- Extended validation to multiple mod folders with deterministic dependency load order.
- Added physical multi-mod merge of goods/recipes plus disk patch files and canonical post-merge fingerprint.
- Added deterministic exact package fingerprints across every TOML file in the selected mod set.
- Dependency-ordered definitions and disk patches are now merged by the CLI with a canonical final fingerprint; the example mod exercises the full path.
- Added multi-layer conflict reports and explicit save/mod-set compatibility diagnostics.
- Added strict versioned save envelopes, exact pre-payload compatibility checks, round-trip tests, and migration-chain contracts.
- Added complete binary world snapshots and verified save/load continuation against uninterrupted history.
- Added checksummed atomic save files, `.bak` recovery, corruption tests, and a verification CLI.
- Added validated automatic fallback to `.bak`, explicit recovery source, dual-failure diagnostics, and CLI reporting.
- Added a shared replayable command boundary and verified `20-year snapshot + 30-year command tail` against uninterrupted 50-year history.
- Enforced that every definition belongs to the declaring mod namespace, preventing namespace hijacking.

## Progress update rule

Every accepted slice updates this file with status, tests, measurements, known approximations, and the next gate. New ideas are added only after passing the depth rule in `docs/design/simulation-plan.md`.

- Added authoritative monthly production execution with input consumption, output inventory, replayable command, and typed events.

- Added deterministic local market clearing with physical supply, budgets, price-ranked sellers, canonical buyers, fills, and explicit unmet demand.

- Reoriented social causality toward concrete cohort experience durations and typed historical events rather than generic crisis debuffs.

- Added replayable firm cash-flow expectations and expectation-aware advisory staffing, with recovery and deterioration tests; forecasts remain knowledge state and cannot move resources directly. Next gate: derive forecasts from observed sales, input prices, and financing offers rather than externally supplied values.

- Added replayable operational forecast derivation from realized sales, physically feasible recipe throughput, and observed regional input prices. Financing remains zero until concrete offers exist; current values persist across the forecast horizon as an explicit myopic approximation. Next gate: finite sales/price memory and concrete financing offers.

- Added a replayable twelve-observation firm operating history containing realized sales, actual production batches, and observed recipe-input prices. Forecasts now use deterministic integer averages when history exists; save/load and fingerprints include the bounded history. Lost sales and financing offers remain the next concrete information sources.

- Added canonical seller-side market outcomes for every offer: offered, sold, unsold, price, and remaining regional unmet demand. Settlement validates outcomes against fills, and firms retain the evidence in operating history without turning it into a generic lost-sales score. Next gate: use repeated stockout exposure in production and inventory decisions.

- Added advisory production adjustment from bounded market history, expected operating cash margin, and current physical constraints. Stockout evidence may raise the advised output only within the observed demand ceiling and feasible labor/capacity/input limit; no production is changed automatically. Next gate: an explicit replayable production-target decision and monthly execution against it.

- Added authorized replayable firm production targets. Operations-scoped actors choose a monthly batch target; execution now applies the minimum of that target, installed capacity, active-agreement labor, and available inputs. Targets survive save/load, replay, and fingerprinting, while advisory feasibility remains independent of the current target. Next gate: connect planned output to concrete seller offer formation and inventory policy.

- Added policy-driven seller offer formation. Firms retain an inventory buffer derived from bounded realized sales, then offer remaining output inventory at the regional reference price plus their authorized markup. Offer planning is non-mutating and remains separate from market clearing. Next gate: a replayable monthly commercial cycle that explicitly sequences production, offer formation, demand, clearing, settlement, observation capture, and reset.

- Added an atomic replayable monthly commercial cycle: production, policy-driven offers, household demand, clearing, settlement, firm observation capture, and account reset. Failures leave state and events unchanged, and duplicate execution for the same simulation date is rejected. Next gate: explicit monthly time advancement and integration of payroll and household cashflows before demand formation.

- Added deterministic calendar-month advancement and an atomic monthly economic cycle sequencing payroll, household cashflows, commerce, cohort experience, social-state derivation, and time. Monthly stages reject duplicate execution, and household demand now spends concrete post-cashflow liquid wealth instead of recreating annual income. Next gate: run long histories through twelve monthly cycles per year and reconcile annual fiscal/political closure with monthly state.

- Added monthly economic years: twelve atomic monthly cycles followed by one annual demographic, output, fiscal, and political closure using the explicitly closed year for deterministic RNG. A fifty-year economic test now covers 600 monthly cycles, 50 annual closures, replay, and twelve-observation bounded firm memory. Next gate: make the content blueprint capable of supplying complete firms, ownership, appointments, policies, and production targets so the console chronicle can use the monthly path by default.

- Closed the first survival-consequence loop: settled survival fulfillment now changes persistent cohort functional capacity, age-sensitive excess mortality, fallback labor income, effective contracted production labor, employment counts, households, and exact regional population. A full monthly market-to-mortality test prevents persistent severe deprivation from remaining an observational counter. Current response coefficients are explicit Stage 0 approximations. Next gate: add concrete coping and autonomous institutional response so shortage can cause substitution, relief, migration, mobilization, or policy rather than only physical decline.

- Closed the observed firm-management loop: monthly market outcomes feed bounded history, a three-month operational forecast, production advice, deterministic selection of an authorized operations manager/CEO/majority owner, and the ordinary replayable production-target command for the next month. A no-sales test now contracts a managed target from five batches to zero instead of accumulating unsold output indefinitely. Next gate: close household coping and government relief/mobilization loops before expanding product or institution detail.

- Closed the first institutional relief loop: market clearing now identifies survival goods that remained physically available but unaffordable, records a cohort affordability gap, selects an actual political-office holder, and applies a replayable treasury-funded transfer for next-month purchases. A two-month test proves relief buys the previously unaffordable food and restores full survival fulfillment; a no-supply test proves cash is not transferred into a physical shortage. Next gate: add household borrowing/substitution and make government response a constrained choice among relief, procurement, rationing, borrowing, and inaction.

- Added bounded household survival borrowing before market formation and public relief. Cohorts now use income and savings first, borrow only the remaining survival cost up to twice annual income, service the debt through ordinary cashflow, and expose only the residual affordability gap to government. A two-month test proves full first-month consumption followed by a binding debt ceiling and 30% survival fulfillment. Next gate: make political response choose among treasury relief, public borrowing, procurement/rationing, and inaction, with legitimacy and debt consequences.

- Made emergency response an authoritative political policy: political-office holders can select treasury-only relief, bounded public borrowing, or deliberate inaction through replayable commands. Public borrowing is capped at twice aggregate annual output minus existing debt and moves through treasury before transfer. Identical affordability crises now branch into funded debt relief or zero-consumption inaction. Next gate: represent physical-shortage response through procurement/rationing and connect sustained inaction, deaths, and debt issuance to legitimacy and organized political pressure.
