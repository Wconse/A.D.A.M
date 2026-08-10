# Economy Rebuild Plan

Status: implementation contract after the source-level audit in `docs/engine-audit.md`.

This document is deliberately not a list of isolated bug fixes. The audit shows that most defects
come from four architectural mistakes: quantities move without counterparties, market-wide signals
are treated as firm-local facts, sensors are disconnected from the state they claim to measure, and
bare constants act as invisible decision-makers. Patching individual symptoms would preserve those
mistakes and make later work harder to reason about.

The rebuild must keep Stage 0's goal: a deterministic headless world whose fifty-year chronicle is
interesting because actors make consequential choices, not because the engine injects drama.

---

## 1. Non-negotiable outcomes

The rebuilt economy is accepted only when all of the following are true.

1. **Every stock has one authoritative owner.** Money, goods, labor time, installed capacity,
   dwellings, transport capacity and legal claims cannot be represented by competing fields.
2. **Every flow names both sides.** A debit without a recipient, a good without a source, work
   without a worker, or debt without a creditor is invalid unless the event explicitly represents
   issuance, destruction, spoilage, death or write-off.
3. **Intent is not obligation.** A forecast, plan, bid, contract, reservation, delivery and settled
   payment are distinct states. Only a valid reservation can block a resource.
4. **Physical and financial settlement agree.** Production consumes inputs and labor; delivery
   consumes time and capacity; payment moves between named balances; failure leaves a claim or a
   loss with a named bearer.
5. **Signals use settled evidence and preserve causes.** Unmet need, rejected price, lack of cash,
   missing supply, congestion, breach and political exclusion remain different facts.
6. **The engine enforces feasibility, not prudence.** Legal authority, resources, time, contracts
   and physical capacity are hard constraints. Fairness, risk tolerance, favoritism and ideology
   belong to actor policy.
7. **No economic constant is ownerless.** A threshold must be an arithmetic domain rule, content
   parameter, contract term, institution policy or actor preference. Comments must name who holds
   the line and what that party pays.
8. **Determinism is structural.** Canonical ordering resolves simultaneous bookkeeping, never who
   wins scarce resources. Allocation requires a rule represented in the world: price-time priority,
   contract seniority, auction, rationing policy, queue discipline or explicit authority.
9. **The chronicle is downstream.** Narration reads durable events and cannot be the only place a
   consequence exists.
10. **A fifty-year run remains inspectable.** The engine must explain why a price, shortage,
    migration, bankruptcy, investment or political crisis happened without replaying the full event
    history every month.

---

## 2. Target architecture

### 2.1 One economic kernel

Introduce a small internal kernel in `adam-core`; do not create a separate simulation or UI-owned
state. It owns typed stock movements and exposes domain operations to existing systems.

The minimum stock families are:

- **Money accounts:** household cash, firm cash, actor cash, treasury, lender cash, escrow and
  restricted project accounts.
- **Claims:** loans, wage arrears, trade payables, tax liabilities, freight claims, program
  appropriations and liquidation claims. A claim names debtor, creditor, principal, seniority,
  due schedule and status.
- **Goods:** inventories by owner and location, goods in transit, reserves, committed quantities
  and spoilage/write-off records.
- **Labor:** people by cohort and participation status, active work commitments, available hours,
  skill, wage and employer.
- **Capital:** installed productive capacity, condition, owner, location, construction state and
  operating requirements.
- **Settlement capacity:** dwellings, terminal work slots and route movement slots with explicit
  time units.

A stock is never copied into a second authoritative field for convenience. Derived views may be
cached, but they must be rebuildable and checked against the source.

### 2.2 Balanced postings, not scattered setters

Every money-changing operation posts a transaction with:

- transaction id and date;
- cause and authorizing actor/institution;
- debit account(s) and credit account(s);
- related claim or contract;
- optional explicit source/sink classification for currency issuance, retirement and adjudicated
  write-off.

Normal transfers must sum to zero. Currency issuance and retirement must name the institution and
policy that authorizes the stock change. Claim creation is not cash creation. Claim settlement moves
cash and reduces the liability and creditor asset by the same amount.

`observed_money_stock` becomes a projection over the account registry. A monthly close verifies:

`closing cash = opening cash + authorized issuance - authorized retirement`

and separately reconciles claims, inventories and reserved balances. A diagnostic event is emitted
only after the close passes.

