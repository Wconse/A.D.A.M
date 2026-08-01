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

## Near-term strategic direction — societal adaptation

Pause further credit depth. The next Stage 0 arc is material-crisis adaptation: physical government procurement and reserves, endogenous firm entry, cohort migration, labor matching, survival substitution, material political choice, information narratives, then causal chronicle arcs. The acceptance scenario is a trade-dependent region that can recover, substitute, shrink through migration, or collapse for concrete resource and institutional reasons.

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

- Added proportional survival rationing as the first physical-shortage policy. When local survival offers cannot cover requested need, policy can replace canonical first-buyer priority with deterministic largest-remainder quotas. Quotas still require payment, rationed-away need remains visible to health and firm observations, and a replayed test splits one food unit into two equal half-unit fills with 50% fulfillment for both cohorts. Next gate: add treasury-funded procurement/reserve release that moves real inventory, then connect relief, rationing, inaction, debt, and deaths to legitimacy attribution.

- Closed the executable path from configured world through the full monthly economic simulation into a deterministic yearly console chronicle. The CLI now runs `advance_economic_years` instead of the legacy annual shortcut, and the chronicle summarizes event-backed survival fulfillment, deaths, coping, relief, rationing, production, trade, and politics. A real 50-year example run exposed the next blocking gap: because content has no firms/recipes/employment/ownership/appointments/targets, all three regions collapse to 100 total people while macro output still grows. Freeze new policy depth; next gate is a minimal producing content blueprint and another 50-year narrative audit.

- Added schema-v5 producing content: recipes, firms, employment, ownership, appointments, policies, and initial targets now build an executable survival economy in all three example regions. Partial contracts reduce fallback income only for their contracted worker share, fixing the artificial loss of whole-cohort income. The seed-47 50-year audit now finishes with 36,843,754 people, active production and trade, and about 98.4% minimum survival fulfillment instead of collapsing to 100 people. Next gate: derive regional output and fiscal capacity from micro production, payroll, and trade so the legacy macro harness cannot diverge from material activity.

- Closed macroeconomic output from the monthly market path: economic years now sum settled local trade into regional annual output, then use that same material base for fiscal closure, debt, and politics. A no-trade test produces zero output and zero fiscal activity instead of seeded phantom growth. The calibrated seed-47 fifty-year audit ends with active production and trade, about 31.1 million people, and regional outputs near 5.3–6.1 trillion rather than independently growing toward 100 trillion. Next gate: attribute taxes, procurement, inventory investment, and intermediate consumption without double counting.

### Realized fiscal revenue gate

Completed vertical slice:

```text
realized firm sales
→ liquidity-bounded sales-tax payment
→ country revenue
→ treasury / public debt
→ political closure
```

The next accounting gate is to separate final consumption, intermediate consumption, inventory investment, and government procurement so regional value added does not double-count turnover.

### Final consumption and inventory gate

Completed vertical slice:

```text
settled household consumption
+ valued firm inventory change
→ expenditure-side regional output
→ fiscal spending / debt / politics
→ causal chronicle
```

The next gate is explicit intermediate-goods procurement between firms. B2B turnover must move cash and inventories while regional output continues to count only final demand plus inventory investment, preventing double counting.

### Intermediate procurement gate

Completed vertical slice:

```text
firm input requirements -> B2B offers -> procurement fills moving cash and inventories -> final-demand-only regional output -> conserved money -> replayable year
```

Verified end-to-end by `crates/adam-core/tests/intermediate_procurement.rs`: a farm -> bakery -> households chain proves twelve monthly B2B grain trades move firm cash and inventories, measured regional output equals settled household final consumption plus valued inventory change and excludes intermediate turnover, money is conserved across firms, households, and paid sales taxes, and the economic year replays identically through the command boundary.

Repository debt discovered while closing this gate: the committed tree referenced the `adam-cli` and `adam-content` crates without any sources, although earlier progress entries describe a working console chronicle and schema-v5 producing content. Placeholder targets were restored so the workspace builds again. The next gate is to rebuild the Stage 0 console chronicle runner and producing content, and to settle two accounting questions: record B2B market outcomes at the actual transaction price instead of the regional reference price, and decide whether the sales tax should cascade over intermediate turnover or apply only to final sales.
### Stage 0 console chronicle gate

Completed vertical slice:

```text
embedded demo content -> 50 deterministic economic years -> readable yearly console chronicle -> seed comparison -> graceful regional extinction
```

`adam-cli` is a real runner again (`--seed N --years N`, defaults 1/50): it advances whole economic years and renders a yearly chronicle from the domain event log (household purchases, firm-to-firm trade, sales taxes, measured regional output, population, fiscal closure, politics). `adam-content` provides the deterministic embedded two-region demo scenario: Northreach carries a thin bakery cash buffer and decays into extinction mid-century under the sales-tax monetary drain, while Southvale is buffered to survive the full horizon. Verified on a real run: equal seeds produce byte-identical 50-year chronicles, different seeds diverge, and the collapse of Northreach unfolds causally (firm cash exhaustion -> wage arrears -> deprivation -> mortality -> extinction) while the country's legitimacy erodes from about 8900 bp to about 4500 bp.

Core fix shipped with this slice: an extinct region (all-zero cohort ledger with a zero rescale target) now closes the annual demographic closure cleanly instead of erroring, covered by a dedicated regression test.

Author decision recorded: the sales tax must stop cascading over intermediate B2B turnover and apply to final household sales only. That accounting separation (taxable final revenue vs. total revenue as decision evidence for firms) is the next gate. After it, rebuild the lost TOML content pipeline (schema v5) on top of the embedded demo scenario.
### Final-sales-only sales tax gate

Completed vertical slice:

```text
separate taxable final revenue from total revenue -> tax final household sales only -> untaxed intermediate B2B turnover -> re-verified 50-year deterministic chronicle
```

The annual sales tax (20%) is now levied exclusively on final household sales. Firm monthly accounts and operating observations carry both figures: `final_sales_revenue` (the taxable base, filled only by household market settlement) and `sales_revenue` (total revenue including intermediate firm-to-firm turnover), which stays untaxed decision evidence for firm management. Both fields participate in the stable world fingerprint; SIMULATION_VERSION bumped to 27. Verified on a real run: year-one taxes dropped from 66,000 (turnover cascade) to 52,800 = 20% x 264,000 of household spending; equal seeds still produce byte-identical 50-year chronicles and different seeds diverge; the demo narrative is preserved (Northreach still decays into extinction mid-century, Southvale persists).

Observed modeling debt for a future slice: with the smaller tax base the country runs a permanent deficit and public debt grows without bound or consequence (about 544,000 by year 50) - there is no debt service and no fiscal adjustment yet. A debt-and-spending feedback belongs in a later politics/fiscal slice.

Next gate: rebuild the TOML content pipeline (schema v5) on top of the embedded demo scenario.
### TOML content pipeline (schema v5)

Completed vertical slice:

```text
embedded Rust scenario constants -> strict serde schema v5 -> TOML scenario asset -> deterministic World registration
```

`adam-content` now exposes `world_from_toml_str(seed, document)` and keeps `demo_world(seed)` as the stable Stage 0 entry point. The demo scenario lives in `crates/adam-content/assets/demo.toml`; unknown fields, malformed TOML, unsupported enum values, and schema versions other than 5 are rejected. The loader preserves deterministic registration order and converts content into the same authoritative `adam-core` domain model used by commands and simulation.

Acceptance evidence: formatting and the full quality gate pass; the TOML-loaded seed-1 world produces the exact established 50-year fingerprint `10095822769523244874`, proving that extracting content from Rust changed neither world state, event order, nor chronicle behavior.

Next narrow debt candidate: make firm-to-firm procurement observations carry the actual transaction price rather than the regional reference price, while preserving deterministic replay and money conservation.
### Actual procurement trade prices gate

Completed vertical slice:

```text
settled B2B fill at the offer price -> per-(seller, good) trade price -> seller monthly market outcome -> bounded operating history -> unchanged demo fingerprint
```

`execute_monthly_firm_procurement` now records the seller-side `MarketOfferOutcome.unit_price` from the actually settled offer price instead of looking the value up in the regional reference table. The price is captured per (seller, good) at fill time in a local deterministic map, so no new world state, save schema, or fingerprint format was introduced. A new gate test in `crates/adam-core/tests/intermediate_procurement.rs` gives the farm a 20% markup so the trade price (6) diverges from the reference price (5) and proves both that the B2B fill settles at 6 and that the seller's captured monthly observation reports 6, not 5.

Acceptance evidence: formatting and the full quality gate pass (88 tests across all crates); the seed-1 50-year fingerprint is unchanged at `10095822769523244874`, exactly as required - all demo markups are zero, so trade prices coincide with reference prices by construction and the chronicle must not move.

Remaining sibling debt for future narrow slices: buyer-side `input_prices` in `capture_monthly_firm_observation` still come from the regional reference table (an honest buyer-side record needs a monthly procurement-fills buffer in world state, i.e. a save-schema and fingerprint change), and `plan_regional_inventory_change` still values inventory investment at reference prices.

### Actual buyer-side procurement input prices gate

Completed vertical slice:

```text
settled B2B fills -> per-(buyer, good) monthly purchase buffer in world state -> capture_monthly_firm_observation derives each input price from actual monthly spend / quantity -> reference-price fallback only for goods not purchased this month -> monthly reset -> SIMULATION_VERSION 28 -> new demo fingerprint baseline
```

`capture_monthly_firm_observation` now reports the actual average price the buyer paid for each recipe input during the month instead of the regional reference price. Procurement aggregates settled fills into a new world-state buffer `monthly_firm_procurement_purchases` keyed by `(buyer, good)` with total quantity and total spend; the buffer participates in the stable fingerprint, is written only after the atomic firm-state swap, and is cleared in `reset_monthly_firm_accounts`, so it lives exactly one month. Because observed input prices feed `observed_operating_baseline` and firm expectations, management decisions now react to the prices firms actually paid. A new gate test gives the farm a 20% markup so the bakery pays 6 per grain unit while the reference price stays 5, and proves the bakery's captured observation reports 6, not 5.

Acceptance evidence: formatting and the full quality gate pass (89 tests across all crates); the seed-1 50-year chronicle body is byte-identical to the pre-patch run once the `final fingerprint:` line is excluded (all demo markups are zero, so actual prices coincide with reference prices by construction and the history must not move). The fingerprint baseline changes by design because `SIMULATION_VERSION` (27 -> 28) and the new world-state buffer participate in the hash: the old baseline `10095822769523244874` is retired and the new seed-1 50-year baseline is `6474822005825804939`.

Next gate: value planned regional inventory changes at observed prices - `plan_regional_inventory_change` still prices inventory investment from the regional reference table.

### Observed-price inventory valuation gate

Completed vertical slice:

```text
bounded firm observation history (exactly the closed year) -> latest observed per-(firm, good) transaction price: buyer-side input price first, then seller-side settled market outcome -> plan_regional_inventory_change values annual inventory deltas at observed prices -> regional reference table only as a fallback for goods with no observed trades -> no schema change -> SIMULATION_VERSION stays 28 -> fingerprint baseline unchanged
```

`plan_regional_inventory_change` now values annual inventory investment at the prices firms actually transacted at instead of the regional reference table. A private valuation helper scans the firm's bounded operating history and prefers the most recent buyer-side observed input price, then the most recent seller-side settled market-outcome price for the good; the regional reference price remains only as a fallback. Together with the seller-side (actual trade prices) and buyer-side (actual procurement prices) gates this closes the loop: both components of measured regional output - final consumption and inventory change - now derive from actual transactions.

Acceptance evidence: formatting and the full quality gate pass (90 tests across all crates); a new gate test gives the farm a 20% markup and proves the annual inventory change is valued at the observed transaction price of 6 per grain unit instead of the reference price 5. Because no world-state schema changed and all demo markups are zero, the seed-1 50-year fingerprint must stay at `6474822005825804939` - and it does.

Next gate: charge annual interest on public debt - `close_budget` capitalizes deficits into debt, but the debt stock is causally inert: a country carrying half a million of debt closes the same budget as a debt-free one.

### Public debt interest gate

Completed vertical slice:

```text
outstanding public debt -> annual 300 bps interest charge added to fiscal spending in both fiscal planners -> worse fiscal balance -> close_budget capitalizes the unfunded deficit back into debt -> compounding debt burden feeds the existing negative fiscal signal on legitimacy and elite cohesion -> SIMULATION_VERSION 29 -> new demo fingerprint baseline
```

Public debt is no longer a causally inert counter. Both fiscal planners now add `debt * DEBT_INTEREST_BPS / 10_000` (`DEBT_INTEREST_BPS = 300`) to fiscal spending before the budget closes, so a country pays for past deficits with a worse present balance: more debt means more spending, a deeper deficit, more capitalized debt next year, and - through the existing fiscal signal - sustained pressure on legitimacy and elite cohesion. No new world state was added; the rule changes history for the same seed, so `SIMULATION_VERSION` moves 28 -> 29.

Acceptance evidence: formatting and the full quality gate pass (91 tests across all crates); a new gate test runs two worlds identical except for an initial 1,000,000 public debt and proves the indebted world closes the year exactly 1,030,000 deeper in debt - the seed principal plus the 300 bps interest, charged and capitalized. The old seed-1 50-year baseline `6474822005825804939` is retired because the demo world runs persistent deficits and its debt now compounds; the new baseline is `7530160749567993110`.

Next gate: surface debt service in the typed journal - the interest charge is folded into aggregate fiscal spending inside `CountryFiscalYearClosed` and cannot be audited as a separate domain event.

### Public debt service journal gate

Completed vertical slice:

```text
annual interest charge -> carried through CountryUpdate as opening debt + interest -> apply_year emits PublicDebtInterestCharged before the fiscal closure event -> chronicle aggregates the charges into an explicit yearly debt-service sentence -> event log and chronicle change, world state does not -> SIMULATION_VERSION stays 29 -> fingerprint baseline unchanged
```

Debt service is now auditable instead of being folded invisibly into aggregate fiscal spending. Every annual closure that charges interest emits a typed `PublicDebtInterestCharged { country, opening_debt, interest }` domain event next to `CountryFiscalYearClosed`, and the yearly chronicle reports the total charge across indebted countries as its own sentence. This keeps the causal chain readable in the archive: opening debt stock, the concrete interest charged on it, and the resulting fiscal closure are separate, replayable journal entries.

Acceptance evidence: formatting and the full quality gate pass (93 tests across all crates); a new simulation gate test proves a world seeded with 1,000,000 debt emits the event with exactly 30,000 interest while a debt-free world emits none, and a new chronicle gate test proves the sentence renders from the event alone. The stable fingerprint hashes world state only - not the event log - and no state or rule changed, so the seed-1 50-year baseline must stay at `7530160749567993110` - and it does.

Next gate: split procurement into plan/settle/record helpers - procurement.rs carries a TODO(refactor slice) and mixes planning, settlement, and observation recording in one monolithic function; the refactor must keep behaviour and the fingerprint baseline byte-identical.

### Procurement plan-settle-record refactor gate

Completed vertical slice:

```text
execute_monthly_firm_procurement -> plan_procurement_offer_book (pure deterministic offer book) -> settle_procurement_orders (all cash and inventory movement on a cloned ledger, committed only on full success) -> record_procurement_outcomes (purchase aggregates, FirmProcurementTrade events, market outcomes, revenue) -> behaviour byte-identical -> fingerprint baseline unchanged
```

The TODO(refactor slice) monolith in procurement.rs is gone. Planning, settlement, and recording are now separate functions with one narrow seam each: the offer book is a pure read, settlement is the only place resources move and stays atomic on error, and recording is the only place evidence is persisted. A private ProcurementSettlement struct carries fills, unmet demand, trade prices, and purchase aggregates between the phases. No rule changed: the slice is accepted only because the full gate stays green (93 tests, behaviour locked by the intermediate procurement suite) and the seed-1 50-year fingerprint stays at `7530160749567993110`.

