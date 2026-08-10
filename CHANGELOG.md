- Made firm investment commitments real restricted money. `InvestmentReservation` now moves operating cash into typed `FirmInvestmentEscrow`, observed firm money includes that balance, and project launch releases it through one atomic `InvestmentOutlay` transaction to the local households that build the works. Direct command and autonomous capacity-investment paths now share the same settlement, cannot omit or double-pay builders, and insufficient cash or escrow leaves the world unchanged. Version 120 records 2965 settlements and fingerprint 15155816647335977890 over the seed-1 fifty-year run at 360.2 ms/year.

- Moved annual sales taxes and owner distributions into the balanced money journal. `SalesTax` now stages each firm-to-treasury receipt before annual fiscal closure, while `OwnerDistribution` records one firm decrease and every positive owner credit in one atomic transaction, preserves canonical remainder ownership, and omits zero-rounded shares. Tax events, recorded revenue, and journaled decreases reconcile exactly; rejected distributions leave balances and the journal untouched. Version 119 records 2863 settlements and fingerprint 18361472839204730196 over the seed-1 fifty-year run at 363.6 ms/year.

- Moved firm-to-firm input purchases into the balanced money journal: buyer operating cash now funds seller goods revenue and optional carrier freight revenue through one atomic `FirmPurchase` transaction, while physical inventory remains staged on the cloned monthly ledger. Version 118 records 2700 settlements and fingerprint 1921886831440795849 over the seed-1 fifty-year run.

- Introduced the first authoritative balanced money-posting path: typed household, firm, treasury and actor accounts; atomic preflight; canonical postings; and a serialized, fingerprinted settlement journal. Paid payroll, household purchases (including carrier revenue) and direct emergency relief now use that path without duplicate raw balance mutation. A release-only failure exposed positive-priced sub-unit fills that round to zero minor units; those still move goods but correctly create no fake zero-valued money transaction. Version 117 completes the first ADR 0190 vertical gate with 2017 journal entries and fingerprint 5540401480800370794 over the seed-1 fifty-year run.

- Made hunger a reason to move. Internal migration was not dead code - it runs every year - but it could only ever fire for an unemployed household leaving six months of regional unemployment for six months of vacancies somewhere else. Every demo cohort is employed and no region advertises vacancies once firms are staffed, so in fifty years nobody ever moved: Eastport starved to zero output beside two regions that still had food, and its people stayed there and died. Nothing in the world pays for the rule "you may leave only once somebody elsewhere is hiring", so the rule is gone rather than tuned. A household may now leave because it cannot eat, judged by the same people-weighted survival fulfilment record that already decides who loses capacity and who dies, provided the refuge is materially better fed and has a dwelling free. Both sides pay: the mover pays the relocation fee and gives up its job, the standing agreement being reduced at the employer it abandons so the arrival is genuinely unemployed with no income, while the refuge pays in housing pressure and in more mouths against the same food - arrivals enter the destination's own provisioning average, which is what closes the gap that drew them. The one-household-per-region-per-year cap went too, being another limit nobody pays for; an exodus now ends only when the dwellings run out, when the households left cannot fund the fee, or when the refuge is no longer better fed. The first measured run then exposed a cost that was not economic at all: a permanent new cohort per migrating family, hundreds of them after one exodus, and a simulated year slowed from 139 to 870 ms. Arrivals now join a destination group that lives the same way, carrying people, dwellings, money and debts across intact, with health records merged by the people behind each so a starving family's famine is not washed away by the arithmetic of merging. In the demo, 247 households containing 733 people leave Northreach for Southvale between 2040 and 2043 and stop when housing pressure reaches the top of the scale - the first time a population in this project has answered a regional collapse by going somewhere else. It is not a rescue: excess deaths fall only from 3179 to 3151, because migration can redistribute people across regions that are all failing but cannot make any of them produce. ADR 0179; simulation version is now 105.