### 2.3 The common action lifecycle

Economically important operations use the same lifecycle:

1. **Observe:** collect bounded, causal evidence from completed settlements.
2. **Decide:** an actor or institution produces an intent under its policy.
3. **Validate:** check authority, resources, law, contracts and physical feasibility.
4. **Reserve:** lock only the resources needed for the accepted action.
5. **Execute over time:** consume labor, goods, capacity and calendar time.
6. **Settle:** transfer ownership, cash and claims atomically.
7. **Record:** emit durable events and update bounded observations.
8. **Release or default:** unused reservations return to their owner; failure assigns losses and
   claims instead of silently deleting them.

This replaces special-case shortcuts for imports, programs, investment and relief.

### 2.4 Three layers that may not be mixed

- **Domain layer:** authoritative stocks, contracts, legality and settlement.
- **Decision layer:** forecasts, bids, policies, preferences and actor choices.
- **Observation layer:** bounded history, statistics and chronicle evidence.

Decision code may read observations but cannot mutate stocks. Observation code may read domain
events but cannot create economic consequences. Domain validation may reject impossible actions but
cannot replace an unwise actor choice with a safer one.

### 2.5 Explicit time and capacity units

Every capacity field includes its unit in the type or name: quantity per day, worker-hours per
month, simultaneous storage quantity, dwellings, batch capacity per month. Reservations use an
interval. Route legs are reserved when cargo is scheduled to enter that leg, not for the whole trip
at dispatch. Terminal throughput and storage occupancy are separate resources.

---

## 3. Gameplay loops the rebuilt model must support

The economy is not rebuilt merely to conserve numbers. Each loop must create decisions, conflicts
and history.

### 3.1 Household security loop

Employment, wages, taxes, benefits, debt service and prices determine disposable cash. Households
choose consumption, savings, borrowing, substitution, training and migration. Survival shortfall
has a preserved cause. Repeated shocks reduce health and labor capacity; recovery requires actual
consumption and services. Political blame follows observed responsibility rather than any missing
purchase.

Interesting choices:

- accept a lower wage now or wait for a skilled job;
- draw savings, borrow, substitute, migrate or organize politically;
- fund retirement through family transfer, pension, savings or public benefits;
- remain in a high-wage congested region or move to cheaper housing with weaker services.

### 3.2 Firm operating loop

A firm forecasts its own demand evidence, quotes prices, procures inputs, hires labor, produces,
sells, pays tax and debt, then calculates distributable profit. It may conserve cash, change output,
invest, borrow, renegotiate, downsize, enter administration or liquidate.

Profit is settlement-based:

`revenue - consumed input cost - wages owed - taxes accrued - depreciation - interest accrued`

Cash flow and profit remain distinct. Unpaid wages increase liabilities, not earnings. Capital
purchases affect cash immediately and profit through depreciation or impairment over time.

Interesting choices:

- low price and high utilization versus margin and resilience;
- local expensive input versus risky import;
- permanent staff versus training and temporary work;
- debt-funded expansion versus cash reserve;
- pay dividends, retain earnings, recapitalize or accept dilution;
- reorganize a viable operation or liquidate assets to higher-value users.

### 3.3 Supply, price and entry loop

Offers compete on delivered price, delivery date, reliability and contractual preference. Demand
that refuses price is not a physical shortage. Seller observations contain only the seller's own
offer, fills, refusals and attributable unserved demand.

Persistent profitable unserved demand can attract entry. Entry requires founder money, a project,
inputs, labor and time. A plant is never created by paying an abstract capital cost to nobody.

### 3.4 Logistics loop

All cross-region goods use the same shipment system, whether the buyer is a household, firm or
state. A shipment has route legs, terminals, expected and actual dates, custody, insurance or loss
bearer, freight contract and payment schedule. Reliability affects delay/loss probability through
deterministic seeded draws. Congestion creates queues, demurrage, breach, rerouting and investment
signals.

Carriers receive freight revenue and pay named fuel suppliers, workers, maintainers, terminal
operators and taxes. Route and terminal investment is a real project. Market imports may use a
fast abstraction only if it posts the same costs, time and risks as an equivalent explicit shipment.

### 3.5 Public economy loop

Taxes accrue from defined taxpayers and settle into treasury. Spending requires an appropriation,
funding source and executing institution. Programs procure goods and labor or make explicit
transfers. Debt issuance has a creditor or a modeled monetary authority; interest and restructuring
assign gains and losses to holders.