The README was refreshed in the same slice: the repository layout now names crates/adam-content instead of the retired config/ directory, the quick start matches the release CLI invocation, and the foundation list and status describe the running monthly economy, fiscal closure with debt service, and the chronicle instead of the original scaffold.

Next gate: fiscal stress must feed back into spending - a country whose debt service consumes a growing share of its revenue should restrain discretionary spending instead of expanding it, so debt spirals become a policy problem rather than an arithmetic inevitability. This is a rule change: it will move the fingerprint baseline and bump SIMULATION_VERSION.

### Fiscal debt brake gate

Completed vertical slices (combined step 026):

```text
chronicle: CountryFiscalYearClosed -> fiscal totals sentence (revenue, spending, closing debt, countries) -> cli: prose chronicle printed before the final fingerprint -> rule: public debt to output ratio restrains discretionary spending (1 bps of spending per 10 bps of debt ratio, capped at 600 bps, spending floor 1_200 bps) -> SIMULATION_VERSION 30 -> new seed-1 50-year baseline 17363996423594943694
```

Three slices in one step. First, the yearly chronicle now reports what governments collected, spent, and owed at closure, derived only from CountryFiscalYearClosed events; second, the console runner prints the prose chronicle before the fingerprint, so a 50-year run ends with readable history instead of raw statistics. Both slices were gated against the previous baseline `7530160749567993110` and left it untouched, proving they observe without causing.

Third, the debt brake: a country whose accumulated public debt grows relative to its annual output now restrains discretionary spending before the deficit compounds further. The cause is a concrete monetary state (the debt stock and measured output), not a derived mood score; the effect is graduated, capped, and floored so states never spend below a survival floor. The gate test holds seed and legitimacy fixed and varies only opening debt: revenue is identical, spending is strictly lower for the indebted twin. This is a rule change, so SIMULATION_VERSION is now 30 and the new seed-1 50-year fingerprint baseline is `17363996423594943694` (95 tests green).

Next gate: seed comparison in the console runner - run two seeds side by side and report where their chronicles diverge, closing the Stage 0 requirement for seed comparison, plus a timed release profile of the 50-year run before any optimization work.

### Seed comparison and timing gate

Completed vertical slice (step 027):

```text
cli: --compare-seed N -> second deterministic world -> first divergent chronicle year reported side by side -> compare fingerprint printed -> timing line (simulated N years in X s, Y ms per year) after the fingerprint
```

The console runner now closes two Stage 0 requirements. Seed comparison: `--compare-seed` runs a second world under identical rules and reports the first year where the two chronicles diverge, proving that the seed shapes history itself rather than only the fingerprint header; comparing a seed with itself must report identical chronicles and an identical fingerprint, which doubles as a determinism check in release mode. Timing: every run now reports wall-clock duration and milliseconds per simulated year, giving the first measured profiling baseline before any optimization work. Wall-clock time is reporting-only diagnostics in the CLI layer; it never enters the simulation.

Acceptance: 95 tests green, seed-1 50-year fingerprint baseline unchanged at `17363996423594943694` (SIMULATION_VERSION 30), self-comparison identical, seeds 1 and 2 diverge.

Next gate: a second country in the demo content (TOML) so the chronicle narrates more than one fiscal and demographic path in a single run. This is a content change: the seed-1 baseline moves and the content schema gains a second country without any rule changes.

### Second country gate

Completed vertical slice (step 028):

```text
content schema v6: [[countries]] list + per-region country id -> demo adds Borealia with region Eastport (actor Sana Kettu, grain and bread firms) -> annual closure emits CountryFiscalYearClosed per country -> chronicle reports totals across 2 countries -> console runner narrates two national paths in one run
```

The Stage 0 requirement of several countries is now real content, not a schema promise. The scenario schema moved from a single `[country]` table to a `[[countries]]` list with an explicit `country` id on every region, so scenarios can shape any political map without code changes. Borealia starts between Arcadia's two extremes (mid buffers, mid wages), so its long-run fate is decided by the simulation, not by construction. A gate test builds the demo world, advances one economic year, and requires the chronicle to report fiscal closure across 2 countries.

This is a content change under unchanged rules: SIMULATION_VERSION stays 30 and the seed-1 50-year fingerprint moved to `8292603064090461814` as the new content baseline. Acceptance: 96 tests green, chronicle mentions two countries end to end, release self-comparison identical.

Next gate: named elite actors in the chronicle. Scenario actors (Mara Voss, Ilya Roden, Sana Kettu) exist in the world but the chronicle never names them; surface owner decisions as attributed chronicle lines so the letopis reads as history made by people, not aggregates.

### Named chronicle actors gate (step 029)

- The yearly chronicle resolves names from the journal itself (ActorRegistered / RegionRegistered)
  and narrates who acted: the actor behind the largest emergency relief, the most active
  production manager, and the region hit hardest by rationing.
- Derived narration only: no rules changed, no new events, SIMULATION_VERSION stays 30,
  seed-1/50y baseline unchanged: 8292603064090461814.
- Gate: 97 tests; the release chronicle must name at least one demo elite
  (Mara Voss / Ilya Roden / Sana Kettu).
- Next gate: first causal war slice - a concrete material grievance between two countries
  becomes a typed hostility state that disrupts cross-border trade, resolved deterministically
  inside the monthly tick.

### Cross-border intermediate procurement gate (step 030)

- A firm can now buy a recipe input from another region only when a directed logistics route
  connects the seller to the buyer. Local offers retain strict priority; reachable imports are
  then ordered by delivered price (offer price plus the direct-route tariff), region, and seller.
  No route leaves demand visibly unmet.
- Content schema v7 adds strict `[[routes]]` declarations, including carrier ownership. The demo
  removes Eastport's grain producer and supplies its bakery through the Northreach-to-Eastport
  road, making the second country materially dependent on external grain rather than merely
  adjacent in the chronicle.
- This is deliberately a narrow commercial bridge, not a duplicate shipment system: procurement
  does not reserve transport capacity or create in-transit stock, and its tariff remains part of
  the seller's settled delivered-price revenue. Carrier cash settlement, route capacity, delay,
  reliability, borders, and disruption remain authoritative logistics work for later slices.
- Gate coverage: `cross_region_procurement.rs` proves delivered pricing, local-first priority,
  unmet demand without a route, and replay through the shared annual command boundary. Full
  format, check, Clippy, test, and docs gate passes with 101 tests. This rule and content change
  advances SIMULATION_VERSION 30 -> 31; the new seed-1/50-year baseline is
  `7045927566456398391`, with release self-comparison byte-identical.
- Next gate: first causal war slice - a concrete bilateral hostility state severs cross-border
  procurement and makes foreign material dependence a readable conflict pressure.

### Bilateral hostility embargo gate (step 031)

- Countries now carry a canonical, symmetric bilateral-hostility relation. The replayable
  `SetCountryHostility` command validates two existing, distinct countries, records a typed
  `BilateralHostilityChanged` event only on transition, and includes the relation in persistence,
  equality, and the stable fingerprint.
- Active hostility is a concrete economic constraint: it removes otherwise valid direct foreign
  offers from firm procurement while leaving local commerce and domestic supply untouched. This
  makes the Northreach-to-Eastport dependency created in step 030 vulnerable to a political break
  rather than treating the route graph as scenery.
- Gate coverage extends `cross_region_procurement.rs` to two countries and proves that the same
  route which clears an import in peace yields explicit unmet input demand under hostility; the
  command-boundary replay is bit-identical and the transition is journaled. Full format, check,
  Clippy, test, and docs gate passes with 102 tests. SIMULATION_VERSION advances 31 -> 32 and the
  seed-1/50-year baseline is `10616160303486040494`; release self-comparison remains identical.
- Deliberate limit: hostility is currently a replayable authoritative decision, not yet an
  endogenous grievance or combat model; it blocks only cross-border intermediate procurement,
  not shipments or household markets.
- Next gate: derive a bounded bilateral grievance from observed cross-border material dependence
  and shortage, then make it deterministically propose or activate hostility so conflict has a
  material cause rather than an externally injected flag.

### Hostility chronicle gate (step 032)

- The yearly chronicle now resolves country names from registration events and narrates every
  typed bilateral-hostility transition in the year, including both escalation and de-escalation.
  It remains a pure event-log projection: no world rule, persistence layout, or fingerprinted
  state changed.
- Gate: a focused chronicle test creates Arcadia and Borealia, journals their hostility entry,
  and requires the rendered history sentence to name both countries. The full quality gate passes
  with 103 tests. SIMULATION_VERSION remains 32 and the seed-1/50-year baseline remains
  `10616160303486040494` because the demo has no hostility transition.
- Next gate: derive a bounded bilateral grievance from observed cross-border material dependence
  and shortage, then make it deterministically propose or activate hostility so conflict has a
  material cause rather than an externally injected flag.

### Material bilateral grievance gate (step 033)

- Conflict now has an executable material cause. Monthly firm procurement maintains a directed,
  bounded grievance level per (aggrieved, target) country pair, derived only from observed facts
  of the same month: a firm's unmet input demand, a foreign offer of that good in the pristine
  pre-settlement book, and a direct route that could deliver it in peace. Route reachability
  deliberately ignores hostility, so an embargo that blocks a deliverable good entrenches
  grievance instead of erasing the evidence.
- Dynamics are deterministic and bounded: +500 bps per evidence month, -250 bps decay per month
  without evidence, removal at zero, ceiling at 10000 bps. Every change is journaled as the new
  typed `BilateralGrievanceChanged` event, and crossing 7500 bps activates hostility through the
  same journaled transition as the step 031 command, so emergent conflict reuses the one
  authoritative hostility relation and embargo behavior. Grievance is authoritative world state:
  serialized, compared, and fingerprinted. ADR 0082 records the design.
- Gate coverage: new `bilateral_grievance.rs` proves accrual from a real unmet import against
  reachable foreign supply, no accrual when no route exists, decay to removal once imports
  resume, deterministic escalation to hostility at exactly the threshold month with the bounded
  ceiling afterwards, and bit-identical command-boundary replay. Full format, check, Clippy,
  test, and docs gate passes with 107 tests. The rule and fingerprint-schema change advances
  SIMULATION_VERSION 32 -> 33; the new seed-1/50-year baseline is `6416278853682209045`
  (31 ms per simulated year), and the demo chronicle stays grievance-free because its shortages
  are household-side rather than firm procurement against reachable foreign supply.
- Deliberate limits (per ADR 0082): evidence comes only from firm procurement over direct
  routes, constants are fixed, and active hostility never de-escalates on its own.
- Next gate: make peace reachable inside the simulation - when the grievance that caused an
  emergent hostility has decayed to zero with no fresh material evidence, deterministically
  deactivate that hostility through the same journaled transition.

### Emergent hostility de-escalation gate (step 034)

- Peace is now reachable inside the simulation. The world remembers which hostilities it created
  itself in a new canonical set `emergent_hostilities`; grievance escalation marks the pair when
  it activates hostility. After the monthly grievance update, a pair whose directed grievances
  have both decayed to removal is deactivated through the same journaled
  `set_country_hostility` transition, emitting the same `BilateralHostilityChanged` event that
  commands emit. The full material arc closes: shortage -> grievance -> hostility -> embargo ->
  domestic substitution -> grievance decay -> peace.
- Commanded hostility stays strictly stronger than the material system: pairs raised by
  `SetCountryHostility` are never marked emergent and never de-escalate on their own, and a
  commanded peace clears the emergent marker so the pair restarts from a clean slate. With
  +500/-250 bps dynamics an emergent hostility lasts at least 30 calm months, so embargoes have
  real duration without hand-tuned timers. ADR 0083 records the design.
- Gate coverage: new `hostility_deescalation.rs` proves that an emergent hostility de-escalates
  after a governed domestic producer ends the material cause (with bit-identical
  command-boundary replay), that a commanded hostility without grievance never de-escalates, and
  that a commanded peace clears the emergent marker. Full format, check, Clippy, test, and docs
  gate passes with 110 tests. The emergent set is serialized and fingerprinted, advancing
  SIMULATION_VERSION 33 -> 34; the new seed-1/50-year baseline is `8818694516742230572`
  (32 ms per simulated year) with an unchanged chronicle narrative.
- Deliberate limits (per ADR 0083): de-escalation consults only grievance state, not broader
  diplomacy; firm entry is still not a replayable command, so the gate test registers its
  domestic substitute producer on both timelines directly; only firms with an enacted policy
  plan market offers, so material recovery requires a governed domestic producer.
- Next gate candidates: derive grievance from household-side survival shortage so famines can
  also sour relations; narrate grievance accrual, escalation, and peace in the chronicle so the
  story is visible; or make firm entry a replayable `RegisterFirm` command so structural
  substitution can happen inside the journal instead of beside it.

### Replayable firm registration gate (step 035)

- Firm entry is now a journaled transition. A new `RegisterFirm(Firm)` command variant delegates
  to the existing validated `World::register_firm` transition, so a producer entering an
  existing history replays from commands alone instead of being applied beside the journal on
  both timelines. The variant is appended at the end of the command enum so previously recorded
  journals keep their encoding. ADR 0084 records the design.
- No rule or state-schema change: registration validation and atomicity are untouched,
  SIMULATION_VERSION stays at 34, and the seed-1/50-year baseline fingerprint is unchanged at
  `8818694516742230572`.
- Gate coverage: new command-boundary test `firm_registration_is_replayable_and_atomic` proves
  replay equality of world state and fingerprint plus atomic rejection of a duplicate
  registration, and the step 034 de-escalation gate now registers its mid-history domestic
  producer through the command in a real scenario. Full format, check, Clippy, test, and docs
  gate passes with 111 tests.
- Deliberate limits (per ADR 0084): firm entry stays exogenous - no entrepreneur model, entry
  costs, or capital raising - and ownership stakes, appointments, and actor registration still
  lack command wrappers.
- Next gate candidates: derive grievance from household-side survival shortage so famines can
  also sour relations, or narrate grievance accrual, escalation, and peace in the chronicle so
  the emergent conflict arc is visible in the year-by-year story.

### Grievance arc narration gate (step 036)

- The yearly chronicle now narrates the whole emergent conflict arc by country name. Each
  directed grievance pair contributes one deterministic sentence per year - "deepened to N
  basis points", "eased to N basis points", or "resolved its material grievance" when the year
  ends at zero - placed before the hostility sentences so cause precedes effect. Grievance and
  hostility narration share one helper, keeping the year-finishing function under the Clippy
  line budget. ADR 0085 records the design.
- Importance tiers: any hostility change now ranks the year at 85, and grievance-only diplomacy
  ranks 50, so conflict years surface above routine production years without displacing
  survival crises.
- The chronicle stays a pure derivation from journaled events: no state or event changes,
  SIMULATION_VERSION stays at 34, and the seed-1/50-year baseline fingerprint is unchanged at
  `8818694516742230572`.
- Gate coverage: new test `chronicle_narrates_the_grievance_arc_by_country_name` pins a
  two-year escalation-then-peace story (deepen to 7500, enter hostility, resolve to zero, end
  hostility, importance 85 both years). Full format, check, Clippy, test, and docs gate passes
  with 112 tests.
- Deliberate limits (per ADR 0085): the chronicle reports first-vs-last level per year rather
  than intra-year oscillations, and grievance attribution stays at country level.
- Next gate: derive grievance from household-side survival shortage so famines - not only
  unmet firm imports - can sour relations between countries through the same journaled
  grievance ledger.

### Household survival grievance gate (step 037)

- Household survival shortage now enters the same journaled bilateral grievance ledger as unmet
  firm imports. After the household market clears, an unmet `NeedTier::Survival` need accrues
  grievance toward a foreign country only when its firm offered the same good over a direct route
  that could deliver in peace. Shortage without a route still blames nobody; multiple firm and
  household shortages union into one canonical country pair, so they cannot double-accrue a month.
  ADR 0086 records the design.
