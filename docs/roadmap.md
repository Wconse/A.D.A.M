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