The government may concentrate spending, overpay, neglect regions, borrow excessively or ration by
political priority if authorized. Those actions remain legal where appropriate and create prices,
shortages, corruption evidence, creditor losses, opposition and legitimacy effects.

### 3.6 Demography and settlement loop

Population changes only through births, deaths and migration. These operations also update
households, age structure and balance sheets. Household formation and dissolution are explicit.
Deaths resolve property and debt through inheritance/default rules. Ageing moves portions between
bands; it does not age an undifferentiated multi-decade cohort at once.

Housing is capital produced by projects with land/site permission, labor, materials, finance and
time. Rent or ownership cost connects occupancy to household budgets. Congestion therefore has a
real supply response and real losers.

---

## 4. Migration strategy: replace vertically, never rewrite everything at once

The current simulation remains runnable while one authoritative vertical slice at a time is
replaced. A compatibility adapter may read legacy state, but new code must not write both models.
For each slice:

1. write its invariants and acceptance scenarios;
2. identify legacy fields that will cease to be authoritative;
3. introduce the new state and settlement path;
4. migrate scenario content;
5. switch every reader in that slice;
6. delete the old writer and adapter;
7. run the complete gate;
8. update `docs/roadmap.md` with measurements and the next gate.

A feature flag may select old or new behavior during development, but a saved world cannot run both
for the same stock. Flags are removed when the slice is accepted.

---

## 5. Ordered implementation slices

The order is dependency-driven. Later slices may not compensate for an earlier missing invariant.

### Slice 0 - Audit harness and baseline scenarios

**Purpose:** make structural errors fail immediately before changing behavior.

Deliver:

- a compact `EconomicInvariantReport` computed at monthly and annual close;
- account, claim, goods, population/household and reservation reconciliations;
- scenario fixtures for one closed local economy, two trading regions, one distressed firm, one
  public program and one congested route;
- opening and closing stock tables in test output;
- a machine-readable defect-to-test matrix referencing audit sections.

Acceptance:

- the harness detects a deliberately injected money sink, duplicated inventory, ghost household,
  overbooked route and orphan claim;
- current known failures are explicit allowlisted failures with audit ids, not ignored assertions;
- determinism remains byte-for-byte stable for identical seed and commands.

### Slice 1 - Account registry and transaction journal

**Status (2026-08-10):** the first vertical gate is accepted, firm
procurement joined it in version 118, annual sales taxes plus owner distributions
joined in version 119, and firm investment escrow joined in version 120. Typed
owned accounts, balanced atomic postings and a serialized fingerprinted journal
now settle paid payroll, household purchases, firm purchases, direct emergency
relief, tax collection, owner payouts, investment reservation and project outlay.
Government-program escrow, explicit issue/retire authority and the shared claim
registry remain in this slice.

**Purpose:** establish the only legal path for money movement.

Deliver:

- typed account ids and account owners;
- balanced posting API with atomic preflight;
- restricted/escrow accounts;
- explicit currency issue/retire events;
- claim registry shared by household, firm and public debt;
- projections replacing scattered money sums.

Migrate first: wages, household purchases, firm purchases, taxes, dividends and direct relief.

Acceptance:

- every migrated transfer preserves total money;
- failure leaves all balances unchanged;
- no migrated module calls raw cash setters;
- rounding remainders have a named owner or deterministic largest-remainder allocation.

### Slice 2 - Labor, payroll and household income

**Purpose:** remove the split between cohort status, agreements, firm workers and income.

Deliver:

- labor participation by cohort and age band;
- employment commitments as the source of employed workers;
- available workers derived from people minus incompatible commitments;
- payroll that creates wage expense, cash settlement and arrears claim;
- income history derived from received wages and named transfers;
- explicit unemployment benefit, pension and informal-income sources where content enables them;
- retirement that ends work and activates only funded replacement income.

Remove: payerless `uncontracted_monthly_income`, immutable productive workforce and status fields
that do not follow agreements.

Acceptance scenarios:

- hiring increases staffed production but not population;
- layoffs reduce payroll, household cash and production capacity next month;
- unpaid wages conserve cash and create a worker claim without increasing profit;
- a retired cohort receives no money unless a pension/savings/transfer source pays it;
- one person cannot work for two employers or a program in the same time interval.