- Timing is explicit: firm evidence uses its immutable pre-procurement offer book, while household
  evidence uses the offer book presented to the household market after firm procurement. This
  preserves existing firm escalation/de-escalation behaviour while grounding household grievance
  in the authoritative market shortfall.
- Gate coverage: two new commercial-cycle tests prove a suspended local farm plus reachable
  foreign food supplier yields an unmet survival need and a replay-identical 500-bps grievance,
  while the identical world without a route creates no grievance. Full format, check, Clippy,
  test, and docs gate passes with 114 tests.
- No new state schema: `SIMULATION_VERSION` stays at 34; seed-1/50-year baseline remains
  `8818694516742230572` (32.3 ms per simulated year).
- Deliberate limits (per ADR 0086): household markets still settle locally; the foreign offer is
  evidence that peaceful logistics could supply the need, not yet an executed cross-border retail
  import. Delivered household pricing, tariff, capacity, and shipment mechanics remain separate.
- Next gate: make household survival imports settle across a direct peaceful route, with delivered
  price and route cost, so reachable foreign food can avert the shortage rather than only explain
  its diplomatic consequence.

### Delivered household survival imports gate (step 038)

- Household survival imports now settle across a direct peaceful route. The generalized
  `clear_market_with_delivery` keeps one shared inventory remainder per source offer: local offers
  retain strict priority, then survival needs consider foreign offers by delivered price (offer
  price plus the lowest direct route tariff), source region, and seller. The existing local-market
  API remains a wrapper with no import routes, so local callers keep their prior behaviour.
- The shared `direct_market_route_cost` rejects hostile country pairs and uses the lowest-tariff
  direct route with stable tie-breaking. Market fills debit household wealth at delivered price,
  credit the foreign seller through the existing settlement, and preserve the seller's source
  region in outcomes. ADR 0087 records the design.
- Gate coverage: the former household-famine scenario now proves a full replay-identical import
  from foreign farm to household at 9 offer + 1 route cost = 10, no survival unmet demand, and no
  grievance; the no-route scenario still leaves the demand unmet. Full format, check, Clippy,
  test, and docs gate passes with 114 tests.
- This changes authoritative household-market outcomes without adding state schema:
  SIMULATION_VERSION stays at 34; the seed-1/50-year release baseline changes to
  `12100901864703017553` (30.5 ms per simulated year).
- Deliberate limits (per ADR 0087): imports remain immediate accounting transfers without route
  capacity, terminals, or shipment progress; only survival needs may import; and household demand
  budgets still use local reference prices, so a more expensive delivered import can remain partly
  unaffordable.
- Next gate: make household demand reserve against the cheapest reachable delivered survival price
  rather than only the local reference price, so a solvent household can correctly budget for an
  import that costs more than its local benchmark.

### Delivered-price survival budgeting gate (step 039)

- Household survival demand now reserves against the current priority supply rather than blindly
  against its local reference price. With a local offer it quotes the cheapest local offer; with
  no local offer it quotes the cheapest peaceful direct foreign offer plus route tariff. The
  public no-offer planner remains a local-reference wrapper, and participation-plus tiers remain
  local-reference priced. ADR 0088 records the design.
- The commercial cycle passes its current offer book to the new offer-aware planner before it
  creates market orders, so the reserve matches the same survival import the market can settle.
  This removes the artificial partial fulfillment of a solvent household facing a pricier import.
- Gate coverage: the household import scenario now gives the cohort exactly 11 minor units and
  proves that it reserves 11 for foreign offer 10 + route tariff 1, buys the full unit, leaves no
  unmet survival demand or grievance, and replays bit-identically. Full format, check, Clippy,
  test, and docs gate passes with 114 tests.
- No state schema or version change: SIMULATION_VERSION stays at 34; the seed-1/50-year release
  baseline stays `12100901864703017553` (40.9 ms per simulated year).
- Deliberate limits (per ADR 0088): local offer priority can still beat a cheaper foreign offer,
  partial local supply is not yet quoted as a blended source price, offers are not reserved during
  planning, and emergency-relief costing still uses local reference prices.
- Next gate: quote partial local survival supply plus reachable imports as an explicit deterministic
  multi-source budget, so households can reserve the true local-first blended cost when the local
  market can cover only part of a basic need.

### Blended local-first survival budgeting gate (step 040)

- Survival targets now quote a deterministic source basket: local offers by price and seller first,
  then peaceful direct imports by delivered price, origin, and seller. `PricedTarget` carries the
  resulting basket cost directly, so proportional budgeting no longer assumes one fictitious price.
- Source-segment order limits round up, matching the minimum cash ceiling required by market
  affordability for fractional quantities; market settlement remains unchanged and rounds actual
  spending down. ADR 0089 records the decision.
- Gate coverage: 500 milli-units local at 10 plus 500 imported at 13 reserves 12 (5 + ceil(6.5)),
  buys both portions, settles 11 actual units, leaves no survival shortage, and replays exactly.
  Full format, check, Clippy, test, and docs gate passes with 115 tests.
- No schema or version change: SIMULATION_VERSION remains 34 and the seed-1/50-year baseline stays
  `12100901864703017553` (39.8 ms per simulated year).
- Deliberate limit: concurrent cohorts still quote shared supply independently before canonical
  clearing; a future gate should allocate planning-time source capacity across ordered household
  intents so aggregate reservation cannot overstate a scarce offer.

### Shared survival supply planning gate (step 041)

- Household survival planning now consumes a shared planning copy of market offer quantities in
  the exact canonical market order (offers by region, good, price, seller; cohorts by region and
  id), so one scarce offer can no longer be reserved by several cohorts at once.
- The shared ledger is active only while every stored government emergency policy keeps the
  default `MarketAllocation` shortage strategy; under `ProportionalRationing` planning keeps
  independent per-cohort quotes because rationing applies proportional quotas separately before
  clearing. ADR 0090 records the decision.
- Gate coverage: with one scarce foreign offer (1000 milli-units at delivered price 11) and two
  identical cohorts, the first canonical buyer reserves 11 and settles the full unit, the second
  cohort falls back to the reference price 10 and leaves explicit unmet demand, and the month
  replays bit-identically; the proportional-rationing gate is unchanged. Full format, check,
  Clippy, test, and docs gate passes with 116 tests.
- No state schema or version change: SIMULATION_VERSION stays at 34 and the seed-1/50-year
  release baseline stays `12100901864703017553` (35.7 ms per simulated year).
- Deliberate limits: only survival tiers consume the planning ledger, emergency-relief costing
  remains reference-priced, and routes remain immediate accounting links without capacity
  constraints.

### Capacity-constrained household import gate (step 042)

- Household survival imports are now capped by the remaining uncontracted
  monthly capacity of the selected route: monthly capacity minus active
  freight-contract reservations and spot freight usage; in-transit shipments
  already occupy one of those pools (ADR 0091).
- `clear_market_with_delivery` consumes a shared per-route capacity map across
  all orders of the month; the demand planner mirrors the same cap so reserved
  budgets anticipate capped fills plus the reference-priced fallback.
- Concrete gate: with foreign price 6, tariff 1, and route capacity 600 for a
  1000-unit survival need, the planner reserves 9, clearing fills exactly 600
  at spend 4, and 400 stays unmet; replay reproduces the same state.
- Demo history is unchanged (fingerprint 12100901864703017553,
  SIMULATION_VERSION 34) because demo routes have ample capacity.
- Known limits: firm-to-firm procurement imports are not capacity-capped yet;
  the proportional-rationing planning path caps quotes without sharing
  capacity between cohorts; relief purchasing still uses reference prices.

### Import capacity audit gate (step 043)

- Fixed a double count in the market import capacity pool: in-transit
  shipments were subtracted on top of the contract and spot freight pools
  that already carry them, understating the capacity available to household
  imports (ADR 0091 formula corrected).
- Concrete gate: with route capacity 1000 and a 400-unit spot shipment in
  transit, the market pool is 600 (not 200); after delivery it returns to
  1000, and an idle active 300-unit freight contract lowers it to 700.
- Demo history is unchanged (fingerprint 12100901864703017553,
  SIMULATION_VERSION 34).

### Shared route capacity pool for B2B and household imports (step 044)

- B2B firm procurement imports and household survival imports now compete for
  the same physical monthly route capacity pool. A single `market_spot_route_capacity`
  map is computed once per commercial cycle; firm procurement fills consume from
  it first, then market clearing bounds household import fills by whatever
  remains. Demand planning receives a clone for first-canonical-buyer reservation
  logic independently of clearing. ADR 0092 records the design.
- Concrete gate: route capacity 1000, B2B demand 1000 milli-units of grain.
  Ample capacity (10 000): both flows coexist freely.
  Tight capacity (1 000 == B2B demand): B2B fills in full; zero route capacity
  remains for household imports on the same route.
  Insufficient capacity (500): B2B import capped at 500; unmet procurement
  500 milli-Grain recorded.
  Year replayability confirmed.
- `direct_market_route_cost` removed (dead code after procurement switched to
  `direct_market_route` directly).
- Demo history unchanged: fingerprint 12100901864703017553,
  SIMULATION_VERSION 34 (32.3 ms per simulated year).

### Procurement-shortfall chronicle gate (step 045)

- Firm procurement shortages are now journaled as canonical
  `FirmProcurementShortfall { buyer, good, quantity }` evidence once per
  unmet buyer/input pair and month. The event records a material fact only:
  it neither moves resources nor changes the grievance rule or world state.
- The yearly chronicle aggregates the evidence by firm and input, resolves
  current names, and reports the causal fact directly — for example,
  `Bakery could not procure 500 milli-units of Grain.` B2B-only shortage years
  receive importance 75, while existing survival/death tiers remain stronger.
  ADR 0093 records the decision.
- Gate: the 500-capacity B2B route scenario produces named Bakery/Grain/500
  narration and preserves command-boundary replay. Full quality gate passes
  with 122 tests; the seed-1/50-year release fingerprint remains
  `12100901864703017553` (34.7 ms per simulated year, SIMULATION_VERSION 34).

### Capacity-aware bilateral grievance gate (step 046)

- Firm procurement now classifies a remaining input shortage as route-capacity
  limited only when an eligible foreign offer still has stock while its selected
  direct route has exhausted the shared monthly capacity pool. This transient
  settlement evidence distinguishes infrastructure scarcity from a supplier
  withholding a deliverable good.
- Capacity-limited shortages do not accrue new bilateral grievance toward the
  foreign supplier; existing grievance still follows its normal decay path.
  Missing routes, hostile borders, and non-capacity shortage evidence retain
  their previous behavior. ADR 0094 records the decision.
- Gate: a cross-border Bakery demand of 1,000 through a 400-capacity road fills
  400, leaves 600 explicitly unmet, creates no grievance against the Farm's
  country, and replays identically. Full validation and release fingerprint
  recorded with the commit.

### Capacity-specific procurement evidence gate (step 047)

- Capacity-limited B2B shortages now emit the dedicated appended event
  `FirmProcurementRouteCapacityShortfall` instead of the generic procurement
  shortfall event. The existing event shape is unchanged, preserving the
  serialized contract for earlier journal entries.
- The yearly chronicle carries the settlement cause into named narration:
  `Route capacity prevented Bakery from procuring 500 milli-units of Grain.`
  Generic shortages keep their previous wording and both retain importance 75.
- This is an observability-only slice: resource movement, allocation,
  diplomacy, world state, SIMULATION_VERSION, and fingerprint inputs are
  unchanged. ADR 0095 records the decision. The full gate passes with 123
  tests; the seed-1/50-year fingerprint remains `12100901864703017553`
  (34.7 ms per simulated year, SIMULATION_VERSION 34).


### Bounded input-shortage management gate (step 048)

- Procurement shortages now constrain the ordinary observed-management target:
  a firm with a positive target cannot expand during the shortage, contracts by
  at most one batch per month, and keeps a one-batch floor. This prevents a
  temporary partial delivery from setting target zero and suppressing the very
  procurement order required for recovery.
- The response reads the current month's typed generic or route-capacity
  procurement-shortfall event; no parallel shortage state is introduced.
  Ordinary unsold-output contraction without an input shortfall is unchanged.
- Gate: with a 500-unit route cap and a 1,000-unit Grain requirement, the Bakery
  keeps target one, accumulates two partial deliveries across isolated replayed
  commercial months, and resumes one Bread batch in the third month. ADR 0096
  records the rule. This changes target history, so SIMULATION_VERSION advances
  34 -> 35. Full gate: 124 tests; seed-1/50-year fingerprint
  `12134027468760220507`; 35.9 ms/year.


### Market-import carrier revenue gate (step 049)

- Immediate household and B2B imports now split one delivered payment into goods revenue for the
  supplier and freight revenue for the selected route carrier. The buyer still pays the exact
  delivered total; fixed-point rounding assigns the residual to freight so buyer outflow always
  equals the two credited legs.
- Household goods revenue remains taxable final sales. Carrier freight is ordinary non-final firm
  revenue, while B2B goods and freight remain non-final. Firm buyers observe the full delivered
  input price; supplier-side market outcomes retain the actual goods offer price.
- Appended `MarketFreightPaid` and `FirmProcurementFreightPaid` events expose the carrier leg without
  changing existing trade-event semantics. The yearly chronicle aggregates freight payments into
  auditable route-carrier revenue. ADR 0097 records the contract.
- Separate-carrier gates prove seller/carrier cash allocation, accounting classification, delivered
  buyer prices, seller goods prices, typed evidence, money conservation, and command replay. Full
  workspace validation passes 125 tests. The authoritative cash and accounting change advances
  SIMULATION_VERSION 35 -> 36; seed-1/50-year fingerprint is `6856032423052036241` at 27.6 ms/year.
- Next gate: use observed carrier revenue, margin, and exhausted route capacity as evidence for a
  bounded, financed route-capacity investment response rather than an artificial expansion rule.


### Carrier-funded route-capacity expansion gate (step 050)

- Household and B2B matching now retain a route-specific constrained signal only when the shared
  route limit cuts an otherwise feasible fill after demand, stock, and affordability are considered.
  The commercial cycle unions both flows after settlement, so one authoritative response sees the
  whole month without double counting shared capacity.
- A route must be constrained and earn positive typed freight revenue for three consecutive months.
  Its registered carrier may then spend cash equal to twelve months of tariff revenue on a bounded
  10% capacity increment. Missing revenue clears the streak; insufficient cash preserves capped
  pressure without mutating cash or capacity. Successful construction clears pressure and emits
  `RouteCapacityExpanded`. ADR 0098 records the rule and Stage-0 capital-expenditure treatment.
- Pressure is persisted and fingerprinted, while the route remains the single authoritative capacity
  location read by every market and freight consumer. Existing commercial/economic commands replay
  the automatic response. The yearly chronicle aggregates installed capacity and carrier spending.
- Gates prove pressure and revenue requirements, exact carrier debit, exact capacity growth,
  insufficient-cash atomicity, typed evidence, command replay, and stable fingerprints. Full
  workspace validation passes 128 tests. The causal state change advances SIMULATION_VERSION
  36 -> 37; seed-1/50-year fingerprint is `8785785093042010742` at 27.4 ms/year.
- Next gate: audit the demo's long-run production collapse and public-debt spiral, then choose a
  larger causal slice that connects firm exit/recovery, credit, or fiscal default to existing
  accounting evidence rather than adding isolated surface mechanics.

### Sovereign debt restructuring gate (step 051)

- A 50-year seed-1 audit confirmed the fiscal failure mode: after the spending floor bound, Arcadia's
  debt service kept compounding while debt remained an inert stock with no terminal institutional
  event. The audit selected fiscal default before adding new surface systems.
- Annual closure now restructures public debt only when closing debt exceeds twice measured annual
  output and annual interest reaches at least one third of realized tax revenue. The settlement
  writes off 40% of principal, then applies an 800-bp legitimacy shock and a 500-bp elite-cohesion
  shock. This creates a real recovery-versus-instability tradeoff rather than free debt deletion.