- Let a new employer outbid an old one for staff, then found out that the shortage evidence never reached it. A founder who finds nobody idle now scans standing agreements in the region for a qualified worker with three months of tenure whose employer is solvent, and offers the incumbent wage improved by the same ten percent margin the labour market already uses, capped by the ADR 0173 value ceiling so a founder and an incumbent are judged by identical arithmetic; the founder carries three months of the higher wage as working capital and the incumbent answers through the ordinary retention pass next month. It founded nothing, and the fingerprint had moved only because the version number is hashed. The reason was a dead channel: `reset_monthly_firm_accounts` runs at the end of the commercial cycle and cleared the shortfall map, while firm entry reviews the region after that cycle, so every shortage ADR 0176 recorded was wiped before the only party entitled to answer it could read one. The unit tests drove the stages by hand and never crossed the reset, so the channel was alive in tests and dead in every real month. Evidence now lives from the month that writes it to the month that reads it - procurement clears the map when it starts recording, not the accounts reset - and entry reviews grain in all three regions where it previously reviewed it never. Separately, a refusal to enter now names its cause instead of collapsing four unrelated situations into one silent `None`: `NoKnownRecipe`, `NoWorkerToBeHad`, `RaiseExceedsWorkValue`, `NoLocalPrices`, `NoSolventFounder`. The demo still founds no firms, and now says why - either the raise needed to move a worker exceeds what the work earns at local prices, or no resident holds enough cash to fund the works and the first wages. Both are honest economics and neither was tuned away. ADR 0177 and ADR 0178; simulation version is now 104.

- Made a shortage of an input a reason to produce it. Firm entry built its evidence from unmet household needs only, and the sole consumption profile in the demo is bread, so grain - an intermediate good no household ever eats - could go missing in the same region every month for fifty years without a single farm ever being considered. The shortage was journaled as an event and then discarded; nothing in the world remembered it. The world now keeps `monthly_firm_input_shortfalls` beside the other monthly firm accounts, procurement accumulates every unfilled order against the buyer's region as it reports the shortfall, and entry pressure accrues from that map exactly as it does from an unfed household. Both causes of an unfilled order count, including an exhausted route pool: a good nobody could deliver here and a good nobody here has are the same invitation to make it here. First-year excess deaths fall from 73 to 58 and the yearly toll decays faster. The demo still starves, and now for a stated reason rather than a mysterious one - entry can only hire the unemployed, every demo cohort is employed, so the pressure this creates has nowhere to go until a new employer can outbid an existing one for staff. Recorded alongside it: a spike on `spike/contested-input-price` proved that pricing a contested input by displacement, while correct, saves nobody on its own, because raising the price of a fixed quantity only decides who goes without. ADR 0176; simulation version is now 103.

- Stopped public housing investment from destroying money: the cost committed to a construction project is now paid to the households of the region that builds it, split by population with the same largest-remainder routine used for founding capital, and reported as a typed `HousingOutlayDistributed` event. Previously the treasury was debited and nobody was credited, which is the same leak ADR 0170 closed for taxation and ADR 0172 closed for capital works, left open for housing. A region with no households now starts no project, because a project with no builders would be the leak under another name. This was not found by inspection: it surfaced as a stubborn twelve-unit discrepancy once the `fiscal_circuit` invariant was extended to count every claim on the treasury, including reserve procurement. Gated: every unit the treasury spends on building must be earned by a builder. ADR 0175; simulation version is now 102.

- Let an employer end a standing agreement the work can no longer pay for, but only once it has stopped meeting the wage. Each month every active agreement is measured against the same value ceiling ADR 0173 uses for new hires; where the wage stands above it and the agreement already carries unpaid arrears, the agreement is stood down and reported as a typed `EmploymentEndedAsUnprofitable` event, with accrued arrears surviving the release. The arrears condition is the whole point: declining a hire is free, but ending a standing agreement costs the employer a worker it already found and trained, so it waits out a bad price for as long as its own cash lasts, and the question only settles when the loss stops being financed by the employer and starts being financed by the worker. The first draft of this rule fired whole rosters on one month of bad prices at no cost to anyone; it broke nearly every test world in the repository, and the worlds were right - a costless instant layoff is not a thing employers have. Gated in both directions: an unpaid loss-making agreement ends, a loss-making agreement whose wage is still met survives. Separately, the fiscal circuit invariant now counts reserve purchases and housing construction as claims on the treasury, which exposed that housing construction still pays its money to nobody at all. ADR 0174; simulation version is now 101.

- Gave the employer a side of the wage bargain: a firm now refuses a hire whose wage exceeds the value of the work it buys, measured as the local price of a batch of output less the local price of the inputs it consumes, divided by the labor the recipe says a batch takes. The vacancy stays open instead of being filled at a loss, and keeps registering as vacancy pressure. Previously the wage was built only from what the worker used to earn and how badly the firm wanted staff, so firms hired themselves into permanent loss and the whole demo economy starved behind that one missing half of the loop. The bound is owned on both sides: the firm would pay in cash every month, and the worker pays by staying unemployed because the work available is not worth the asking wage. Gated in both directions. ADR 0173; simulation version is now 100.