### Slice 3 - Production, inventory costing and capital condition

**Purpose:** make output the result of owned inputs, labor and plant.

Deliver:

- bills of materials with consumed quantity and inventory cost lots;
- worker-hours and staffed-capacity constraints;
- energy/maintenance inputs where configured;
- work-in-progress for multi-month production;
- capital condition, depreciation and mothballing;
- production settlement that posts goods and expenses together.

Acceptance:

- production cannot exceed the minimum of plant, staffed labor and inputs;
- no-input or no-worker firms cannot produce unless the recipe explicitly requires neither;
- goods conservation holds through transformation;
- shutting plant avoids selected operating costs but sacrifices output and may damage condition;
- depreciation affects profit without destroying cash.

### Slice 4 - Household demand and causal market clearing

**Purpose:** replace wealth exhaustion and undifferentiated shortage with choice and evidence.

Deliver:

- disposable-resource budget after contractual obligations and a policy-selected reserve;
- survival, normal and discretionary plans with explicit substitution sets;
- offer ranking by effective delivered terms;
- refusal reasons: price, cash, quantity, delivery time, reliability, policy and congestion;
- buyer and seller observations derived from the same settlements;
- allocation rule chosen by market institution, not collection order.

Acceptance:

- households do not automatically spend all liquid wealth;
- a high price produces price refusal, not physical shortage or foreign grievance;
- each seller sees only attributable demand;
- rationing policy can favor a group only through explicit authority and emits displaced demand;
- changing identifier order does not change allocation under equivalent bids.

### Slice 5 - Prices, procurement and trade credit

**Purpose:** make prices and input choice respond to real costs and alternatives.

Deliver:

- firm quote floor from expected marginal cost, tax and financing policy;
- observed transaction-price updates with bounded evidence windows;
- procurement optimization over delivered price, date, reliability and relationship terms;
- aggregate batch affordability across all inputs, wages, tax and logistics;
- purchase orders, deposits, delivery obligations and trade-payable claims;
- local preference as actor policy or legal rule with measurable cost, never engine priority.

Acceptance:

- a cheaper reliable import can beat a local supplier;
- a local-preference policy may still choose local and records its premium;
- total planned batch cost cannot exceed the authorized budget while each line passes separately;
- partial wholesale fills preserve offered, sold and unsold quantities;
- reference prices learn from settlements and decay stale evidence.

### Slice 6 - Firm accounts, credit, distress and investment

**Purpose:** unify profitability, liquidity and ownership decisions.

Deliver:

- accrual income statement and cash-flow statement from journal entries;
- retained earnings reconciled to closed periods;
- debt schedules with lender assets, borrower liabilities, interest and default;
- underwriting from after-tax, after-maintenance, debt-aware cash projections;
- collateral appraisal from expected liquidation proceeds and market depth;
- investment projects with sponsor, contractor, materials, labor, milestones, escrow and failure;
- board decisions for dividends, recapitalization, restructuring and liquidation.

Acceptance:

- unpaid obligations never increase distributable profit;
- a proposed project does not block cash until approved and reserved;
- one market shortage cannot be financed independently by every firm as its own demand;
- completed capacity equals delivered project output; cancelled projects release unspent escrow;
- liquidation uses bids or explicit negotiated transfers, honors seniority and charges procedure
  costs to the estate or institution that provides them;
- owner recapitalization is a decision or contractual call, not automatic confiscation.

### Slice 7 - Unified logistics and infrastructure

**Purpose:** remove the instant-import economy and make geography meaningful.

Deliver:

- one shipment path for household, firm and government purchases;
- leg-by-leg route and terminal scheduling;
- distinct throughput, storage occupancy and handling work;
- freight settlement to carrier and carrier expenses to named counterparties;
- contract start, expiry, priority, breach, compensation and renegotiation;
- deterministic reliability outcomes and cargo loss allocation;
- infrastructure projects using Slice 6 project machinery.

Acceptance:

- a 90-day trip consumes only the appropriate interval on each leg;
- contracts cannot silently promise the same capacity twice;
- an oversized shipment is split, rejected or specially scheduled and cannot block the queue
  forever;
- terminal operators earn handling/storage revenue and bear operating costs;
- explicit and market-generated imports have equal accounting, time and risk for equal routes;
- congestion, high rates and profitable utilization create evidence for actor-chosen expansion.