- `PublicDebtRestructured` records debt before and after plus principal written off; the chronicle
  narrates the crisis and gives it importance 88. ADR 0099 records the Stage-0 aggregate-creditor
  approximation and the path toward explicit bonds, maturities, holders, and negotiated defaults.
- Gate coverage proves the exact haircut, authoritative closing debt, political cost, non-trigger in
  a sustainable twin, and chronicle narration. The full format/check/Clippy/test/docs gate passes
  with 130 tests. The rule change advances SIMULATION_VERSION 37 -> 38. In the seed-1 release run,
  Arcadia restructures in 2070 and writes off 133,851 minor units; the new 50-year fingerprint is
  `18077342798678252871`, self-comparison is identical, and runtime is 34.5 ms/year.
- Next gate: introduce evidence-driven firm distress and owner/creditor recapitalization. Repeated
  wage arrears and exhausted cash should force a governed choice among fresh owner capital,
  downsizing, insolvency, or closure, while preserving inventories, worker claims, and replayability.


### Owner-funded firm recapitalization gate (step 052)

- Persistent payroll failure now creates authoritative firm distress: each post-payroll month with
  outstanding wage arrears advances a canonical per-firm counter, while a clean payroll clears it.
- After three consecutive distressed payrolls, the largest economic owner injects personal cash
  up to the smaller of available owner liquidity and total worker claims. Equal ownership resolves
  by stable actor id. The transfer debits owner cash and credits firm cash without cancelling wage
  arrears, so workers remain owed and receive the rescue only through ordinary future payroll.
- Missing owner liquidity creates no money and keeps distress at the threshold for reconsideration.
  `FirmRecapitalized` journals the owner, firm, amount, and preserved arrears; the yearly chronicle
  reports aggregate recapitalizations. ADR 0100 records the rule.
- Gate coverage proves the exact three-month trigger, bounded owner debit, firm credit, preserved
  worker claim, distress reset, typed evidence, and deterministic replay. The full workspace test
  suite passes 131 tests. The new fingerprinted distress state advances SIMULATION_VERSION 38 -> 39;
  seed-1/50-year fingerprint is `10583892862554856218`, with identical self-comparison and 42.0 ms/year.
- Deliberate limits: recapitalization is automatic once evidence and liquidity bind; there is no
  dilution, board vote, creditor hierarchy, or closure yet. Next gate: convert unrecoverable distress
  into governed downsizing and eventual insolvency/closure while preserving inventories and worker
  claims, rather than letting cashless firms persist forever.


### Governed distress downsizing gate (step 053)

- A firm that reaches the three-month wage-arrears threshold without owner liquidity now follows a
  second causal branch instead of remaining permanently overstaffed. Its authorized operations actor
  reduces each active employment agreement by 25%, rounded up to at least one worker. The bounded
  response can repeat while distress persists, trading productive capacity for a lower future wage bill.
- Downsizing uses the ordinary employment transition, so production labor, cohort employment, and the
  journal stay coherent. `FirmDownsizedForDistress` records the responsible actor, workers before and
  after, and outstanding claims; the yearly chronicle reports workers released by distressed firms.
- Fixed a prerequisite accounting gap: inactive employment agreements with arrears now remain in
  payroll settlement. Terminated workers accrue no new wages but retain their existing claim, which a
  late recapitalization can fund and ordinary payroll can subsequently pay. ADR 0101 records the rule.
- Gate coverage proves the cashless branch, exact bounded reduction, preserved post-layoff arrears, late
  owner rescue, eventual worker payment, typed chronicle evidence, and deterministic monthly replay.
  The workspace now passes 132 tests. The rule change advances SIMULATION_VERSION 39 -> 40; the
  seed-1/50-year fingerprint is `14265164837894074090`, self-comparison is identical, and runtime is
  40.8 ms/year.
- Deliberate limit: a zero-workforce firm still owns its cash, inventories, capacity, contracts, and
  claims. Next gate: explicit insolvency after prolonged zero-workforce distress, with frozen operations,
  an auditable asset-and-claim snapshot, and a recovery path that cannot silently destroy inventories
  or unpaid wages.


### Firm insolvency administration gate (step 054)

- Six consecutive months of wage-arrears distress now declare a zero-workforce firm insolvent when no owner rescue is available. The ordinary operations actor becomes administrator.
- Declaration preserves and fingerprints an immutable cash, inventory, and wage-claim snapshot. Insolvent firms cannot produce, offer goods, procure inputs, change autonomous targets, or finance route expansion; estate assets and terminated-worker claims remain intact.
- `FirmInsolvencyDeclared` makes the transition auditable, and the chronicle reports preserved claims and inventory. ADR 0102 records the boundary.
- Full format, check, Clippy, test, and docs gate passes with 132 workspace tests. SIMULATION_VERSION advances 40 -> 41. The seed-1 50-year fingerprint is `6714888034744335345`; self-comparison is identical at 42.0 ms/year. The demo reaches one insolvency in 2041 after six downsizing actions, preserving 36,432 minor units of unpaid wages.
- Next gate: governed reorganization. An administrator should be able to reopen only after worker claims are funded and a viable staffing/production plan exists; otherwise a later creditor-priority and asset-sale gate should liquidate transparently.


### Funded firm reorganization gate (step 055)

- Insolvency now has a governed recovery path through the shared command boundary. The appointed administrator submits a sponsor contribution, cohort staffing plan, and production target.
- Reopening is atomic and requires an economic-owner sponsor, payment of every preserved wage claim, positive valid staffing, a feasible target, and enough post-claim cash for one complete month of proposed payroll. Direct target changes and rehiring remain blocked while insolvency is active.
- Acceptance transfers real owner cash, pays worker households, restores employment and production through ordinary transitions, removes administration, and journals `FirmReorganized`; the chronicle reports contributions, claims paid, and returning workers. ADR 0103 records the rule.
- The rejection gate proves underfunding changes nothing; command replay reproduces state and fingerprint exactly. Full format/check/Clippy/test/docs passes with 133 workspace tests. SIMULATION_VERSION advances 41 -> 42. Seed-1/50-year fingerprint is `4178072713035629664`, self-comparison is identical at 42.4 ms/year.
- The demo still leaves its 2041 insolvency unresolved because no autonomous owner chooses a plan. Next gate: evidence-based administrator proposals and owner acceptance, followed by transparent liquidation if no fully funded plan appears within a bounded administration period.

### Policy-driven autonomous firm reorganization gate (step 056)

- Insolvent firms are now reconsidered every economic month after payroll and distress resolution. The administrator derives a canonical minimum viable plan from preserved wage claims, one-batch labor requirements, available workers, existing wage agreements, estate cash, ownership, and owner liquidity.
- The largest economic owner sponsors the plan, with stable actor-id tie-breaking. Reopening is attempted only when the exact contribution fits both current owner cash and the firm's authorized reinvestment share; cautious policy therefore preserves owner liquidity while a recovery policy can restore production.
- Derived plans execute through the existing funded-reorganization boundary, preserving administrator authority, complete worker-claim priority, one month of payroll runway, atomicity, typed events, save/replay, and player/AI rule parity. ADR 0104 records the decision.
- Gate coverage proves policy refusal, exact minimum funding, canonical one-worker staffing, claim payment, production-target restoration, ordinary command replay, and stable fingerprints. Full workspace tests pass with 134 tests. The rule is integrated into the monthly economic cycle and advances SIMULATION_VERSION 42 -> 43.
- The seed-1 50-year demo remains materially unchanged because the insolvent firm's owner cannot afford a compliant plan; the new fingerprint is `5456998744248754763`, changed by the simulation-version contract. Next gate: bounded administration and transparent liquidation when no viable funded plan emerges, with worker claims paid first from estate cash and explicit treatment of inventories and residual owner value.

### Bounded firm liquidation gate (step 057)

- Insolvency administration now has a twelve-month deadline. Each monthly cycle attempts the existing funded reorganization first; only an estate that still lacks a viable plan at the deadline enters terminal liquidation.
- Estate cash pays wage claims before owners. Underfunded claims share available cash proportionally through deterministic largest-remainder allocation; unpaid balances are explicitly written off instead of persisting as phantom claims. Residual cash is distributed by economic ownership rights.
- Unsold inventory is explicitly written off because Stage 0 has no physical estate auction or public inventory buyer. The liquidation event records worker recovery, claim losses, inventory losses, and owner recovery; the liquidated estate remains permanently frozen and cannot reorganize.
- The transition is atomic, available through the shared command boundary, integrated into the economic month, serialized, fingerprinted, and narrated in the yearly chronicle. ADR 0105 records the decision. Focused gates cover the exact twelve-month boundary, worker priority, partial recovery, residual owner value, terminal state, and deterministic replay.
- The rule changes authoritative claims, cash, inventory, and insolvency state, so SIMULATION_VERSION advances 43 -> 44. The full format/check/Clippy/test/docs gate passes with 137 workspace tests. Two seed-1 release runs produce byte-identical chronicles and fingerprints after excluding the intentionally variable runtime line; the 50-year fingerprint is `1313592537849333646`. In the demo, the unresolved 2041 insolvency liquidates in 2042 and explicitly writes off 36,432 minor units of unfunded wage claims.
- Next gate: replace blanket inventory write-off with a physical estate auction to real solvent firms, while adding creditor classes and preserving money, inventory, and claim priority exactly.

### Physical estate inventory auction gate (step 058)

- Terminal liquidation now attempts a real local asset sale before destroying stock. Solvent producers buy only recipe inputs they can use, up to their authorized production target, current inventory shortfall, and available cash; buyers and goods resolve in canonical order.
- Every sale moves physical inventory and cash atomically at the regional reference price. Proceeds enter the existing estate waterfall, so workers retain priority and owners receive only the residual. Unmarketable stock remains an explicit write-off rather than disappearing silently.
- `FirmLiquidationInventorySold` provides typed evidence, and the chronicle reports inventory preserved and proceeds recovered alongside claims and residual losses. ADR 0106 records the administered Stage-0 auction and its local/reference-price limits.
- Full format, check, Clippy, 137-test, and documentation gates pass. `SIMULATION_VERSION` advances 44 -> 45; the seed-1 50-year self-comparison is identical with fingerprint `11514179436319650475` at 53.7 ms/year.
- Next gate: add explicit creditor claims and priority classes, then let liquidation transfer productive capacity or successor ownership without inventing value.

### Ranked firm creditor claims gate (step 059)

- Operating firms can now borrow real actor cash through the shared command boundary. Issuance atomically debits the creditor, credits the firm, and creates a persisted, fingerprinted principal claim; insolvent firms cannot originate fresh credit.
- Liquidation now enforces an explicit waterfall: worker wage claims first, then secured creditor principal, then unsecured creditor principal, then owners. Scarce value within one rank is allocated proportionally through deterministic largest-remainder allocation with canonical tie-breaking.
- Each creditor settlement journals paid and written-off principal, settled claims leave authoritative state, and the yearly chronicle exposes aggregate creditor recovery and loss. Reorganization deliberately preserves outstanding principal rather than granting an implicit debt discharge.
- ADR 0107 records the contract and Stage-0 limits: no interest, maturity, collateral matching, covenants, loan trading, bank balance sheets, or repayment schedule yet.
- Gate coverage proves money-conserving replayable loan issuance, worker seniority, secured-before-unsecured recovery, explicit write-offs, terminal claim removal, liquidation replay, and stable fingerprints. Full workspace tests pass with 138 tests. The new creditor state advances `SIMULATION_VERSION` 45 -> 46; the seed-1/50-year self-comparison remains byte-identical with fingerprint `7807349546474967676` at 58.1 ms/year.
- Next gate: let a liquidated estate sell installed productive capacity to a solvent successor, preserving physical capacity and transferring ownership without inventing output or value.

### Physical productive-capacity successor sale gate (step 060)

- Terminal liquidation now offers installed production capacity to solvent local firms using the same recipe before retiring the estate. A successor must already target its installed ceiling, retain one full month of active payroll after purchase, and can at most double its current scale in one administered sale.
- Capacity is priced at one month of reference-price gross output per batch. Every accepted sale moves real buyer cash and physical capacity atomically; proceeds enter the existing worker-first, secured-creditor, unsecured-creditor, and owner waterfall. The auction never changes the buyer's production target or creates output.
- Unsold capacity is explicitly retired and narrated rather than silently disappearing. `FirmLiquidationCapacitySold` records estate, successor, batches, and proceeds; liquidation results and the yearly chronicle distinguish transferred from retired capacity. ADR 0108 records the rule and its strict same-region/same-recipe Stage-0 boundary.
- Focused coverage proves price formation, capacity conservation, cash conservation, improved worker recovery, terminal estate retirement, typed evidence, command replay, and stable fingerprints. The workspace passes 139 tests. `SIMULATION_VERSION` advances 46 -> 47; the seed-1/50-year release fingerprint is `310691261537918658` at 63.9 ms/year.
- Next gate: represent creditor maturity and scheduled repayment for operating firms, so credit affects monthly liquidity before insolvency rather than existing only as liquidation priority.

### Scheduled firm debt service gate (step 061)

- Operating firms can now issue actor-funded credit with a deterministic 1-120 month principal schedule. The first installment falls due one calendar month after issuance; principal amortizes by rounded-up remaining-balance division so the final installment absorbs every remainder.
- Monthly causality is explicit: payroll is settled first, then scheduled principal, then distress response, reorganization, and liquidation. Firms pay only available cash, creditors receive only real transferred money, partial payments preserve unpaid principal, and complete repayment removes the claim.
- Maturity does not forgive debt. Any remaining balance becomes overdue and is attempted in later operating months. Insolvency freezes ordinary scheduled service without erasing the claim, so workers remain senior and the balance enters the existing secured/unsecured liquidation waterfall.
- `ScheduledFirmCreditIssued` and `FirmDebtServiceSettled` make issuance, payment, remaining principal, and overdue status auditable. Schedule state is serialized, replayable, and fingerprinted. The chronicle reports annual due, paid, unpaid, and overdue attempts. ADR 0109 records the contract.
- Focused coverage proves deterministic amortization, partial payment, maturity, payroll seniority, complete claim removal, insolvency freeze, worker-first liquidation recovery, command replay, and chronicle narration. `SIMULATION_VERSION` advances 47 -> 48. The accepted seed-1/50-year fingerprint is `8401388131904837857`; self-comparison is identical.
- Deliberate limits: principal-only debt has no interest, grace period, acceleration covenant before maturity, refinancing, collateral revaluation, loan trading, or bank balance sheet. Next gate: introduce priced firm credit offers with interest and underwriting based on observable cash-flow and collateral evidence, without allowing forecasts to create money or guarantees.

### Observed priced firm credit gate (step 062)

- Lenders can now create concrete two-month offers only after three operating observations. Underwriting subtracts observed input costs, active payroll, and existing scheduled debt service from average realized sales; at most half of residual monthly surplus supports new debt. Requested principal is further bounded by term, annual interest, lender cash, and - for secured credit - 70% of observable inventory and installed-capacity value.
- Majority owners and chief executives accept live offers through the shared command boundary. Acceptance moves real lender cash, creates the existing ranked claim, removes the offer, and exposes only live offers as concrete expected financing to management forecasts. No forecast creates money or guarantees acceptance.
- Interest now accrues monthly on outstanding principal. Cash settles accrued interest before principal; unpaid interest persists through maturity, freezes in insolvency, and joins principal in the worker-first liquidation waterfall. Debt-service events and the chronicle distinguish interest charged and paid.
- Offers, underwriting evidence, rates, accrued interest, commands, events, save state, and fingerprints are deterministic. ADR 0110 records the rule. Focused replay coverage proves exact approval evidence, offer-aware expectations, authorized acceptance, real cash transfer, interest accrual, interest-first payment, and stable fingerprints.
- The full workspace test suite passes 144 tests. `SIMULATION_VERSION` advances 48 -> 49. The seed-1/50-year demo remains materially unchanged because its actors do not yet originate offers; the accepted fingerprint is `116941166565012856`, with an identical seed-1 self-comparison at 56.3 ms/year.
- Next gate: autonomous lender search and borrower acceptance. Distressed but viable firms should request competing offers; lenders should choose among firms using return, concentration, liquidity reserve, collateral, and default evidence instead of credit appearing only by external command.