- Connected supply to the price signal: a solvent firm that keeps selling out with a positive operating margin now orders new capacity on its own each month, builds it over three months, and gains the output. The expansion is sized by two real quantities only — the demand it watched walk away and the capacity it can pay builders for out of its own cash — so a poor firm in a hungry market honestly stays small. Committing the money is a scoped corporate power crossing the replayable command boundary, and the construction budget is paid to local households as building income rather than vanishing into the works. Gated in both directions: a profitable sold-out bakery must expand and pay its builders, and a loss-making chain must refuse to expand however hungry its market is. ADR 0172; simulation version is now 99.

- Closed the investment side of the monetary circuit: the capital cost of founding a producer is now paid to the households of the founding region as building income, split by population without rounding loss and reported as a typed `CapitalOutlayDistributed` event. Previously that money was deducted from the founder and destroyed, which was the same leak ADR 0170 closed for taxation. Simulation version is now 98.

- Made prices form themselves out of settled trade: a market that sold out with demand unfilled raises its price by the observed scarcity, a market holding unsold stock cuts its price even when needs went unmet beside it, and every revision is reported as a typed `RegionalPriceAdjusted` event. The move is unbounded, because the two rejected drafts proved that a monthly ceiling on an integer price grid both lied about its own size and hid the missing supply response. Only two limits remain, each with somebody who pays for it: the variable-cost floor and a sub-unit carry that exists because currency is discrete. Established the binding project rule *No unowned constraints* in `AGENTS.md`: every limit must name the party bearing its cost, or be deleted rather than tuned. ADR 0171; simulation version is now 97.

- Closed the fiscal circuit so taxation moves money instead of destroying it: executed public spending now reaches household cohorts as public wages, transfers, and coupon on domestic debt, split by population without rounding loss and reported as a typed `PublicOutlayDistributed` event. Money is now created only by public borrowing, proven by a dedicated invariant gate. The demo economy no longer dies at once: output is sustained for roughly four decades where it previously fell to zero almost immediately. ADR 0170; simulation version is now 96.

- Fixed the Linux workspace build by explicitly enabling the `x11` and `wayland` windowing backends for the observatory, which Bevy's disabled default features had removed; corrected the Step 100 roadmap entry that claimed COMPLETE while CI was red, and pointed README/performance docs at commands that actually exist. ADR 0169.

- Recorded two confirmed simulation defects as open, deliberately unfixed: the demo economy collapses to a few hundred people on every seed because the 20% final-sales tax leaves the circuit without returning to it, and chronicle `importance` is saturated at 100 so ranking does nothing.

- Fixed fully invisible interface text by embedding the Cyrillic font directly in the executable instead of loading it from a manifest-relative asset path, and by applying it to every newly added text component rather than only at startup.

- Replaced the rectangular province blockout with a deterministic organic province map: jittered hex lattice with shared seam-free corners, noise-generated continent and coastline, balanced contiguous country/region partitioning, terrain relief shading, layered province/coast/region/country borders, capital markers, and zoom-dependent labels.

- Fixed missing-glyph squares in the Russian observatory by bundling OFL-licensed Noto Sans, applying it to every UI and world-space text entity at startup, and verifying the asset in a captured runtime smoke launch.

- Added a clickable Russian strategic control deck for map modes, pause, and five time speeds, plus active/hover styling, a selected-country flag, and authoritative confidence, population, and production bars in the regional inspector.

- Localized the complete Bevy command surface into Russian, including current country/region/program names and graphical chronicle summaries; added an authoritative country resource bar and zoom-dependent country, province, city, and terrain detail.

- Fixed the observatory startup panic by making the inspector, map-mode, and legend text queries explicitly pairwise disjoint; added a real executable smoke launch to the verification pass.

- Expanded the Bevy observatory into an interactive strategic-map command surface with a default political mode, stable country palettes, mouse hover and click selection, regional tooltips, camera pan/zoom/reset, map-mode legend rail, city/capital markers, terrain marks, and conflict-free controls.

- Replaced the observatory region-card grid with a first grand-strategy province-map blockout: contiguous Arcadian and Borealian land, irregular coast silhouettes, province borders, country labels, an inter-country route, ocean grid, overlay coloring, and selected-province emphasis.

- Added an authoritative replayable hourly clock, automatic monthly economic settlement at calendar boundaries, a reactive date header, pause/speed controls, and a first structured visual-panel pass for the Bevy observatory; simulation version is now 95.

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