### Slice 8 - Public finance, reserves, services and programs

**Purpose:** make the state an actor using the same economy, not a second money system.

Deliver:

- tax liabilities and settlement from defined bases;
- appropriations backed by treasury, issued debt or explicit arrears;
- public creditors or monetary authority for debt issuance;
- program and reserve escrow with release on cancellation;
- procurement through ordinary markets and logistics;
- public employment through the labor ledger;
- service output from facilities, labor, materials and maintenance;
- reserve storage owned by operators receiving carrying-cost payments;
- region incidence reports from every tax, transfer, purchase, wage and project flow.

Acceptance:

- the same spending cannot both transfer household income and independently buy free services;
- a program with no workers or materials delivers nothing and retains/reallocates its budget;
- cancellation returns unspent funds according to law;
- reserve maintenance pays an operator and neglect assigns spoilage and political responsibility;
- concentrated, corrupt or wasteful spending remains possible when authorized and produces real
  winners, losers, prices and opposition;
- restructuring writes down named creditor assets.

### Slice 9 - Demography, health, housing and migration

**Purpose:** replace formula rescaling with stock-flow population history.

Deliver:

- births into child cohorts with parent household links or deterministic aggregate lineage;
- deaths by age/health with estate and debt resolution;
- household formation, merging and dissolution;
- fractional ageing/movement between age cohorts;
- migration of people, household balances, health, skills and obligations without orphan state;
- housing occupancy and construction via project machinery;
- health driven by actual consumption, shelter, work and service delivery.

Acceptance:

- people reconcile exactly as opening + births + arrivals - deaths - departures;
- households are zero for zero people and never exceed people;
- per-household demand follows household formation rather than population rescaling;
- starvation deaths cannot be overwritten by a separate growth formula;
- migration preserves or explicitly disposes every linked state record;
- new arrivals do not receive perfect-health evidence for an unobserved month;
- housing supply requires land/site authority, finance, workers, materials and time.

### Slice 10 - Politics, actor policy and chronicle

**Purpose:** let material outcomes drive politics while preserving player freedom.

Deliver:

- political sensors derived from causal records produced by prior slices;
- responsibility attribution for price refusal, domestic policy, foreign breach and congestion;
- actor policies for prudence, ideology, favoritism, corruption, risk and time preference;
- warnings and forecasts that advise but do not silently veto feasible commands;
- chronicle arcs built from durable changes, conflicts and named decisions.

Acceptance:

- employment, hardship and service sensors reconcile to domain records;
- poverty alone cannot create a foreign grievance without attributable foreign action;
- changing an actor policy changes decisions but not legality;
- explicit extreme decisions remain executable and create modeled consequences;
- the chronicle can name the decision, material mechanism, winners, losers and delayed result.

### Slice 11 - Legacy removal and long-run acceptance

Deliver:

- delete superseded fields, setters, duplicate shortcuts and compatibility adapters;
- remove tests that assert known defective behavior and replace them with invariant/scenario tests;
- run 1-, 10- and 50-year scenario suites across multiple seeds and policy profiles;
- profile memory and runtime; replace full-history scans with bounded projections;
- document remaining approximations and their owners in `docs/roadmap.md`.

Final acceptance:

- formatting, Clippy, tests and docs pass;
- no allowlisted audit defect remains without an explicit deferred ADR;
- all stock reconciliations pass every month for fifty years;
- equivalent command replays produce identical events, state and fingerprint;
- at least one scenario produces growth, recession, shortage, recovery, migration, investment,
  default and political reaction through causal loops rather than scripted events;
- no regression fingerprint is changed merely to hide a behavioral difference.

---

## 6. Required test portfolio

### 6.1 Invariant tests

Run after every economic stage, not only at year end:

- balanced cash and authorized issuance;
- claim asset/liability equality;
- non-negative owned stock unless a type explicitly permits overdraft;
- inventory conservation across transformation, transit, spoilage and write-off;
- population and household reconciliation;
- labor commitment exclusivity;
- route/terminal interval capacity;
- reservation ownership and expiry;
- retained earnings reconciliation.

### 6.2 Metamorphic tests

These catch identifier and magic-order defects.