### Autonomous firm credit market gate (step 063)

- Viable operating firms now search for working capital after monthly commerce and management when three observations show positive operating surplus but current cash cannot cover one target-based month of inputs, payroll, and preserved wage claims. The request is exactly that gap; insolvent firms and firms already carrying creditor claims cannot stack autonomous debt.
- Domestic non-owner lenders compete through the existing evidence-based underwriting boundary. Each keeps 50% of current cash liquid, total firm-credit exposure is capped at 40% of liquid cash plus claims, existing concentration raises the rate, and every borrower distress month adds 250 bps. Autonomous offers are secured and remain bounded by the existing cash-flow and 70% collateral tests.
- An authorized majority owner or CEO accepts the cheapest offer covering at least 25% of the gap, preferring larger principal and then stable actor id on ties. Rejected competing offers are removed. The accepted loan uses the ordinary command, real cash transfer, ranked claim, interest schedule, insolvency freeze, and liquidation waterfall.
- The credit market is an atomic once-per-month stage inside the economic cycle. Typed completion evidence, cycle results, save/replay state, fingerprinting, and chronicle narration are integrated. ADR 0111 records the rule.
- Focused coverage proves a 100-unit viable gap, two domestic lenders, canonical competition, a 600-bp accepted loan, exact lender/firm cash movement, rejected-offer cleanup, duplicate-stage atomicity, command replay, stable fingerprints, and event-derived chronicle prose. The full workspace gate passes 145 tests.
- `SIMULATION_VERSION` advances 49 -> 50. The seed-1/50-year demo does not originate autonomous loans because its scenario actors have no lendable cash, but the new monthly causal stage is replayed identically; accepted fingerprint: `9943765951260526877` at 76.1 ms/year.
- Next gate: give scenario actors explicit liquid portfolios and run a controlled long-history credit experiment. The content must produce at least one useful working-capital rescue and one refused or defaulted case without turning lender cash into a scripted bailout.

### Scenario credit portfolios and default closure gate (step 064)

- Content schema v8 gives owners explicit non-negative liquid cash and supports optional regional financiers as ordinary actors without ownership or management authority. Initial portfolios are evented, persisted, and fingerprinted.
- The demo now contains the Arcadian Working Capital Trust and two tiny, positive-margin grain producers. After three observations the first producer receives a six-unit secured loan against a four-unit funding gap plus its first payment reserve; the trust retains nine units of cash and reaches its portfolio concentration ceiling. The otherwise viable second producer is therefore refused without leaving an inadequate offer in expectations.
- The accepted borrower clears wage arrears, preserves its worker, and repays all six units of principal plus six units of rounded interest during 2026. The chronicle reports both the accepted working-capital loan and `1 of 2` viable searches ending without an acceptable domestic offer.
- Funding-gap viability now subtracts recurring payroll and target inputs, not legacy wage claims; arrears remain part of the one-month cash requirement. Lender-specific requests reserve the estimated first principal installment and interest payment before ordinary cash-flow, collateral, liquidity, and concentration caps apply.
- Distressed firms with two or three operating observations receive only enough grace to complete the four-observation credit review window. Matured unpaid debt independently accumulates distress, so a zero-workforce debtor without wage arrears reaches insolvency instead of becoming an immortal overdue claim.
- Focused coverage proves exact scenario acceptance/refusal, real lender cash, claim repayment, preserved employment, rejected-offer cleanup, service-reserve sizing, and debt-only insolvency. The full workspace gate passes 147 tests. ADR 0112 records the design.
- `SIMULATION_VERSION` advances 50 -> 51. Seed 1 over 50 years replays an identical chronicle and fingerprint `16300205677265857770` at 89.4 ms/year.
- Next gate: model lender income and realized credit losses as explicit actor portfolio history, so future rates and lending capacity respond to successful repayment and default rather than only current concentration.

### Realized lender track-record gate (step 065)

- Every creditor now accumulates explicit principal repaid, interest income, realized losses, successful resolutions, and defaults. Scheduled debt service records only real cash received; terminal liquidation splits recovery interest-first and classifies each resolved claim exactly once.
- `LenderCreditHistoryUpdated` exposes settlement deltas while cumulative history is serialized, inspectable, replayed through the existing command transitions, and included in stable fingerprints. The chronicle narrates annual lender recovery, income, successful loans, defaults, and losses.
- Portfolio appetite now starts at the existing 40% financial-wealth limit, gains 100 basis points per successful loan up to 500, and loses half the realized loss ratio up to 2,500 basis points. The result remains bounded to 15-45%, and the independent 50% current-cash reserve still binds.
- Pricing adds up to 3,000 basis points from realized loss ratio and grants 50 basis points per successful loan up to a 250-basis-point discount. Existing concentration, borrower-distress, cash-flow, collateral, liquidity, and minimum-coverage rules remain authoritative.
- The controlled demo trust closes 2026 with 6 principal recovered, 6 interest income, one successful loan, and zero losses. Focused regressions prove exact 40% -> 41% earned capacity, 40% -> 16% post-loss capacity, 600 -> 550 bps success pricing, 550 -> 2,050 bps loss pricing, liquidation loss attribution, event evidence, and fingerprint participation.
- The full workspace gate passes 148 tests. `SIMULATION_VERSION` advances 51 -> 52. Seed 1 over 50 years replays an identical chronicle and fingerprint `12342046951249449541` at 76.7 ms/year.
- Next gate: batch contemporaneous credit applications and let each lender allocate finite headroom by risk-adjusted return, rather than allowing canonical borrower order to consume the portfolio before later applications are compared.

### Batched risk-adjusted credit allocation gate (step 066)

- The monthly firm-credit market now collects every eligible application before any lender commits
  capital. Canonical firm order remains a replay tie-breaker but no longer decides who consumes a
  scarce portfolio first.
- Each lender provisionally underwrites the full domestic batch through the existing observed
  cash-flow, collateral, liquidity, concentration, and term gates. Finite headroom is then committed
  by a transparent risk-adjusted return: contractual rate, minus 400 bps per borrower-distress month,
  plus at most 500 bps for collateral above principal. High rates therefore cannot automatically
  make the weakest borrower the preferred allocation.
- Borrowers still accept the cheapest sufficiently large offer through the ordinary shared command;
  allocation and acceptance remain atomic and deterministically tie-broken. ADR 0114 records the rule.
- Focused coverage creates two simultaneous 100-unit gaps and only 120 units of lender headroom. The
  healthy later-id firm receives the 110-unit offer instead of the distressed earlier-id firm, and
  command replay reproduces state and fingerprint exactly. Full workspace tests pass with 149 tests.
- `SIMULATION_VERSION` advances 52 -> 53. The current demo's chronicle body remains materially
  unchanged; the seed-1 50-year fingerprint changes by version contract to
  `14936349298780613292`, with an identical self-comparison at 85.0 ms/year.
- Deliberate limit: committed headroom released by rejected offers is not re-auctioned in the same
  month. Next gate: add firm-level borrower credit histories for punctual service, delinquency,
  restructuring, and default so underwriting can price demonstrated borrower conduct rather than
  only current distress and collateral.

### Firm borrower credit-history gate (step 067)

- Firms now retain explicit borrower conduct: scheduled debt service due and paid, punctual and
  delinquent attempts, successful loan resolutions, and defaults. Updates come only from real debt
  service or terminal claim resolution and are journaled as typed evidence.
- Autonomous pricing now adds a bounded firm-specific adjustment: payment shortfall contributes up
  to 2,000 bps, delinquent attempts up to 1,500 bps, defaults up to 3,000 bps, and successful loans
  earn at most a 250-bps discount. Current distress, collateral, cash flow, lender liquidity,
  concentration, and lender track record remain independent constraints. ADR 0115 records the rule.
- Focused coverage proves a 50-of-100 partial payment moves an otherwise 600-bps offer to 1,700 bps,
  a later fully paid resolution rehabilitates it to 1,316 bps, and a default moves it to 2,316 bps.
  History counters and stable-fingerprint participation are exact.
- The history is serialized, replayed by existing service/liquidation commands, inspectable, and
  fingerprinted. `SIMULATION_VERSION` advances 53 -> 54. Seed 1 over 50 years self-compares
  identically with fingerprint `11122244119561874726` at 92.1 ms/year.
- Next gate: attribute loan purpose and realized operating outcome, so lenders can observe whether
  working capital preserved payroll and production rather than judging credit only by repayment.

### Government physical reserve gate (step 068)

- The roadmap now explicitly pauses further credit depth and begins the societal-adaptation arc.
- Political-office holders can buy domestic firm inventory into regional public reserves through the shared command boundary. Purchases move real treasury cash, seller cash, and physical stock atomically; rejected purchases change nothing.
- `ReserveRelease` uses stocked goods as in-kind survival aid after household clearing and before grievance, social stress, and health. Distribution reduces the same unmet-demand and deprivation state used by downstream consequences, so physically covered needs cannot still cause phantom deaths or foreign blame.
- Reserve stock is serialized, inspectable, replayed, and fingerprinted. Focused gates prove exact money/stock conservation, full shortage closure, and insufficient-treasury atomicity. ADR 0116 records the boundary. The full workspace gate passes 152 tests; `SIMULATION_VERSION` advances 54 -> 55, and the seed-1/50-year self-comparison is identical with fingerprint `4615256241328481519` at 78.1 ms/year.
- Deliberate limits: procurement is an explicit decision, domestic and immediate; reserves have no transport, spoilage, storage cost, coverage target, or autonomous proposal yet. Next gate: evidence-driven reserve procurement proposals based on observed survival shortage, treasury headroom, available stock, and target coverage.

### Observed government reserve procurement gate (step 069)

- The commercial cycle now converts residual survival unmet demand into a one-month regional reserve requirement after household settlement. Existing reserve stock is deducted before action.
- Under `ReserveRelease`, the canonical political-office holder buys only real post-market inventory from operating local firms that produce the required good. Quantity is bounded by need, stock, and treasury affordability; purchases use the existing atomic command and stable firm ordering.
- Procurement runs before reserve distribution, grievance, stress, and health, so retained producer stock can physically avert same-month harm without pre-empting ordinary buyers. The once-per-month stage is command-replayable, journaled, serialized, and fingerprinted. ADR 0117 records the rule.
- Focused coverage proves exact 1,000-unit procurement for 10 currency units, treasury/firm/reserve conservation, bit-identical replay, and duplicate-stage atomicity. The full workspace gate passes 153 tests; `SIMULATION_VERSION` advances 55 -> 56, and the seed-1/50-year self-comparison is identical with fingerprint `11094233980192924180` at 88.3 ms/year.
- Deliberate limits: one observed month of coverage, local reference-price purchases, no public transport, bids, storage cost, spoilage, corruption, or competing budget authorization. Next gate: explicit reserve coverage targets and carrying costs before interregional procurement.

### Reserve coverage, budget, and causal chronicle gate (step 070)

- Political-office policy now selects a one-to-twelve-month reserve coverage target and a zero-to-100% automatic procurement ceiling against opening monthly treasury. The ceiling is shared canonically across every competing regional good in the country, preventing each shortage from independently spending the full budget.
- Observed procurement compares current stock with shortage-derived target coverage, buys only the gap, and can retain physical inventory for a future month after the current emergency release. Seller stock, treasury cash, and the common command boundary remain authoritative.
- Every active `(region, good)` review now journals observed shortage, target stock, opening reserves, eligible local supply, available budget, purchased quantity, spending, remaining gap, and independent supply/budget constraint flags. Zero-purchase decisions are therefore evidence, not silence.
- The yearly chronicle explains why reserves were bought and issued: it reports reviewed targets, stock already held, local supply, uncovered gaps, binding constraints, attributed procurement spending, and cohort deliveries before grievance, stress, and health consequences.
- Focused coverage proves a retained second month of stock, one shared country budget across competing goods, exact supply-limited and budget-limited evidence, attributed chronicle prose, replay, and atomic rejection. The full workspace gate passes 157 tests. `SIMULATION_VERSION` advances 56 -> 57; two seed-1 50-year timelines produce identical chronicles and fingerprint `17917500942299006191` at 91.2 ms/year.
- Deliberate limits: targets extrapolate the currently observed shortage rather than forecasting seasonality; reserves still have no storage cost, spoilage, interregional public transport, supplier bidding, or corruption. Next gate: introduce bounded storage loss and carrying cost so excessive buffers compete honestly with treasury resilience before any cross-region procurement.

### Public reserve maintenance and spoilage gate (step 071)

- Reserve policy now configures independent monthly baseline spoilage and carrying cost. Cost is assessed against opening physical stock at the regional reference price; zero rates preserve the previous model.
- Maintenance runs at month opening, before payroll and commerce. Previously carried stock therefore costs and decays, while stock bought during the current shortage is not charged retroactively before its first use.
- Scarce treasury upkeep is shared proportionally across every reserve good in a country using deterministic largest remainders. Unpaid assessment produces additional neglect spoilage proportional to the funding gap, capped at 25% when upkeep receives nothing.
- Typed evidence records opening stock, reference value, assessed and paid cost, baseline loss, neglect loss, and closing stock for every maintained regional good. The shared command boundary replays the once-per-month stage atomically.
- The chronicle now distinguishes normal spoilage from stock destroyed by unfunded upkeep and reports the unpaid fiscal burden, turning reserve depth into a visible preparedness-versus-treasury tradeoff. Focused coverage proves funded upkeep, maximum neglect, proportional multi-good funding, next-month charging, command replay, duplicate-stage atomicity, and narration. The full workspace passes 161 tests. ADR 0119 records the design; `SIMULATION_VERSION` advances 57 -> 58. Two seed-1 50-year timelines produce identical chronicles and fingerprint `18007562743465462498` at 80.8 ms/year.
- Deliberate limits: reference prices proxy storage difficulty; there are no warehouses, stock-age lots, good-specific shelf lives, rotation sales, theft, or public transport. Next gate: let reserve policy adapt gradually to observed spoilage, repeated coverage gaps, and treasury stress rather than remaining fixed forever.

### Evidence-driven reserve policy adaptation gate (step 072)

- Every `ReserveRelease` country now reviews current-month procurement, distribution, and maintenance evidence after the commercial reserve response. Four serialized streaks distinguish recurring unbuffered shortage, explicit budget binding, unfunded upkeep, and idle spoilage.
- Doctrine changes only after persistence thresholds: three serviceable shortage months raise coverage by one; two budget-bound months raise monthly authority by 500 basis points; two neglect months reduce coverage by one (or authority at the one-month floor); three quiet spoilage months reduce coverage by one. Fiscal retrenchment has priority when signals conflict.
- Reviews are canonical, once-per-month, atomic, command-replayable, serialized, and fingerprinted. Typed evidence records physical totals, all streaks, and previous/new coverage and budget; completion evidence counts reviewed and changed countries.
- The chronicle names the leading country revision and distinguishes coverage expansion/retrenchment from procurement-authority expansion/retrenchment. Focused coverage proves three-month preparedness learning, two-month budget adaptation, upkeep-driven retrenchment, exact replay, duplicate-stage atomicity, and narration. ADR 0120 records the boundary; `SIMULATION_VERSION` advances 58 -> 59.
- Deliberate limits: this is bounded institutional learning rather than optimization; doctrine remains national rather than good-specific, and there are no forecasts, elections, legislative vetoes, storage investment, or public transport. Next gate: differentiated reserve priorities by good or region before interregional public logistics.

### Differentiated public-reserve priorities gate (step 073)

- Political-office holders can now assign a replayable zero-to-100% priority share to each regional good. Full priority preserves the national coverage target, partial priority scales it with deterministic upward rounding, and zero explicitly excludes new automatic stockpiling without destroying held reserves.
- Observed reserve requirements are reviewed from highest priority to lowest before stable country/region/good tie-breakers. The existing country-wide treasury ceiling remains binding, so prioritization reallocates scarcity rather than creating budget or inventory.
- Priority decisions validate country-region ownership, known goods, and real office authority. State, typed decisions, requirement evidence, save/replay, and stable fingerprints are authoritative; the chronicle explains when differentiated targets shaped allocation. ADR 0121 records the boundary.
- Focused coverage proves that a higher-priority medicine target receives a scarce shared budget ahead of lower-id food, while the food target is honestly scaled and journaled. The full workspace gate passes 165 tests. `SIMULATION_VERSION` advances 59 -> 60; two seed-1 50-year timelines produce the identical fingerprint `7948595364616626360`.
- Deliberate limits: priorities are explicit doctrine rather than learned proposals; reserve maintenance remains proportional, and stock cannot yet move between regions. Next gate: derive bounded priority proposals from observed regional vulnerability, import dependence, deaths, and repeated shortage before adding public interregional logistics.

### Evidence-driven regional reserve-priority adaptation gate (step 074)

- Every configured regional-good priority now retains separate streaks for serviceable but uncovered reserve gaps and for idle baseline spoilage. Three consecutive uncovered months raise priority by 500 basis points; six idle-spoilage months lower it by 500, with vulnerability taking precedence when signals conflict.
- Supply-limited shortages do not falsely raise priority. Adjustments remain bounded to 0-10,000 basis points and execute through the same authorized `SetGovernmentReservePriority` command used by explicit political decisions, so institutional learning cannot bypass authority or create resources.
- Pressure memory, reviews, resulting commands, typed evidence, save/replay, stable fingerprints, and yearly chronicle narration are authoritative. ADR 0122 records the boundary; `SIMULATION_VERSION` advances 60 -> 61.
- Focused coverage proves a half-priority target rises to 5,500 only after three consecutive uncovered months and that the complete decision replays bit-identically. Chronicle coverage names the leading country and region revision. The full workspace gate passes 167 tests; two seed-1 50-year timelines produce the identical fingerprint `6561286070422285131`.
- Deliberate limits: this first learning rule uses direct reserve performance. Cohort deaths are not yet attributable to individual goods, and delivered household import dependence is not retained as regional-good monthly evidence. Next gate: persist delivered household import dependence and let repeated reliance influence bounded priority proposals without treating all imports as failure.

### Observed household import-dependence gate (step 075)

- Atomic household market settlement now classifies fulfilled survival consumption by buyer region and good into local and delivered-import quantities. Monthly dependence state and typed evidence retain the physical totals and exact imported share; save/replay and stable fingerprints include the observation.
- Reserve-priority learning treats imports as vulnerability only after they supply at least 60% of fulfilled survival consumption for six consecutive months. Occasional or minority imports reset the streak and do not change doctrine, so efficient trade is not mislabeled as failure.
- Sustained high dependence raises a configured priority by one bounded 500-basis-point step through the ordinary authorized political command. Uncovered gaps retain first precedence, import reliance second, and idle-spoilage retrenchment third. Chronicle narration identifies import-driven revisions. ADR 0123 records the boundary; `SIMULATION_VERSION` advances 61 -> 62.
- Focused coverage proves a fully imported survival purchase is persisted and evented exactly, six high-reliance months move a half-priority target from 5,000 to 5,500, the revision reason is explicit, and command replay remains bit-identical. The full workspace gate passes 168 tests; two seed-1 50-year timelines produce the identical fingerprint `15123477875765043850`.
- Deliberate limits: import share does not yet distinguish diversified suppliers from a single fragile route or country. Next gate: measure supplier and route concentration so preparedness responds more strongly to concentrated dependence than to diversified trade before public interregional reserve logistics.

### Shortage-driven local firm-entry gate (step 076)

- Residual survival shortage is now aggregated by region and good after trade, rationing, and reserve release. Three consecutive shortage months create an entry opportunity; a cleared month resets the authoritative, fingerprinted streak.
- Feasible entry requires an existing output recipe, all observed local prices, an unemployed local worker, and a local actor able to fund installed capacity plus working capital. Missing technology, labor, price evidence, or founder liquidity leaves the shortage unresolved without creating resources.
- Startup economics are concrete and bounded: one batch of capacity costs two observed output-batch values; working capital covers one input batch and three wage months. The founder pays both, only working capital becomes firm cash, no opening inventory is minted, and production still needs ordinary inputs and labor.
- Founding uses the shared `FoundFirm` command and creates canonical ownership, chief-executive authority, one local employment agreement, a default policy, and a one-batch target atomically. Typed events and chronicle narration identify the founder, region, good, installed capital, working capital, and initial job. ADR 0124 records the boundary; `SIMULATION_VERSION` advances 62 -> 63.
- Focused coverage proves that one or two shortage months cannot create a firm, the third month produces a funded and staffed entrant with exact cash conservation across founder and working capital, insufficient founder cash blocks entry without mutation, and direct command execution replays bit-identically. The full workspace gate passes 171 tests; two seed-1 50-year timelines produce the identical fingerprint `10082682265911392485`.
- Deliberate limits: entry is one-batch and equity-funded; construction time, founder risk preferences, bank-funded startups, skill requirements, and competitive wage bidding remain future work. Next gate: labor matching in which entrants and incumbents compete for finite workers through observed wages and qualifications.

### Competitive opt-in labor-market gate (step 077)

- Recipes may now opt into competitive vacancy matching with an explicit minimum-education profile. Recipes that omit the profile retain legacy staffing behavior, preventing silent economic changes in existing scenarios and mods.
- Vacancies derive from authorized production targets and recipe labor intensity, are bounded by installed firm worker capacity, and subtract existing active agreements. Each solvent firm can hire at most one worker per monthly stage.
- Firms make cash-covered wage offers anchored to local cohort income and adjusted by production urgency. Scarce qualified workers accept the highest wage; stable firm and cohort identifiers resolve ties deterministically.
- Every accepted match revalidates vacancy, cash, locality, education, profile identity, and unallocated population through the shared command boundary. Typed completion and match events, save state, replay, and stable fingerprints are authoritative.
- Content schema v8 accepts optional `minimum_education` recipe values from `none` through `tertiary`. Shortage-driven entry keeps its historical Basic fallback while later vacancy matching remains opt-in. ADR 0125 records the compatibility boundary.
- Focused coverage proves wage competition for one worker, qualification rejection without mutation, direct/replay identity, schema loading, and legacy profile omission. The full workspace gate passes 174 tests; formatting, Clippy, and docs pass. `SIMULATION_VERSION` is 64. Two seed-1 50-year timelines remain identical with fingerprint `1189594396703659306` at 99.9 ms/year.
- Deliberate limits: employed workers do not search or switch jobs; offers have no persistence, contracts have no duration, and there is no unemployment/vacancy history, bargaining, training, migration, or labor-market chronicle. Next gate: retain regional labor-market evidence and use persistent vacancy and unemployment pressure for bounded wage adaptation before adding mobility or training.

### Persistent regional labor-market evidence gate (step 078)

- Every region with at least one explicitly configured competitive recipe now retains an authoritative post-matching observation: remaining unemployed workers, remaining target-derived vacancies, funded offers, accepted hires, and mean offered wage. Legacy-only regions remain absent.
- Consecutive unemployment pressure advances only when available workers exceed vacancies; vacancy pressure advances only when vacancies exceed available workers. Balance or reversal resets the corresponding bounded streak, distinguishing durable surplus from durable scarcity.
- Automatic matching now treats only cohorts explicitly marked unemployed as active job seekers, avoiding silent recruitment from employed aggregate cohorts. Observations are serialized, fingerprinted, inspectable, and emitted as typed regional evidence.
- The yearly chronicle aggregates labor offers and hires and names the regions with the longest unemployment or unfilled-vacancy pressure. ADR 0126 records the evidence boundary.
- Focused coverage proves one-worker competition leaves one residual vacancy, zero-demand regions accumulate unemployment pressure across months, direct/replay state remains identical, and chronicle attribution names persistent scarcity. The full workspace gate passes 176 tests; formatting, Clippy, docs, and `git diff --check` pass. `SIMULATION_VERSION` advances 64 -> 65.
- Two seed-1 50-year timelines remain identical with fingerprint `6342305687305279707` at 102.9 ms/year. The demo has no opted-in recipes, so its economic and chronicle path remains behaviorally unchanged apart from the deliberate simulation-version fingerprint contract.
- Deliberate limits: evidence is regional rather than occupation-specific and does not yet alter wages. Next gate: let firms adapt offered wages gradually after persistent unfilled vacancies while persistent unemployment restrains bids, without bypassing cash coverage or payroll risk.

### Bounded evidence-driven wage-adaptation gate (step 079)

- Competitive bids now react only to persistent, previously observed regional pressure. Every three residual-vacancy months add 500 basis points, capped at +2,000; every three residual-unemployment months subtract 250 basis points, capped at -1,000. One- or two-month noise changes nothing.
- The adjustment composes with the existing local-income anchor and production-urgency premium. It cannot mint cash, bypass qualification, reserve future payroll, or force a hire; firms must still cover the complete adapted wage from current cash.
- Replayable employment matches carry the signed pressure adjustment. Direct execution re-derives both wage and adjustment and rejects stale or fabricated bids atomically. Typed match evidence and yearly chronicle narration distinguish scarcity-raised from surplus-restrained accepted wages. ADR 0127 records the rule.
- Focused coverage proves three scarcity months move the representative bid from 120 to 125 with +500 basis points, while three surplus months restrain it to 118 with -250 basis points. Existing no-pressure competition remains unchanged. The full workspace gate passes 177 tests; formatting, Clippy, docs, release build, and `git diff --check` pass. `SIMULATION_VERSION` advances 65 -> 66.
- Two seed-1 50-year timelines remain identical with fingerprint `9534485292765561832` at 102.8 ms/year. Because the demo has no opted-in competitive recipes, the economic chronicle remains behaviorally unchanged.
- Deliberate limits: only new hires receive adapted bids; existing contracts do not renegotiate and workers do not switch employers. Next gate: permit bounded employed-worker search and voluntary job switching when a competing offer materially exceeds the current wage, while preserving firm capacity, notice, and one-worker-per-firm monthly limits.

### Bounded voluntary employment-switching gate (step 080)

- After unemployment matching, configured firms with residual vacancies may recruit one qualified worker from an active local agreement only when the deterministic funded offer is at least 10% above the current wage.
- Ordinary job seekers retain priority. A firm may participate in only one monthly labor transition as hiring destination, switching destination, or source, preventing same-month chains and oscillation. Source staffing falls exactly as destination staffing rises; source arrears remain enforceable.
- Replayable switch commands re-derive vacancy, locality, qualification, cash coverage, current source wage, material gain, and pressure-adjusted target bid. Typed events and yearly chronicle evidence retain the old wage, new wage, source, destination, and aggregate worker gain. ADR 0128 records the boundary.
- Focused coverage proves a worker moves from 100 to a funded 113 offer, direct/replay state and fingerprints are identical, and monthly completion distinguishes ordinary matches from switches. The full workspace gate passes 178 tests; formatting, Clippy, docs, release build, and `git diff --check` pass. `SIMULATION_VERSION` advances 66 -> 67.
- Two seed-1 50-year timelines remain identical with fingerprint `9855621930944341265` at 99.6 ms/year. The demo remains behaviorally unchanged because competitive labor profiles are opt-in.
- Deliberate limits: switching has no tenure friction, counteroffers, notice periods, or non-wage preferences. Next gate: add bounded employment tenure and switch cooldown before introducing qualification-specific evidence and training.

### Employment-tenure stability gate (step 081)

- Every agreement now retains bounded months at the current firm. New and reactivated agreements begin at zero, active tenure advances once per completed labor-market month, and inactive agreements do not age.
- Voluntary switching requires three completed source-firm months. The destination tenure resets, creating a deterministic cooldown without a separate timer; the one-transition-per-firm monthly rule still prevents chains. Tenure is serialized and fingerprinted. ADR 0129 records the boundary.
- Focused coverage proves the worker remains with the incumbent for three markets, then moves on the fourth when the pressure-adjusted offer is materially better. Direct and replayed tenure, switch state, and fingerprints remain equal. `SIMULATION_VERSION` advances 67 -> 68.
- Deliberate limits: firms do not make counteroffers and employment has no notice period or non-wage utility. Next gate: separate evidence by demanded qualification so specialist scarcity does not distort every wage.

### Qualification-specific labor-evidence gate (step 082)

- Each configured region now retains one authoritative labor observation per exact recipe education floor: qualified unallocated supply, exact-threshold vacancies, and bounded surplus/scarcity streaks.
- Higher-educated workers count as qualified supply for lower thresholds, while every vacancy belongs to one exact demand bucket. Cohorts currently in training are excluded from available supply.
- Competitive wage adaptation now reads the target recipe's qualification bucket rather than aggregate regional pressure. Typed events, save state, stable fingerprints, getters, and yearly chronicle attribution expose the evidence. ADR 0130 records the semantics. `SIMULATION_VERSION` advances 68 -> 69.
- Deliberate limits: education floors still aggregate occupations that share a credential level. Next gate: allow persistent skill scarcity to expand qualified supply through costly, delayed training.

### Household-funded workforce-training gate (step 083)

- Six consecutive months of qualification-specific vacancy pressure can start at most one local program per region and month for an unemployed, unallocated, underqualified cohort with sufficient savings.
- Training costs one month of aggregate baseline income, with a positive floor, debits real household wealth, lasts three completed labor-market months, and advances education by exactly one adjacent level. Trainees cannot be hired until the month after completion.
- Active programs are serialized and fingerprinted. Typed start/completion events and yearly chronicle narration expose enrollment, completion, skill shortage, and tuition. ADR 0131 records the mechanism.
- Focused coverage proves a Secondary cohort pays 100, remains unavailable for three months, completes Vocational training, and replays with identical state and fingerprint. The combined step 080-083 workspace gate passes 179 tests; formatting, Clippy, docs, release build, and `git diff --check` pass. `SIMULATION_VERSION` advances 69 -> 70.
- Two seed-1 50-year timelines remain identical with fingerprint `9052706876239142019` at 103.8 ms/year. The demo remains behaviorally unchanged because competitive labor profiles are opt-in.
- Deliberate limits: training capacity is regional and household-funded; employer sponsorship, public education budgets, dropout risk, cohort splitting, and occupation-specific curricula remain future work. Next gate: introduce bounded incumbent counteroffers that must pass forward payroll-solvency checks.
### Steps 084-087 complete

Labor mobility now includes solvent retention offers, household-scale training participation, employer tuition sponsorship, and content-defined occupation skill gates. The next labor iteration should turn the new occupation identity into persistent occupation-specific evidence and specialization-producing curricula, then expose those signals in the chronicle.
### Steps 088-090 complete

The simulation now spans household lifecycle transitions, durable but bounded consumer-supplier relationships, and fiscally grounded regional public-service capacity. Step 091 remains the integration gate: partial-cohort migration should react to employment opportunity, household resources, service quality, and destination pressure without creating population or money.

### Resource-bounded internal migration gate (step 091)