- renaming or renumbering equivalent actors does not change totals or winners;
- splitting one equivalent seller into two does not duplicate market-wide demand;
- merging equivalent cohorts preserves aggregate consumption and wealth;
- dividing a shipment into equal lots changes only documented rounding;
- running the same accepted commands through direct API and `WorldCommand` gives identical state;
- adding an irrelevant actor or firm does not change unrelated allocation.

### 6.3 Causal intervention tests

Each feedback loop needs paired worlds differing in one cause:

- higher wage changes household cash and firm cost;
- higher tax changes price/profit/treasury without changing physical output directly;
- route outage changes delivery, not household willingness to pay;
- cash grant fixes affordability but cannot fix physical shortage;
- reserve release fixes physical availability but cannot create household cash;
- cheaper import changes sourcing and local profits;
- health loss changes effective labor and production;
- cancellation releases funds and stops future resource use.

### 6.4 Scenario tests

Keep scenarios small enough to explain by hand:

1. one household, one firm, one good, no debt;
2. two firms competing for one input;
3. two regions with expensive local supply and cheaper risky import;
4. profitable firm with liquidity shortage;
5. unprofitable firm with positive cash;
6. wage arrears and worker-first insolvency;
7. congested route and terminal queue;
8. relief for affordability versus reserve release for physical shortage;
9. biased public program with legal authority;
10. ageing, birth, death, migration and housing pressure.

Long-run tests come only after these local accounts can be explained line by line.

---

## 7. Rules for agent-driven implementation

Every implementation iteration must begin with a written slice contract in the pull request or
roadmap entry:

- decision or feedback loop being added;
- authoritative stocks touched;
- actors and authority involved;
- counterparties for every flow;
- failure modes and loss bearer;
- time and capacity units;
- invariants;
- acceptance scenarios;
- legacy writer to be deleted;
- measurements expected to change.

Before code is accepted, answer this checklist:

1. Can any quantity appear, disappear or be counted twice?
2. Can one person, good, currency unit or capacity interval be committed twice?
3. Is a plan being mistaken for a contract or a contract for settlement?
4. Does identifier order decide a scarce allocation?
5. Does any clamp or threshold lack an actor, institution, contract or arithmetic reason?
6. Does a sensor preserve why an action failed?
7. Are cash flow, profit and wealth kept distinct?
8. Can cancellation, expiry, default and partial fulfillment release or reassign resources?
9. Does a new field duplicate an existing authority?
10. Is the behavior tested by intervention rather than only by a snapshot?

Forbidden workflow:

- edit many systems first and infer invariants afterward;
- make a long-run number look realistic with a cap;
- add a second corrective transfer instead of balancing the first;
- update a deterministic fingerprint without explaining every changed event;
- leave both old and new writers active behind a permanent compatibility path;
- accept a green test that codifies a known audit defect.

---

## 8. Documentation and decision records

Create an ADR before Slice 1 defining accounts, claims and authorized money creation. Subsequent ADRs
are required only for decisions that constrain future work:

- time/capacity reservation semantics;
- inventory costing and depreciation;
- market allocation and rationing institutions;
- public debt holder/monetary authority model;
- demographic cohort transition model.

Do not create ADRs for tunable content values or one-off implementation details. Content parameters
must state unit, interpreting actor/institution and expected causal effect.

`docs/roadmap.md` is updated after each accepted slice with:

- what became authoritative;
- which legacy state was removed;
- invariants and scenario results;
- performance and fifty-year measurements when applicable;
- known approximations;
- next gate.

---

## 9. First executable milestone

The first coding iteration should be **Slice 0 only**, followed by the smallest part of Slice 1:

1. add an invariant report that exposes the current four liquid balances, committed-but-unheld
   money, claims, goods, population/household mismatches and capacity reservations;
2. add failing regression scenarios for A-2, A-3, AD-1, AD-2, Z-1 and AB-7;
3. introduce balanced postings for exactly three already-conserved flows: payroll paid amount,
   household market purchase and direct emergency relief;
4. prove their results match current intended transfers while the journal balances;
5. delete raw money mutation only for those three paths;
6. run formatting, Clippy, the full test suite and docs;
7. update the roadmap with measured failures still remaining.

Do not begin prices, investment or politics in this milestone. Their current outputs depend on broken
income and accounting; changing them first would produce another round of plausible numbers built on
false state.

The milestone is successful when the engine can reject an unbalanced posting at the operation that
caused it, not when a later annual statistic happens to look plausible.