- Annual internal migration now links six-month labor-market evidence, household liquidity, and regional public-service quality. At most one eligible household partition leaves each persistently unemployed source for a persistently vacancy-constrained destination in the same country.
- Cohorts split instead of teleporting wholesale. Population, households, annual income, liquid wealth, and debt are conserved exactly; persistent health, experience, stress, deprivation, and skill state follows the migrant household. Migration creates no job, so ordinary destination matching still determines employment.
- A one-month proportional income reserve makes mobility resource-bounded, while a 1,000-basis-point service disadvantage veto prevents households from chasing vacancies into dramatically weaker institutions. Deterministic opportunity ranking and cohort selection keep replay stable.
- Typed `HouseholdMigrated` evidence records origin, destination, people, wealth, debt, vacancy persistence, and service advantage. ADR 0139 records the boundary; `SIMULATION_VERSION` advances 77 -> 78. Focused gates prove household-scale movement, exact population/wealth/debt conservation, and rejection under weak evidence or insufficient liquidity. The full workspace gate passes 187 tests plus formatting, Clippy, docs, release build, and `git diff --check`; two seed-1 50-year timelines are identical with fingerprint `347025067225583205` at 119.6 ms/year.
- Deliberate limits: domestic movement only; no housing capacity, moving-industry payment, family network, international law, or guaranteed job. Next gate: destination housing pressure and a real relocation-service payment, so rapid inflows raise costs and create a second balancing loop.

### Housing-constrained paid migration gate (step 092)

- Every region now has persistent, fingerprinted dwelling capacity and a positive baseline monthly housing cost. Legacy scenarios derive conservative defaults from initial population and output, while explicit scenario overrides are supported through the core API.
- Migration evaluates projected destination occupancy. A full housing market blocks the move; rising pressure reduces the opportunity score and raises the relocation fee from 50% to 150% of baseline monthly cost.
- Migrants must afford both their one-month income reserve and the separate fee. The fee moves real liquid wealth into the destination treasury after fiscal closure, so relocation does not destroy or create money and the receiving state gains auditable service revenue.
- `HouseholdMigrated` now records the paid fee and projected housing pressure. Focused gates prove exact population, debt, and combined household-plus-treasury money conservation; they also prove rejection under full housing, weak labor evidence, and insufficient liquidity. ADR 0140 records the boundary; `SIMULATION_VERSION` advances 78 -> 79. The seed-1 50-year self-comparison is identical with fingerprint `12128517368621254933` at 108.6 ms/year.
- Deliberate limits: aggregate dwellings, public relocation provider, no recurring rent or construction. Next gate: pressure- and revenue-funded housing construction with a construction delay, so destination capacity can respond without appearing for free.

### Pressure-funded delayed housing construction gate (step 093)

- Regions at or above 90% housing occupancy can now start one public housing project when their country can fund it from current treasury cash. Canonical region order makes competing projects deterministic.
- Each project adds 10% of existing dwelling capacity, rounded up, and costs twelve baseline monthly housing payments per dwelling. The complete cost is committed at authorization; capacity appears only after two later annual advances.
- Active construction and completed public housing capital are serialized and fingerprinted. Typed start/completion evidence records pressure, spending, delay, delivered dwellings, and resulting capacity. Insufficient treasury leaves both cash and capacity unchanged.
- This closes the first settlement adaptation loop: migration raises occupancy and public fee revenue; crowding raises costs or blocks entry; a solvent state can then fund delayed capacity instead of receiving free housing. ADR 0141 records the boundary; `SIMULATION_VERSION` advances 79 -> 80. The final seed-1 50-year audit remains self-identical at fingerprint `10594705336716621082`; after chronicle projection work it runs at 99.4 ms/year.
- Deliberate limits: public aggregate construction without contractors, materials, land, recurring rent, maintenance, or zoning. Next gate: narrate migration and housing projects in the chronicle, then route construction spending through concrete firms and labor.

### Settlement arc chronicle gate (step 094)

- The yearly chronicle now names migration origins and destinations and reports households, people, relocation fees, and projected destination housing pressure directly from typed events.
- Public housing authorization and completion are narrated separately, preserving the causal delay between committed treasury funds and delivered dwellings. Settlement-only years receive importance 68: above routine production, below emergency relief and severe institutional crises.
- The projection reads only the event journal and region-registration names; no simulation rule or state changed. ADR 0142 records the boundary, and `SIMULATION_VERSION` remains 80. Focused narration coverage raises the full workspace total to 190 tests.
- Next gate: route housing construction spending through concrete construction firms, material inputs, and payroll so public capacity competes for cement, labor, and logistics instead of transforming treasury cash directly into committed capital.

### Material social pressure and legitimacy gate (step 095)

- Regional political pressure now comes from concrete lived evidence: chronic unemployment duration, persistent deprivation, debt distress, and the shortfall in delivered healthcare, infrastructure, and administration. It is stored in authoritative state and included in stable fingerprints.
- Cohort evidence is population-weighted within regions, and regional pressure is population-weighted within countries. Large regions therefore matter because more people are affected, not because their IDs or number of administrative units happen to differ.
- A bounded annual legitimacy signal is centered on 3,000 basis points of pressure: stable conditions can restore at most 150 basis points, while maximum material pressure can remove 350. Existing growth, fiscal, random, and sovereign-crisis signals remain independent.
- Typed regional and country evidence makes the causal chain auditable. The chronicle names the leading region, decomposes pressure, and reports whether it supported or reduced national legitimacy. ADR 0143 records the boundary; `SIMULATION_VERSION` advances 80 -> 81. Focused composition, weighting, legitimacy-bound, and narration coverage raises the full workspace total to 194 tests. The seed-1 50-year release audit is self-identical at fingerprint `7833490598214856219` and 102.2 ms/year.
- Deliberate limits: one national legitimacy stock, no constituencies, ideology, elections, protest organizations, or policy preferences. Next gate: persistent regional interests and policy-specific winners/losers, without prematurely building a full electoral simulator.

### Persistent regional interests and fiscal incidence gate (step 096)

- Regions now retain one leading material policy interest: employment, household security, public services, or stability. A 750-basis-point switching margin prevents noisy annual reversals, while materially pressured regions can leave baseline stability immediately.
- Regional satisfaction responds to improvement or deterioration in the incumbent concern and to observable fiscal incidence. Firm sales taxes are assigned to the paying firm's region; recurring service budgets are assigned to the receiving region. Their difference identifies net contributors and beneficiaries without pretending to model unlocated spending.
- Interests, satisfaction, persistence, and annual fiscal outcomes are authoritative and fingerprinted. Typed evidence and the chronicle identify policy priorities, regional winners and losers, and concrete net transfers. ADR 0144 records the boundary; `SIMULATION_VERSION` advances 81 -> 82.
- Deliberate limits: the ledger covers only physically attributable sales tax and public-service allocation. No ideology, parties, elections, representation, or inferred incidence for debt service and national spending.

### Delayed regional confidence legitimacy gate (step 097)

- Prior-year regional satisfaction is now population-weighted into national confidence before annual political closure. Confidence contributes a separate bounded legitimacy effect from -200 to +200 basis points around a neutral 5,000.
- Current fiscal outcomes update satisfaction only after politics closes, so a government inherits confidence produced by earlier outcomes rather than receiving an instantaneous same-year reward. Immediate social pressure, growth, fiscal balance, random variation, and sovereign crises remain independent signals.
- Typed evidence and chronicle narration expose the exact confidence and legitimacy effect. Focused priority hysteresis, fiscal winner/loser, event-derived incidence, population weighting, bound, and narration tests bring the complete workspace to 200 tests. ADR 0145 records the boundary; `SIMULATION_VERSION` advances 82 -> 83. The seed-1 50-year release audit is self-identical at fingerprint `15365365975592083424` and 121.5 ms/year.
- Next gate: let persistent regional interests influence bounded public-service allocation decisions through existing political offices and influence weights, while preserving treasury constraints and avoiding a full electoral simulator.

### Unrestricted authorized service allocation gate (step 098)

- A political-office holder can now set exact regional shares of the discretionary public-service budget through the same replayable command used by autonomous actors. Shares must conserve 10,000 basis points, but zero shares and a full 100% assignment to one region are deliberately legal. Omitted domestic regions receive zero.
- Engine validation covers only authority, country ownership, and exact accounting. There is no universal fairness floor, concentration cap, or gradualism rule. Unauthorized and foreign-region decisions fail atomically; extreme authorized decisions execute and produce consequences.
- Without an explicit decision, the autonomous planner uses a prudent behavior policy based on population, service need, persistent public-service interests, and low satisfaction. This policy does not narrow the engine action space and is completely replaced by an explicit command.
- Largest-remainder settlement conserves every minor currency unit. Actual regional funding now determines the local service target, while administrative capacity still limits delivery. Typed evidence and chronicle narration distinguish autonomous from explicit allocation and expose extreme concentration.
- ADR 0146 records the boundary and its compliance with the `AGENTS.md` simulation freedom principle. `SIMULATION_VERSION` advances 83 -> 84; authority, atomicity, 100% concentration, prudent AI, money conservation, and narration coverage raise the complete workspace total to 206 tests. The seed-1 50-year release audit is self-identical at fingerprint `6845839973059341690` and 114.1 ms/year.
- Next gate: allow existing political influence relationships and actor policy traits to bias the autonomous allocation choice, without restricting explicit player commands or creating resources.

### Power-network-biased autonomous service allocation gate (step 099)

- Autonomous regional service allocation now reads the existing political graph instead of treating distributive policy as socially anonymous. A political-office holder contributes a 1,000-point home-region preference; established influence into that office contributes half its basis-point weight toward the influencing actor's home region.
- Political bonuses join population, service need, persistent interest, and low satisfaction before deterministic share normalization. Only domestic home regions and political offices of the allocating country participate.
- This remains AI behavior rather than engine law: an explicit authorized allocation bypasses every autonomous political bonus and may still assign any feasible 0-100% shares. Influence cannot veto player commands, expand the spending envelope, or evade delivery capacity.
- Typed evidence identifies the actor, office, favored region, mechanism, source weight, and exact score bonus. Chronicle narration names the strongest actor-region pressure behind the autonomous choice.
- ADR 0147 records the boundary. `SIMULATION_VERSION` advances 84 -> 85; influence causality and actor-attributed narration coverage raise the complete workspace total to 208 tests. The seed-1 50-year release audit is self-identical at fingerprint `9915252302055461441` and 122.9 ms/year.
- Next gate: feed realized allocation bias into elite cohesion and regional satisfaction as separate consequences, so patronage may secure one base while alienating excluded regions and other power centers.

### The Governing State Becomes Visible milestone (step 100)

- Step 100 is now an anniversary sequence, 100.1-100.10, joining persistent government programs and political consequences to the first Bevy world client. The complete sequence and release definition live in `docs/design/step-100-the-governing-state-observatory.md`.
- The ordering is intentional: core program identity, appropriation, contested physical execution, winners/losers, political memory, and chronicle projection precede the viewer, so the graphical client exposes real causality rather than owning or mocking simulation state.
- Step 100.1 establishes the charter boundary. `GovernmentProgram` is persistent, serialized, inspectable, replayable, evented, and fingerprinted, with a political initiator, exact regional shares, annual funding promise, duration, service priority, promised improvement, lifecycle status, appropriation/delivery/carryover ledgers, and delay memory.
- Announcement is separate from execution. An authorized office holder may promise more than the current treasury can fund; declaration creates no cash, appropriation, or services. Invalid accounting, unknown or foreign regions, past scheduling, duplicate identity, and absent authority remain atomic rejections.
- ADR 0148 records the decision. `SIMULATION_VERSION` advances 85 -> 86. Focused tests prove legal over-promising, zero opening execution ledgers, command replay, fingerprint identity, and unauthorized atomicity.
- Next gate (100.2): annual appropriation and competing-program budget allocation, including explicit zero funding, concentration, cancellation, and debt proposals under real authority and resource limits.

### Government-program appropriation gate (step 100.2)

- Authorized actors can now make one replayable annual appropriation decision per scheduled program. The decision may fully fund, underfund, or explicitly assign zero; the original promise remains unchanged and the shortfall is typed evidence.
- Treasury appropriations remove exact existing cash into program carryover. Public-debt appropriations raise debt directly under the existing output-based sovereign headroom and do not create free uncommitted treasury cash. Multiple programs therefore compete for real fiscal capacity.
- Cancellation is an explicit command that stops future appropriations without erasing already committed carryover. Terminal, duplicate-year, out-of-window, unauthorized, over-treasury, over-debt, and negative decisions reject atomically.
- ADR 0149 records the boundary. `SIMULATION_VERSION` advances 86 -> 87. Focused tests prove treasury conservation, debt accounting, legal zero funding, cancellation, replay, and fingerprint identity.

### Administratively bounded program-execution gate (step 100.3a)

- A current-year appropriation can now execute once through the shared command boundary. Exact charter shares allocate committed carryover to regions before capacity constraints.
- Existing regional administration creates a concrete 25-100% absorption rate. Only absorbed money becomes delivered funding and improves the declared service priority; the remainder persists as carryover and increments delay memory.
- Regional typed evidence records share, commitment, delivery, administrative absorption, improvement, and priority. Aggregate evidence records opening carryover, delivered total, remaining carryover, and accumulated delay.
- The controlled default-administration test commits 1,000, delivers 624, retains 376, and records one delayed year; duplicate execution rejects without mutation and replay is identical.
- ADR 0150 records the boundary. `SIMULATION_VERSION` advances 87 -> 88. This is the first execution gate, not the end of 100.3: next add goods, labor, infrastructure, logistics, and political support/friction before calling contested physical execution complete.

### Power-network-contested program execution gate (step 100.3b)

- Government-program execution now reads established influence into domestic political offices. Each influencing actor compares the program share received by their home region with an equal domestic reference share.
- Favored power bases add bounded execution support; underrepresented or excluded bases add bounded political friction. Existing influence weight scales the effect, and the combined program modifier is clamped to ±2,000 basis points before composing with regional administrative absorption.
- Opposition never silently vetoes an authorized command. It reduces realized delivery, leaves more carryover, and lengthens visible delay; support can accelerate delivery but cannot exceed 100% absorption or create money.
- Typed evidence identifies actor, office, home region, support/opposition stance, influence weight, actual share, fair reference, and exact execution modifier. Focused coverage proves a favored 8,000-bps patron raises delivery above the neutral 624 baseline while excluding the same base lowers delivery below it.
- ADR 0151 records the rule. `SIMULATION_VERSION` advances 88 -> 89. Next 100.3 gate: material inputs, temporary public-project labor, and logistics so political and administrative capacity are joined by consumed physical resources.

### Materially bounded program execution gate (step 100.3c)

- Program charters may now declare positive annual requirements for concrete goods. Required goods are validated at declaration and the requirement map is serialized, inspectable, replayable, and fingerprinted. Programs without material requirements retain their previous behavior.
- Each requirement is split by the charter’s exact regional shares. Local public-reserve stock determines a material-availability ratio; the scarcest complementary good binds final execution alongside administration and political support/friction.
- Realized execution consumes the corresponding physical quantity from the receiving region’s reserve. Stock elsewhere cannot teleport, and money cannot substitute for missing material within the execution command. Typed events identify program, country, region, good, and consumed quantity.
- Focused coverage gives both beneficiary regions exactly half their required construction material. Delivery is therefore capped at 500 rather than the neutral administrative 624, all 500 units are physically consumed, 500 funding remains as carryover, and no stock or money is invented.
- ADR 0152 records the boundary. `SIMULATION_VERSION` advances 89 -> 90. Next gate: temporary public-project labor and then explicit interregional public logistics for moving program materials.

### Resource-bounded temporary program-labor gate (step 100.3d)

- Program charters may now declare annual temporary-worker requirements. Exact regional shares split demand, and only unemployed local population not already committed to another program in the same year can participate.
- A serialized and fingerprinted `(year, region)` usage ledger prevents competing programs from double-using the same labor pool. Labor availability joins administration, infrastructure, political support/friction, and materials as a binding execution ratio.
- Realized spending for labor-bearing programs moves into household liquid wealth as temporary wages. Durable employment status is unchanged because this is bounded project participation rather than a permanent job. Typed evidence records program, country, region, workers, and wages.
- Infrastructure is now an independent execution ceiling. Focused coverage proves scarce workers reduce delivery below the neutral 624 baseline, receive real wages, and leave authoritative annual usage evidence.
- ADR 0153 records the boundary. `SIMULATION_VERSION` advances 90 -> 91. The first complete 100.3 vertical slice now has finance, administration, infrastructure, politics, local physical materials, labor, carryover, and delay. Next gate: 100.4 persistent winners, losers, and broken-promise memory.

### Persistent regional program-memory gate (step 100.4a)

- Every annual execution now records cumulative regional promised, committed, and delivered funding, current fulfillment, outcome type, consecutive excluded years, and bounded political memory for every domestic region.
- Outcomes distinguish beneficiaries, underfulfilled promised regions, and explicit zero-share exclusion. Promise failure is intentionally harsher than receiving no promise: a fully broken non-zero promise shifts memory by -300, while explicit exclusion shifts it by -100; full delivery rewards +150.
- Typed events retain the complete causal evidence and incremental shift. Program memory is serialized, inspectable, replayable, and fingerprinted. Focused coverage proves a capital-only charter creates different durable memories for the funded and excluded regions.
- ADR 0154 records the boundary. `SIMULATION_VERSION` advances 91 -> 92. Next gate: apply this stored evidence once per year to satisfaction, confidence, legitimacy, and elite cohesion.

### Annual program political-consequence gate (step 100.4b)

- Program memory now becomes causal once during annual regional-interest closure. Only programs executed in the closing year contribute, preventing command-order effects and repeated application.
- Regional satisfaction receives the bounded sum of local fulfillment, broken-promise, and exclusion shifts. Country legitimacy receives the population-weighted national result. Elite cohesion also pays a distinct polarization penalty for the spread between the best- and worst-treated regions.
- Typed evidence retains execution year, national average, polarization, indicator shifts, and final legitimacy/cohesion. Focused coverage proves an unfunded unequal promise lowers satisfaction in both the promised and excluded regions while also reducing legitimacy and cohesion.
- ADR 0155 records the boundary. `SIMULATION_VERSION` advances 92 -> 93. Step 100.4 is complete as a political-memory vertical slice; next gate is 100.5, the program chronicle.

### Government-program chronicle gate (step 100.5)

- The deterministic event-derived chronicle now narrates program declarations, appropriations, cancellations, delivery, carryover, accumulated delay, material consumption, temporary labor and wages, regional winners/underfulfillment/exclusion, the clearest regional loser, polarization, legitimacy, and elite-cohesion consequences.
- Program names are learned from declaration evidence and remain available to later annual entries. Political activity can produce a chronicle entry even before an ordinary economic-year closure; narration remains read-only and never becomes a second simulation.
- Focused coverage proves the chronicle names the program, identifies a regional loser, and reports polarization from the same replayable evidence used by the simulation.
- ADR 0156 records the boundary. `SIMULATION_VERSION` advances 93 -> 94. Steps 100.1-100.5 complete the authoritative simulation-side governing-state loop; next gate: 100.6, an independent Bevy observatory foundation.

### Independent Bevy observatory foundation (step 100.6)

- Added the `adam-observatory` workspace application on Bevy 0.16. The dependency direction remains one-way: the client reads `adam-core`; the authoritative simulation knows nothing about Bevy.
- A read-only deterministic snapshot exposes date, fingerprint, canonical regions, population, output, confidence, and chronicle entries. Stable region order produces deterministic non-overlapping map coordinates, proven by focused tests.
- The executable opens the first A.D.A.M window and renders a dark regional map with confidence coloring and population/output labels. Bevy transitive dependencies are locked to versions compatible with the workspace Rust 1.85 MSRV.
- ADR 0157 records the boundary. Presentation does not advance `SIMULATION_VERSION`. Next gate: 100.7 interactive inspector and overlays.

### Interactive region inspector and overlays (step 100.7)

- The Bevy observatory now has client-owned selection state and switchable confidence, population, and annual-output overlays. `[1]`, `[2]`, and `[3]` change layers; `Tab` cycles regions in canonical snapshot order.
- A fixed inspector shows the selected region’s identity, country, population, output, and confidence. The selected map card is highlighted, while all color ratios are bounded and safe for empty or zero-valued worlds.
- Interaction remains presentation-only: it changes no core state, event, save, or fingerprint. ADR 0158 records the boundary. Next gate: 100.8, a command-backed playable program desk.

### Command-backed playable program desk (step 100.8)

- Added an observatory content scenario with a legitimate cabinet, political office, and announced national-renewal program. The Bevy client now retains the authoritative `World`; all post-startup decisions use existing `WorldCommand` variants rather than direct mutation.
- `[T]` requests treasury appropriation, `[D]` debt appropriation, `[E]` execution, `[C]` cancellation, and `[Y]` an annual advance. Accepted commands rebuild the immutable snapshot; rejected commands preserve state and display the domain error.
- The desk displays program status, promise, cumulative appropriation, delivery, carryover, and delayed years. Focused coverage proves an authorized command changes the core fingerprint before the refreshed snapshot exposes the transition.
- ADR 0159 records the boundary. Presentation/content setup does not advance `SIMULATION_VERSION`. Next gate: 100.9 graphical political timeline.

### Graphical political timeline (step 100.9)

- The observatory now renders the four latest authoritative chronicle entries with year and importance. Promise, funding, delivery, regional losers, polarization, and legitimacy language comes from the shared core chronicle rather than Bevy-side event interpretation.
- The timeline refreshes only after successful commands rebuild the snapshot; rejection cannot create history. Empty worlds receive an explicit empty state. Focused coverage proves a command-created program cancellation appears in the captured political chronology.
- ADR 0160 records the boundary. Next gate: 100.10 anniversary integration and release audit.

### Step 100 anniversary acceptance gate (step 100.10)

- Accepted the complete promise-to-consequence vertical and the first playable Bevy governing-state observatory. An end-to-end acceptance test applies a program command sequence directly and by replay, then proves equality of both authoritative fingerprints and complete graphical snapshots.
- The full workspace release gate covers tests, warning-free Clippy, formatting, documentation, optimized compilation, and diff hygiene. ADR 0161 and `docs/release/step-100-anniversary-acceptance.md` record scope, controls, and the explicit distinction between the accepted engineering visual baseline and future final art direction.
- **Step 100 — The Governing State Becomes Visible: accepted on Windows only, later corrected.** `SIMULATION_VERSION` remains 94.
- Correction: this gate was recorded as complete while `cargo check --workspace --all-targets` failed on Linux CI. The observatory enabled `bevy_winit` with Bevy default features disabled and without `x11`/`wayland`, so the workspace could not compile on the only platform CI runs. The acceptance was therefore not valid when it was written; the build fix is recorded in ADR 0169. A green cross-platform CI run is a precondition for every future completion claim.


### Defects confirmed by external review

- **FIXED (ADR 0170). The demo economy was structurally insolvent on every seed.** Seeds 1-5 over 50 years all ended at ~266-271 people with zero output. This was not stochastic variance: each firm ran a fixed monthly loss and died exactly when its starting cash was exhausted. After the fiscal circuit was closed, Southvale sustains 144000 of output against 144000 of final consumption for roughly four decades with a growing population. The worlds still end, but from causes that are now visible rather than masked by a monetary leak: static prices, and demography modelled as a rate instead of a process.
- **FIXED (ADR 0170). The root cause was a missing fiscal circuit, not content tuning.** The 20% final-sales tax left firm revenue and never returned as income or demand, so no price vector could satisfy firm solvency (0.8*P*Q >= wages + inputs) and household solvency (wages >= P*Q) at once. A price-raising experiment confirmed this by making the collapse faster, and was reverted. Executed public spending now reaches household cohorts in the same annual closure that records it, split by population without rounding loss. Balance tuning remains deferred until the economic simulation is complete.
- **Chronicle ranking is inert.** `importance` is almost always 100, so the chronicle reads as an accounting log rather than history. The prerequisite is an operational definition of a reportable event, such as deviation from expectation, rather than narrating every computed quantity.

### Step A1: the fiscal circuit closes

- Taxation now moves money instead of destroying it. Collected sales tax equals recorded revenue, and every minor unit of executed public spending is credited to household cohorts as public wages, transfers, and coupon on domestic debt.
- The governing invariant is `firm cash + household liquid wealth + treasury = initial money stock + public debt`: money is created only by public borrowing. A dedicated gate, `crates/adam-core/tests/fiscal_circuit.rs`, proves revenue equality, full delivery, treasury/debt closure, and rounding-free population splits.
- The old assertion in `intermediate_procurement.rs` added the vanished tax back by hand so the books would appear to balance; it encoded the leak it should have caught, and now states the correct invariant.
- Each payment emits a typed `PublicOutlayDistributed { country, cohort, amount }` event, so the causal chain runs between named agents rather than between balances. ADR 0170 records the boundary and `SIMULATION_VERSION` 96.
- Remaining measured defects, in priority order: prices are static and cannot clear a shortage; demography is a rate, not a process, so no one is ever born.

### Step B1: prices form themselves, and no bound may be unowned

- Regional reference prices are now revised every settled month from the sellers' own books. A market that sold out with demand still unfilled raises its price in proportion to the share of demand that walked away empty; a market holding unsold stock cuts its price whether or not needs went unmet beside it; a market that cleared exactly is left alone. Each revision emits a typed `RegionalPriceAdjusted` event carrying the offered, sold, unsold, and unfilled quantities, the cost floor, and whether the floor bound the result.
- Two failed attempts are recorded in ADR 0171 because both were instructive. Raising a price on unfilled demand alone made relief self-defeating: a cohort given emergency money met a higher price the next month. Bounding the monthly move to 3% was worse: paired with "always move at least one minor unit" on an integer price grid it produced 20% monthly moves while claiming 3%, taking grain from 5 to 16 in a year with the physical economy unchanged.
- The move is now unbounded. Two limits survive, and both name who pays: the variable-cost floor, because the producer loses money on every unit sold below it and refuses; and the fractional carry, which conserves sub-unit pressure in thousandths because currency is an integer. Carried pressure is world state and enters the deterministic fingerprint.
- **New binding project rule, recorded in `AGENTS.md` as No unowned constraints:** every limit in the economy must name the party that pays for it in money, health, reputation, forgone profit, or a broken contract. If nobody pays, the limit is magic and must be deleted rather than tuned. Divergence from real life is evidence of an unfinished causal loop, never a licence to bound a sound rule.
- Three tests that asserted fixed prices as constants were rewritten as invariants. Inventory investment is still required to be valued at observed transaction prices, but the test now derives that from the live prices instead of hard-coding the answer. ADR 0171 records the boundary and `SIMULATION_VERSION` 97.
- **Measured consequence, deliberately undamped:** in the two-firm chain the farm sells out monthly while the bakery wants more, and the bakery is short against household demand, so the annual grocery bill rose roughly fourfold over one year. This is the exact shape of the next gate: supply does not respond to scarcity. No firm expands and no competitor enters when a good is dear.
- **Next gate (Step B2): supply answers the price signal.** Production targets and firm entry must react to margin and scarcity, so that a shortage is resolved by output rather than by unbounded price compounding.
- **Audit opened by this step.** Five surviving bounds are unowned and scheduled for removal, each suspected of hiding a loop: the loan rate clamped to 15-45% (`credit_market.rs`), the population growth rate clamped to +/-3% per year and the output growth rate clamped to -15/+20% (`simulation.rs`), the smoothing of regional satisfaction shifts (`regional_interests.rs`), and the clamped effect of government programmes (`government_program.rs`).

### Step B2 (in progress): supply answers the price signal

- **Diagnosis.** The supply machinery already exists and is simply never triggered. `plan_observed_production_adjustments` does expand a firm that keeps selling out with a positive margin, but only up to `demand_ceiling.min(physically_feasible)`, and the physical ceiling is the firm's installed capacity, which never grows. `investment.rs` can build capacity, but `launch_investment_project` and `advance_investment_project` are reachable only through an external command; no monthly stage ever calls them. `firm_entry.rs` founds new producers, but only against residual *survival* shortages. So a dear good attracts neither expansion nor entry, which is exactly the loop the deleted price ceiling was hiding.
- **Blocker found and cleared first.** Committing investment funds moves firm cash into `committed_investments`, which is outside the ADR 0170 money invariant, and founding a firm deducted `capital_cost` from the founder and destroyed it outright. Turning on autonomous investment before fixing this would have reopened the monetary leak that ADR 0170 closed.
- **Done: capital spending now reaches households.** The capital cost of founding a producer is paid to the cohorts of the founding region as building income, split by population with the largest-remainder method so the paid total equals the spent total to the minor unit, and reported as a typed `CapitalOutlayDistributed` event. Construction firms and equipment makers are not separate counterparties yet; that approximation is documented at the call site. A new gate asserts that every minor unit of founding capital arrives somewhere. `SIMULATION_VERSION` is now 98.
- **Next in this step:** an autonomous monthly capacity decision. A firm that sold out with a positive expected margin and a target pinned at its installed capacity should commit cash, launch a project, and advance construction through the ordinary replayable command boundary, with construction spending paid out on the same closed-circuit rule. Then widen firm entry beyond survival goods to any persistently dear good with a solvent founder.

### Authoritative hourly observatory clock and visual correction

- Fixed the stale upper-left date by making the header react to every authoritative snapshot refresh. It now shows day, month, year, hour, pause state, and speed.
- Added replayable hourly world time. The client can run at 1/4/12/24/72 simulated hours per real second, pause, or single-step; crossing month end atomically settles the monthly economy before entering the next day. Program decisions remain legal between economic ticks.
- Introduced a clearer top command bar and separate inspector, program, and political-timeline surfaces using a coherent observatory palette. ADR 0162 records the boundary and `SIMULATION_VERSION` 95.

### Strategic province-map blockout

- Replaced technical region rectangles with adjoining province clusters forming a continuous two-country landmass. Irregular edge tiles establish a coastline silhouette while dark province borders, a gold selected border, country-scale labels, and compact province labels establish a grand-strategy visual hierarchy.
- Added a subdued ocean grid, land shadow, and the visible Arcadia-Borealia transport connection. Overlay colors remain authoritative presentation data; the geometry does not invent simulation adjacency.
- ADR 0163 records the prototype boundary. Authored polygon meshes, mouse picking, terrain, rivers, and zoom-dependent detail remain future map-production work.

### Interactive strategic-map command surface

- Political ownership is now the default map language, with stable Arcadian and Borealian colors plus alternate confidence, population, and production modes. Province fills and borders react independently to hover and selection.
- Added deterministic mouse hit testing, click-to-inspect, a cursor-following regional tooltip, UI occlusion, and presentation-only camera navigation. Camera controls use I/J/K/L, U/O, and R so they cannot accidentally trigger treasury, debt, execution, cancellation, or time decisions.
- Reworked the visual hierarchy into a strategic masthead, reactive command clock, map-mode rail, contextual legend, inspector, program desk, and political timeline. The map now carries city/capital markers, terrain strokes, country labels, and a dashed strategic route.
- ADR 0164 records the interaction boundary. The next production-level map pass should move from clustered sprites to authored shared-edge polygon meshes and zoom-dependent rendering.

### Russian strategic interface and map level of detail

- Converted the full graphical command surface to Russian, including time state, map modes, camera guidance, inspection, tooltips, program decisions and statuses, current demo geography, city roles, and political-timeline presentation.
- Added authoritative country snapshots for treasury, public debt, legitimacy, and elite cohesion. The selected region now drives a state-resource strip in the strategic masthead.
- Added zoom-dependent hierarchy: country labels for strategic scale, province labels for operational scale, and city/terrain marks for close inspection. Localization and visibility remain presentation-only. ADR 0165 records the boundary.

### Clickable strategic control deck

- Added visible Russian buttons for political, confidence, population, and production map modes, together with pause and 1/4/12/24/72-hour speed controls. Active, hover, and inactive states now have distinct strategic-command treatments.
- Added a selected-country color flag and authoritative inspector bars for confidence, relative population, and relative production. Selection and snapshot changes rebuild these indicators without creating client-owned simulation values.
- Real executable smoke launch remained stable for five seconds after the new Bevy interaction systems were installed. ADR 0166 records the boundary.
