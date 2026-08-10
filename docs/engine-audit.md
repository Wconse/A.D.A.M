# Engine Audit — Standing Defect Register

**Status:** diagnostic only. No code in this pass. Nothing here has been fixed.
**Opened:** simulation version 116, fingerprint `2108872914777405248`, 234 core tests green.
**Method:** line-by-line reading of `crates/adam-core/src`, following causal chains rather than files.

## Why this document exists

The gate is green and the world still dies. Green tests prove the code matches its
fixtures; they say nothing about whether the economy closes. This register records
every place where the engine either creates value from nothing, destroys it silently,
or enforces a limit that `AGENTS.md` would call magic — a limit no modelled actor pays
for.

Each entry answers five questions:

- **Where** — file, function, and line where the behaviour lives.
- **What it does** — the literal arithmetic, not a paraphrase.
- **Why it is wrong** — the causal loop that is missing or short-circuited.
- **Who pays** — the `AGENTS.md` test. "Nobody" means the constraint is magic.
- **Consequence** — what it produces downstream, and where that was observed.

Severity is about the closure of the economic circuit, not about how odd the code looks.

- **Critical** — breaks conservation of money or population; makes every long run meaningless.
- **Major** — a causal loop is absent or inverted; behaviour is systematically wrong in one direction.
- **Minor** — bias, misattribution, or fragility that distorts but does not break.

---

# A. Money is created and destroyed outside any transaction

These are the reason a fifty-year run drifts from `3 460 015` to `3 497 155` in measured
money stock, and the reason the `closed_circuit` probe showed a farm dying at zero margin
while its only customer grew from 200 to 1389.

## A-1. Households borrow from nobody — Critical

**Where:** `cohort.rs`, `borrow_for_survival`, called from `execute_monthly_household_coping`.

**What it does:** `liquid_wealth += amount; debt += amount`. That is the whole operation.

**Why it is wrong:** a loan is a transfer. There is no lender field, no counterparty
lookup, and no cash leaves any other balance sheet. The household's assets rise without
anyone's assets falling.

**Who pays:** nobody. Money is minted at the moment of hunger.

**Consequence:** the hungrier the world, the more money it contains. Scarcity is
self-financing, which is the exact inverse of what scarcity should do. Note the amount is
capped by `2 × annual_income − debt` (`coping.rs:3`), so the pump is throttled by a field
that, per **B-2**, is frozen at its fixture value.

## A-2. A monthly income nobody pays — Critical

**Where:** `cohort.rs`, `apply_monthly_cashflow` and `uncontracted_monthly_income`, called
from `execute_monthly_household_cashflows`.

**What it does:**

```
income = annual_income × (people − contracted_workers) × functional_capacity
         / people / 12 / 10_000
available = liquid_wealth + income
```

**Why it is wrong:** wages under an employment agreement are settled in
`execute_monthly_payroll`, where a firm's cash is debited and a cohort's wealth credited —
correctly. This second stream covers the people *not* under contract, and no actor is
debited for it. It is not a transfer, it is an appearance.

**Who pays:** nobody.

**Consequence:** every cohort with a non-zero `annual_income` and anyone outside a contract
receives money forever. In the `closed_circuit` probe this is the largest single reason
household wealth rose while the only firm went bankrupt.

## A-3. Debt service destroys money — Critical

**Where:** `cohort.rs`, `apply_monthly_cashflow`.

**What it does:** `scheduled = debt / 120`, `paid = min(available, scheduled)`, then
`liquid_wealth -= paid` and `debt -= paid`.

**Why it is wrong:** the payment has no recipient. Wealth falls, debt falls, and the money
simply ceases to exist. The divisor `120` is an unnamed constant embedded in the
expression — a ten-year amortisation nobody agreed to — and the loan bears no interest.

**Who pays:** nobody receives, so in effect every other money holder is paid in deflation.

**Consequence:** A-1 and A-3 together make household debt a private money supply valve:
minting on the way in, burning on the way out, with the burn rate fixed at 1/120 per month
regardless of contract, rate, or ability.

## A-4. The state hands out cash it never collected — Critical

**Where:** `simulation.rs:921` `plan_material_country`, field `household_outlay: spending`;
distributed by `distribute_public_outlay` (`simulation.rs:792`) via `credit_wealth`.
Compare `plan_country` (`simulation.rs:1008`) where `household_outlay` is
`Money::default()`.

**What it does:**

```
debt_brake_bps = min(debt × 10_000 / new_output / 10, 600)
spending_bps   = max(2_200 + (10_000 − legitimacy)/10 − debt_brake, 1_200)
interest       = debt × 300 / 10_000
spending       = new_output × spending_bps / 10_000 + interest
household_outlay = spending
```

Revenue appears nowhere in that chain. `close_budget` (`simulation.rs:1125`) then drains
the treasury and turns the remainder into `debt`.

**Why it is wrong:** the payout is a function of *output*, not of money the state actually
holds. When revenue is short the gap becomes public debt, and that debt is an obligation to
nobody — no actor's balance sheet records the matching asset. So the state credits
household wealth with money that was never taken from anyone.

**Who pays:** nobody. The debt is owed to the void.

**Consequence:** an unbounded annual money pump, structurally guaranteed to run every year
because the spending floor (**F-1**) exceeds any plausible tax take. Interest is added *on
top* of spending and also paid out to households, so the pump accelerates with the debt it
creates.

## A-5. Debt restructuring costs no creditor anything — Major

**Where:** `simulation.rs:1094` `plan_public_debt_restructuring`.

**What it does:** when `debt > output × 2` **and** `interest × 3 ≥ revenue.max(1)`, 40 % of
principal is written off. The cost is `−800` legitimacy and `−500` elite cohesion.

**Why it is wrong:** a write-off is a loss to a holder. Since the debt from A-4 has no
holder, nothing is destroyed and nobody is impoverished. Political indicators move; no
ledger does.

**Who pays:** nobody. Legitimacy is a number, not a claimant.

**Consequence:** the one mechanism that could have punished A-4 instead relieves it. The
pump has a built-in reset and can never break itself.

## A-6. Unsold inventory becomes household cash — Major

**Where:** `simulation.rs:239` `plan_material_regions` → `:329`
`plan_regional_inventory_change` → `:312` `observed_inventory_valuation_price`.

**What it does:** regional output = household final consumption + change in firm
inventories, where the inventory delta is valued at the last observed transaction price,
falling back to the regional reference price. That output then drives `spending` in A-4.

**Why it is wrong:** counting inventory investment in output is orthodox accounting and is
not itself the defect. The defect is what output is wired to: `spending` is a share of
output and is *handed to households as cash*. So producing goods nobody buys raises
household purchasing power.

**Who pays:** nobody. The firm keeps the goods *and* the economy gets the money.

**Consequence:** a closed loop with positive gain — overproduce, GDP rises, transfers rise,
households buy more, firms produce more. Nothing in the loop consumes a real resource.

## A-7. Unpaid sales tax is forgiven silently — Major

**Where:** `simulation.rs:379` `plan_firm_sales_taxes`.

**What it does:** `liability = taxable_sales × 2_000 / 10_000`,
`paid = min(firm.cash, liability)`. The shortfall is discarded — no arrears field, no
carry-forward, no penalty, no event.

**Why it is wrong:** either the tax is owed, in which case unpaid tax is a debt, or it is
not owed. Here it evaporates.

**Who pays:** nobody, and the state's revenue silently depends on firms' cash position at
one instant of the year.

**Consequence:** the tax is regressive in the worst way — solvent firms pay in full,
distressed firms pay nothing and owe nothing, and the distinction is invisible in the
fiscal record.

## A-8. The tax base is a rolling memory buffer, not a fiscal year — Minor, fragile

**Where:** `simulation.rs:379`, summing `final_sales_revenue()` over the whole of
`firm_operating_history`, whose length is capped by
`observation.rs:5 FIRM_OBSERVATION_HISTORY_LIMIT = 12`.

**Why it is wrong:** the annual tax base is defined by an unrelated constant that exists to
bound memory. It coincides with twelve months only while every firm records exactly one
observation per month. A firm that misses a month — insolvent, no policy, no offers — slides
its window back into the previous year and is taxed twice on the same sales.

**Who pays:** the firm, arbitrarily.

## A-9. Freight paid by households is untaxed — Minor

**Where:** `market.rs`, `record_market_settlement_evidence`: sellers get
`record_firm_final_sale(goods_spend)`, carriers get `record_firm_sale(freight_spend)`.
Only final sales are taxable (`plan_firm_sales_taxes`).

**Why it is wrong:** a household buying an imported survival good pays the tariff as part of
the till price, and that portion escapes the sales tax entirely. The tax base depends on
how far the good travelled.

---

# B. Labour: the market exists but never touches income

## B-1. Nobody is ever marked employed — Critical

**Where:** the only writes to `HouseholdCohort::employment` in production code are
`cohort.rs:219` (→ `Unemployed`, on aging out of `Child`), `cohort.rs:223` (→ `Retired`, on
entering `Senior`), and `cohort.rs:365` (→ `Unemployed`, in `leave_employment`).

Every occurrence of `EmploymentStatus::Employed` in `crates/adam-core/src` lies inside a
`#[cfg(test)]` module. Verified against the first `#[cfg(test)]` line of each file:
`accounting.rs:525 > 431`, `capacity_investment.rs:376 > 279`,
`cohort.rs:723,847,890 > 704`, `commerce.rs:357…1359 > 263`, `demand.rs:800,905 > 720`,
`distress.rs:368 > 268`, `health.rs:393,464 > 337`, `management.rs:241 > 148`,
`market.rs:926 > 841`, `migration.rs:595 > 518`.

**What it does:** `match_employment_worker` (`labor.rs:548`) creates an
`EmploymentAgreement` and emits `EmploymentMatched`, and never touches the cohort's status.

**Why it is wrong:** the engine has two disconnected notions of work. Agreements are real
and drive payroll and production. Status is a label that only ever moves toward
unemployment.

**Who pays:** nobody — but everything downstream reads the broken label.

**Consequences:**

- `political_economy.rs:99` computes unemployment as the population share of cohorts whose
  status is `Unemployed`. Measured unemployment is therefore permanently ~100 % in any world
  with working adults.
- That feeds `UNEMPLOYMENT_PRESSURE_PER_MONTH = 250` into social pressure, which lowers
  legitimacy, which *raises* `spending_bps` in A-4 and so increases the cash handed to
  households. **The main money pump is throttled by a sensor that is stuck at maximum.**
- `match_employment_worker` gates hiring on `employment() == Unemployed`, a condition that
  can never fail, so the gate is inert.

## B-2. Employment never changes a cohort's income — Critical

**Where:** writes to `annual_income` in production code are only `cohort.rs:279`
(`split_household`), `cohort.rs:327`/`:340` (`absorb`), `cohort.rs:366` (zeroed by
`leave_employment`), and `cohort.rs:472` (rescaled by `apply_excess_deaths`).

**What it does:** being hired at a wage does not raise `annual_income`. Payroll credits
`liquid_wealth` only. Losing a job sets `annual_income` to zero permanently.

**Why it is wrong:** `annual_income` is the engine's only representation of what a person
earns, and it is a fixture constant that decays to zero and never recovers.

**Who pays:** the household, permanently and invisibly.

**Consequences, all three load-bearing:**

1. **Wages collapse to one minor unit.** `competitive_labor_bid` (`labor.rs:1431`) computes
   `base = annual_income / people / 12`, then `wage = base.max(1) × (10_000 + urgency + adj) / 10_000`.
   For any cohort that has ever been unemployed, `base = 0`, so the offered wage is
   **1 minor unit per month**, forever. Unemployment is an absorbing state produced by an
   arithmetic accident.
2. **The borrowing ceiling closes when it is needed.** `coping.rs:3` caps survival debt at
   `2 × annual_income`, which is zero for the unemployed.
3. **The phantom income of A-2 is pinned to fixture values** rather than to anything the
   labour market did.

## B-3. The wage bid is circular — Major

**Where:** `labor.rs:1431` `competitive_labor_bid`.

**What it does:** `urgency = 500 + target × 1_500 / capacity` (so +5 % to +20 %), plus a
skill-market adjustment of ±500/±250 bps per three months of pressure, applied to the
cohort's own prior income per head.

**Why it is wrong:** the wage is derived from the wage. There is no anchor in the value of
the output, in the cost of living, or in a competing offer. Were B-2 fixed, this becomes a
ratchet: every rehire multiplies the previous income by at least 1.05 with nothing pulling
down.

**Who pays:** currently nobody, because B-2 freezes the input at zero. The two defects hide
each other.

## B-4. The wage ceiling ignores the tax the firm will owe — Major

**Where:** `labor.rs:1386` `recipe_labor_value_ceiling`.

**What it does:**
`ceiling = (price × output_per_batch − Σ input_price × input_qty) × 1_000 / labor_milli`,
all at **regional reference prices**.

**Why it is wrong:** the 20 % sales tax of A-7 is levied on the same revenue and is not
deducted. The ceiling therefore certifies as affordable a wage that leaves the firm exactly
20 % of final sales short. It also ignores capital costs and uses reference prices rather
than the prices the firm actually realises.

**Who pays:** the firm, out of capital, until it is insolvent.

**Consequence:** this is the direct mechanical cause of the `closed_circuit` death — revenue
equal to wages, then `FirmSalesTaxPaid { taxable_sales: 2400, liability: 480, paid: 480 }`
out of a cash balance that never had a margin to pay it from.

## B-5. Unpaid wages are free to the employer — Major

**Where:** `labor.rs:1596` `execute_monthly_payroll`; `arrears` on `EmploymentAgreement`.

**What it does:** `paid = min(firm.cash, owed)`; the remainder accumulates as `arrears`
indefinitely, without interest, penalty, or worker reaction.

**Why it is wrong:** workers keep producing while unpaid, forever. The only consequence
appears in `execute_observed_labor_shedding` (`labor.rs:1298`), where arrears are used as
permission for the *employer* to end the agreement.

**Who pays:** the worker, silently, with no ability to withdraw labour.

**Note:** the shedding rule itself is exemplary and should be preserved — it only fires when
`wage > ceiling` **and** arrears exist, with a comment naming exactly who finances the wait.
It is one of the few limits in the engine that passes the `AGENTS.md` test cleanly.

## B-6. Retirement fires the whole workforce at once — Minor

**Where:** `cohort.rs`, `advance_lifecycle_year`, `set_workers(0)` on entering `Senior`.

**Why it is wrong:** a cohort is a single object, so an entire age band leaves employment on
one day, roughly every thirty years, with no notice to the employer and no replacement
pipeline.

## B-7. Shedding releases every worker of a cohort at once — Minor

**Where:** `labor.rs:1298`, `change_employment_workers(firm, cohort, 0)`.

**Why it is wrong:** there is no partial layoff. The firm cannot shrink; it can only sever.

---

# C. Demography: two systems that do not know about each other

## C-1. Population is a money formula that overwrites starvation — Critical

**Where:** `simulation.rs:870` `population_rate`, applied in `apply_year` (`:596`) to both
the formula path and the material path.

**What it does:**

```
prosperity      = clamp((output_per_person − 3_000_000) / 2_000, ±3_000)
population_rate = clamp(4_000 + prosperity + (legitimacy − 5_000)/2 + centered(±1_000), ±30_000)
```

Then `apply_year` sets regional population from this rate and rescales cohorts to match.

**Why it is wrong:** three separate problems in one expression.

1. `3_000_000` is an absolute money threshold for prosperity, with no unit, no source, and
   no relation to what food costs in the world being simulated.
2. Births are a rate, not a process — exactly the case `AGENTS.md` names as magic.
3. It runs in the **material** path too, so the excess deaths computed honestly in
   `health.rs` from real consumption are overwritten by a formula reading a money aggregate.

**Who pays:** nobody. People appear because output per head crossed a constant.

## C-2. New people arrive free — Critical

**Where:** `apply_year` → `plan_region_cohort_rescale` → `set_people`.

**What it does:** cohort headcount changes; `liquid_wealth`, `debt`, and `annual_income`
are not adjusted for the arrivals.

**Why it is wrong:** added people immediately share the existing cohort's wealth per head
and supply labour that cost nothing to raise. There is no childhood, no dependency, no
upbringing cost.

**Who pays:** nobody. Labour and consumers are conjured together.

## C-3. Initial children can be loaded, but the world never produces another child — Critical (confirmed)

**Where:** `cohort.rs` `advance_lifecycle_year`, all production `HouseholdCohort::new`
call sites, and content configs.

`world.example.toml` may seed `age_band = "child"`, while the current demo seeds only adults.
No runtime system creates a new Child cohort. Lifecycle transitions only move existing cohorts
forward: Child → Youth → Adult → Mature → Senior. Annual population growth rescales the people
already present across their current bands instead of creating births, and Senior is terminal.

**Consequence:** any initially seeded children age out after thirteen years and are never replaced.
The whole population eventually becomes `Senior`/`Retired`, while C-1 keeps adding heads to those
surviving bands. The age structure is not merely approximate; reproduction does not exist.

## C-4. Starvation makes the survivors richer — Major

**Where:** `cohort.rs:472` `apply_excess_deaths`.

**What it does:** scales `households` and `annual_income` to the survivors; leaves
`liquid_wealth` and `debt` untouched.

**Why it is wrong:** the dead take their income with them but leave their whole cash balance
and their whole debt to the survivors. Wealth per head rises with the death toll; debt per
head rises too.

**Who pays:** nobody. Famine is a wealth transfer with no estate.

---

# D. Demand: a budget with no future and a signal from goods that do not exist

## D-1. Households spend their entire liquid wealth every month — Major

**Where:** `demand.rs:410`, `let mut remaining = i128::from(cohort.liquid_wealth().minor_units()).max(0);`

**Why it is wrong:** there is no consumption function, no propensity to save, no buffer
against next month. Saving can only happen by failing to buy. A household therefore cannot
accumulate for anything, and any cash it holds is evidence of scarcity rather than choice.

**Who pays:** nobody — it is the absence of a decision, not a limit.

## D-2. Upper tiers budget against goods that may not exist — Major

**Where:** `demand.rs`, `household_target_purchase`. For `tier == Survival` the planner
walks the real offer book; for every other tier it returns the full desired quantity priced
at the **regional reference price**, regardless of whether any seller offers the good.

**Why it is wrong:** budget is reserved for phantom purchases, and the resulting unmet
demand is unbounded. The shortage signal for a luxury nobody produces is structurally
larger than the shortage signal for bread.

**Who pays:** nobody, and the mis-signal is then spent by real actors — see E-1, and
`firm_entry.rs`/`capacity_investment.rs` which invest against it.

## D-3. Reserved money is locked to one good — Major

**Where:** `demand.rs` produces `DemandIntent { reserved_spend }`; `commerce.rs:46` turns
each intent into a `MarketOrder { max_spend }`; `market.rs` `fill_market_order` spends only
within that one order's budget.

**Why it is wrong:** if the good disappears between planning and clearing, the money does
not move to a substitute. The household holds cash and goes without.

**Consequence:** this is the mechanical explanation for idle household money in long runs.

## D-4. Substitution spreads money thinly instead of filling the cheapest need — Major

**Where:** `demand.rs`, `proportional_spends`.

**What it does:** when the tier budget is short, spending is spread proportionally across
every line in the tier, including several substitutes inside the same need group.

**Why it is wrong:** a hungry household buys a little of everything rather than filling its
calories with the cheapest source. Substitution is supposed to be a forced, ranked choice.

## D-5. Local supply always beats imports, at any price — Major

**Where:** `demand.rs`, `candidates.sort_by_key((imported, price, origin, seller))`;
mirrored in `market.rs` `market_candidates`, where local offers are pushed first and imports
appended after.

**Why it is wrong:** the `imported` flag outranks price. A local seller at any price is
preferred to a cheaper delivered import. Home bias may be realistic, but here it is
absolute and free.

**Who pays:** nobody. No consumer bears a cost for the preference and no producer pays for
the protection.

## D-6. The cost of living is measured with the wrong prices — Major

**Where:** `demand.rs`, `monthly_survival_cost`, called with an empty offer book.

**Why it is wrong:** it values the survival basket at regional reference prices, while
actual purchases happen at seller offer prices plus tariffs. Relief size (`relief.rs`) and
the survival borrowing decision (`coping.rs`) are therefore measured with a ruler that does
not match the shop.

## D-7. Rounding is biased toward more demand — Minor

**Where:** `demand.rs:667` area — `desired = (full × open + SCALE − 1) / SCALE` rounds up,
`covered = obtainable × SCALE / full` rounds down, and cost rounds up with `+ SCALE − 1`.

**Why it is wrong:** the residual share of a need is systematically larger than the true
remainder, and reserved budget systematically exceeds what will be spent (see H-2).

---

# E. Market and price formation

## E-1. Every seller is told the entire market's shortage is its own — Critical

**Where:** `market.rs`, `clear_market_with_delivery_preferences`. Unmet need accumulates
into `unmet_by_market` keyed by `(region, good)`, and then the `offer_outcomes` mapping
writes that **whole market total** into `unmet_market_demand` for **every** offer in that
region and good.

**What it does:** with three sellers and 100 units of unmet demand, all three outcomes
report 100.

**Why it is wrong:** the aggregate is counted once per seller. The error scales with the
number of competitors, so competition amplifies the perceived shortage instead of relieving
it — an inversion, not a bias.

**Who pays:** nobody; everyone acts on it.

**Consequence:** feeds `pricing.rs` `observed_market_pressure` → `revised_price`, feeds
`firm_entry.rs` (`ENTRY_SIGNAL_BPS = 3 × FULL_MONTH_BPS`), and feeds capacity investment.
All three over-react in proportion to how competitive the market already is.

## E-2. Deferred rationed demand is added to every offer as well — Critical

**Where:** `rationing.rs`, `restore_rationed_unmet_demand`, called in `commerce.rs:46`
after clearing and before settlement.

**What it does:** adds the full deferred quantity to each of the region's offer outcomes.

**Why it is wrong:** the same defect as E-1, applied a second time to the same signal, and
because it runs before `settle_local_market` the inflated figure is what lands in
`self.unmet_demand` and in deprivation pressure.

## E-3. Withdrawing from the market raises the inventory a firm withholds — Major

**Where:** `market.rs`, `average_observed_firm_sales`, used by `plan_firm_market_offers`
(`market.rs:114`) as `retained = average_sales × inventory_buffer_days / 30`.

**What it does:** the average divides total sales by the number of observations **in which
the firm had any market outcome**, skipping months in which it offered nothing.

**Why it is wrong:** a firm that stops offering keeps a high historical average, so it
retains a large buffer, so it offers even less. The denominator should be elapsed months.

**Consequence:** a self-reinforcing withdrawal spiral, and permanently withheld stock that
still counts as output through A-6.

## E-4. A firm with no policy record silently never trades — Major

**Where:** `market.rs:114`, `.filter_map(|firm| self.firm_policies.get(&firm.id())…)`.

**Why it is wrong:** a missing policy is treated as "no offers" rather than as an error or a
default. The firm produces, holds inventory, owes wages, and never sells. Nothing is logged.

## E-5. Price is anchored to a reference that only learns from pressure — Major

**Where:** `market.rs:114` sets `unit_price = reference × (10_000 + markup) / 10_000`, then
raises it to `firm_variable_unit_cost` if that is higher. The reference itself is moved by
`pricing.rs` `revised_price(previous, pressure, carry)` using
`SHORTAGE_SENSITIVITY_BPS = 2_500` and `SURPLUS_SENSITIVITY_BPS = 700`.

**Why it is wrong:** the reference never learns from the prices at which trades actually
settled. It is a random walk driven by a pressure figure that E-1 and E-2 inflate. Observed
behaviour: a reference price drifting 50 → 34 within a year while trades continued.

## E-6. Loyalty is last month's largest seller, not a relationship — Minor

**Where:** `market.rs`, `derive_household_supplier_preferences`, consumed by
`market_candidates` with the tolerance `preferred × 100 ≤ cheapest × 110`.

**Why it is wrong:** recomputed from scratch each month, so it has no memory and no cost to
break. The 110 % tolerance is an unowned constant: nobody pays for the buyer's willingness
to overpay.

## E-7. Deprivation pressure is dominated by goods that do not exist — Major

**Where:** `market.rs`, `settle_local_market`, weights `Survival 4, Participation 3,
Development 2, Discretionary 1` applied to consumption and unmet quantities.

**Why it is wrong:** because of D-2, unmet quantities in upper tiers are unbounded, so a
cohort registers deprivation over luxuries that were never produced. This flows into social
stress, then legitimacy, then A-4.

**Also:** the weights themselves are unnamed constants with no stated derivation.

---

# F. Public finance

## F-1. Spending has a floor nobody funds — Major

**Where:** `simulation.rs:11-21`, `SPENDING_FLOOR_BPS = 1_200`; applied in
`plan_material_country`.

**Why it is wrong:** with zero revenue and unbounded debt the state still spends 12 % of
output. The floor is not a policy choice by a modelled actor; it is a constant.

**Who pays:** nobody — and via A-4 the floor is precisely the money handed to households.

## F-2. The same money is spent twice — Critical

**Where:** `apply_year`, `simulation.rs:~697`:
`let service_budget = i128::from(country_update.spending.minor_units()).max(0) * 3_000 / 10_000;`

**Why it is wrong:** the full `spending` has already been distributed to households as cash
by A-4. Thirty per cent of it is then *also* used to fund public services. One pot pays two
bills.

**Who pays:** nobody. Public services are free.

**Also:** `3_000 / 10_000` is an unnamed constant in the function body, and the funding
target uses a second one: `funding_target = clamp(regional_budget × 100_000 / regional_output, 0, 10_000)`,
which silently defines "fully funded services" as 10 % of regional output.

## F-3. Interest is paid to the population at large — Major

**Where:** `plan_material_country`: `interest = debt × 300 / 10_000`, added to `spending`,
which is then distributed per head.

**Why it is wrong:** interest is a payment to a creditor. Distributing it by headcount means
the state pays interest to people who hold none of its debt — because, per A-4, nobody holds
any of it.

## F-4. Tax rate without a taxpayer, in the formula path — Major

**Where:** `simulation.rs:1008` `plan_country`: `revenue_bps = 1_600 + cohesion / 20`.

**Why it is wrong:** revenue is a share of output rather than a sum of collections. The
material path correctly uses actually-collected tax; this path invents it. Both paths remain
reachable through the public API (`advance_one_year` vs `advance_economic_year`).

## F-5. Surplus repays only half the debt — Minor

**Where:** `simulation.rs:1125` `close_budget`.

**Why it is wrong:** an unexplained 50 % split between debt reduction and treasury
accumulation, with no actor choosing it.

## F-6. Regional output cannot be negative — Minor

**Where:** `plan_material_regions`, `money_from_i128(measured.max(0), …)`.

**Why it is wrong:** a region that consumed more stock than it produced should report a
contraction. Clamping at zero hides inventory drawdown from every downstream consumer of
output, including the tax and spending formulas.

---

# G. Health and survival

## G-1. Twenty per cent of the survival need is free — Major

**Where:** `health.rs:5-8`, `HEALTH_MAINTENANCE_FULFILLMENT_BPS = 8_000`.

**Why it is wrong:** health is maintained at 80 % fulfilment, so the last fifth of the
survival basket has no consequence for anyone. There is also a dead point exactly at the
threshold where neither loss nor recovery applies.

## G-2. Damage and recovery are asymmetric by unexplained constants — Major

**Where:** `health.rs`, `HEALTH_LOSS_DIVISOR = 12` versus `HEALTH_RECOVERY_DIVISOR = 8`,
with loss taking `ceil(deficit / 12)` and recovery `surplus / 8`.

**Consequence:** total collapse takes about fifteen months, recovery about forty. Full
deprivation kills roughly 2 % per month, about 21 % per year. None of these figures is
derived from anything.

## G-3. A new cohort is born in perfect health — Minor

**Where:** `health.rs`, default `functional_capacity` for cohorts with no health record.

**Also:** `blended_with` takes the **maximum** of two mortality rates when cohorts merge,
rather than a population-weighted average.

## G-4. Relief arrives a month late — Major

**Where:** `commerce.rs:202`, `execute_observed_emergency_relief` is the final stage of the
month, after `clear_market`, after `settle_local_market`, and after
`update_monthly_cohort_health`.

**Why it is wrong:** the money cannot be spent until the following month's market, by which
time the health loss and any excess deaths for this month are already committed. Relief can
never prevent the death it was triggered by.

---

# H. Ordering, rounding, and accounting hygiene

## H-1. Demand plans against one capacity state and trades against another — Major

**Where:** `commerce.rs:46`:

```rust
let demand_intents = next
    .plan_monthly_household_demand_against_offers(&offers, &mut route_capacity.clone())?;
…
let mut clearing = crate::market::clear_market_with_delivery_preferences(
    &orders, &offers, &mut route_capacity, …)?;
```

**Why it is wrong:** planning consumes a **clone** of route capacity to model first-buyer
reservation, while clearing consumes the original. Late-canonical cohorts trim their budgets
for capacity that turns out to be free at settlement, and under-buy for no physical reason.

## H-2. Reserved cost rounds up, settled spend rounds down — Minor

**Where:** `demand.rs` reserves `(price × purchased + SCALE − 1) / SCALE`; `market.rs`
`fill_market_order` computes `affordable` and `spend` with floor division.

**Why it is wrong:** money is conserved — the household is debited exactly what the seller
and carrier receive — but the tier ladder deducts more than will be spent, so upper tiers are
underfunded by a systematic sliver.

## H-3. Rounding crumbs accumulate to carriers — Minor

**Where:** `market.rs` `fill_market_order`: `spend` is floored once on the delivered price
while `goods_spend` is floored separately on the goods price, and
`freight_spend = spend − goods_spend`.

**Why it is wrong:** the residue always lands in freight. Conserved, but a persistent tilt in
favour of carriers.

## H-4. Firms outrank households for scarce transport — Major

**Where:** `commerce.rs:46`. One shared `route_capacity` pool: B2B procurement consumes it
first, household survival imports compete for the remainder.

**Why it is wrong:** in a famine, an input shipment always outranks a food import, by
ordering rather than by price, contract, or authority. Nobody pays for the priority and
nobody can bid it away.

## H-5. Freight revenue is credited to the seller's region as output — Minor

**Where:** `plan_material_regions` sums `fill.spend`, which includes `freight_spend`, into
the **seller's** region.

**Why it is wrong:** the carrier's earnings are attributed to another region's output.
Misattribution across regions, and across countries it distorts the tax and spending base.

## H-6. Money conservation is never asserted — Major

**Where:** `market.rs` `validate_market_clearing` checks that
`offered == sold + unsold` and that outcome sales equal fill sales. There is no equivalent
check that debits equal credits, and `MoneyStockMeasured` records the stock without
comparing it to a prior period.

**Why it matters:** every defect in section A would have been caught on the first run by an
invariant asserting that the change in total money equals net minting by named actors.

---

# I. Value types

## I-1. `Money` has no invariants — Major

**Where:** `value.rs`, `Money` as a signed `i64` with no `checked_sub`, no non-negativity
rule, and no distinction between a balance and a flow.

**Why it matters:** negative balances are representable everywhere, so overdrafts are silent
rather than impossible or explicit.

## I-2. `BasisPoints::shifted` clamps silently — Minor

**Where:** `value.rs:73-76`.

**Why it matters:** saturation at 0 and 10 000 hides the magnitude of a pressure that
exceeded the scale, in exactly the places where that magnitude was the signal.

## I-3. `accumulator.rs` leaks only in empty months — Major

**Where:** `accumulator.rs:21`, `FULL_MONTH_BPS = 10_000`.

**Why it is wrong:** decay applies only in months with zero charge, so a persistent 1 %
shortfall accumulates without bound while never triggering the leak.

---

# J. Production and the firm as an object

## J-1. A firm's headcount ceiling is an immutable registration constant — Critical

**Where:** `production.rs`, `struct Firm { workers: u64 }`. The field is private and the
only mutators defined on `Firm` are `set_cash`, `debit_cash`, `apply_cash_delta`,
`debit_inventory`, `credit_inventory`, `add_capacity`, `remove_capacity`. **There is no way
to change `workers` after `Firm::new`.** It is enforced as a hard cap in two places:

- `labor.rs:1226` `firm_labor_vacancy`: `required = target × labor_milli / 1_000` then
  `.min(definition.workers())`;
- `labor.rs:1535` `change_employment_workers`: `firm_other + workers > definition.workers()`
  returns `InvalidEmployment`.

**Why it is wrong:** capital can grow — `add_capacity` is called by capacity investment and
by reorganisations — while the labour ceiling cannot. A firm that builds capacity can never
staff it: the vacancy calculation refuses to open the post and the hiring call refuses the
worker. `plan_monthly_production_internal` then reports `"labor"` as the limiting factor for
the rest of the firm's life.

**Who pays:** nobody. A firm's maximum size is a number typed into a fixture.

**Consequence:** growth in capital is structurally sterile. Money spent by
`capacity_investment.rs` buys batches that can never be worked. Pending confirmation in the
`firm_entry.rs` pass, `STARTUP_WORKERS = 1` (`firm_entry.rs:36`) permanently caps every
newly founded firm at a single worker, whatever the demand that justified founding it.

## J-2. A firm with no employment agreement works for free — Critical

**Where:** `production.rs`, `plan_monthly_production_internal`:

```rust
let effective_workers = if self
    .employment_agreements
    .values()
    .any(|agreement| agreement.firm() == firm.id())
{
    contracted_workers
} else {
    firm.workers()
};
```

**What it does:** with no agreement on file, the registration field is used as the
workforce. The firm produces at full registered headcount with no payroll, no cohort losing
available labour, and no wages owed.

**Why it is wrong:** labour is an input that must be bought from someone. Here it is
supplied by a struct field.

**Scope, verified:** this is not reachable again after the first hire.
`change_employment_workers` (`labor.rs:1535`) keeps the agreement row and merely calls
`set_workers(0)`, so the `any(...)` test stays true forever once a firm has ever signed.
The hole is open only *before* the first contract — which covers every fixture firm at world
start.

**Corrected in the L pass:** it does **not** cover firms created by `firm_entry.rs`.
`found_firm` registers an `EmploymentAgreement` inside the same atomic founding, so an
entered firm carries a contract from its first day. The free-labour window belongs only to
firms registered directly through `register_firm` — that is, the entire starting population
of firms in `demo.toml` and in every integration-test fixture.

**Who pays:** nobody. Output in those months is unpaid-for.

**Consequence:** the opening months of every run are subsidised, so the baseline the whole
demo world is calibrated against was produced by free labour. Any comparison between a firm
before and after hiring compares free labour with priced labour.

## J-3. Production never looks at money — Major

**Where:** `production.rs`, `plan_monthly_production_internal`. The limiting factors are
exactly `"insolvency"`, `"capital capacity"`, `"labor"`, `"management target"`,
`"intermediate input"`.

**Why it is wrong:** no price, no unit cost, no cash balance, and no expected margin enters
the monthly plan. A firm produces to capacity into a market that will not buy, and keeps
producing while every batch loses money. A margin test does exist —
`expected_operating_cash_margin`, which nets expected sales against expected input costs and
the full payroll including arrears — but it lives in `plan_observed_production_adjustments`,
which is advisory and only reaches production if management writes a target.

**Consequence:** with A-6, producing goods nobody buys is not merely unpunished, it is
rewarded through measured output.

## J-4. Capital never wears out and costs nothing to hold — Major

**Where:** `production.rs`, `execute_monthly_production`. It debits input inventory, credits
output inventory, records the batch, emits `ProductionCompleted`. **No cash moves at all.**

**Why it is wrong:** there is no depreciation on `capacity_batches`, no maintenance, no
energy, no rent, no fixed cost of any kind. Installed capacity is immortal and free.

**Who pays:** nobody.

**Consequence:** exit is never forced by fixed costs. The only recurring cost of existing is
payroll, and B-5 lets a firm simply not pay it. Nothing in the engine makes idleness
expensive, so no firm ever has a reason to shut a plant.

## J-5. Each firm plans to cover the whole market's shortage by itself — Critical

**Where:** `production.rs`, `summarize_market_evidence` takes
`.max()` of `unmet_market_demand` across a firm's outcomes; then
`plan_observed_production_adjustments` computes
`demand_ceiling = (average_sold + average_unmet).div_ceil(output_per_batch)` and, when
stockouts outnumber unsold months and the expected margin is positive, advises
`demand_ceiling.min(physically_feasible)`.

**Why it is wrong:** E-1 has already written the market-wide shortage into *every* seller's
outcome. Each of N firms therefore reads the entire market's missing quantity as its own
opportunity, and the industry collectively plans N times the shortfall.

**Consequence:** this closes the most complete broken loop found so far, and it is worth
stating end to end:

> unmet demand counted once per seller (E-1, E-2)
> → every firm plans to fill all of it (J-5)
> → physical glut
> → unsold stock valued into regional output (A-6)
> → higher `spending` handed to households as cash (A-4)
> → more nominal demand next year
> → the cycle repeats one size larger.

No actor pays anywhere along that chain, and every step is arithmetically sound in
isolation. This is why tuning has never fixed the run.

## J-6. Averages divide by months present, not months elapsed — Minor

**Where:** `production.rs`, `summarize_market_evidence`: `periods` counts only observations
that contain an outcome for the output good, exactly as in E-3. A firm absent from the
market keeps its historical averages intact and plans from them on return.

## J-7. Every person in a cohort is a potential worker — Major

**Where:** `labor.rs:1251` `available_cohort_workers` returns
`people − allocated_to_active_agreements`, with no age band, no dependency ratio, and no
participation rate. `labor.rs:1264` `select_qualified_unallocated_cohort` filters only on
status, education, recipe skill, absence from training, and availability.

**Why it is wrong:** the engine has no concept of a labour force distinct from a population.
A four-person household can supply four workers. Children are excluded only by whatever
status a fixture happened to give them; seniors only because `advance_lifecycle_year` marks
them `Retired`.

**Interaction with B-1:** the `Unemployed` filter in that selector is the same always-true
test, so the only real gates are education, skill, and the J-1 headcount cap.

## J-8. Health correctly reduces effective labour — sound, preserve

**Where:** `production.rs`, `effective = present × functional_capacity / BasisPoints::MAX`,
with `present = agreement.workers().min(cohort.people())`.

This is one of the few complete, owned loops in the engine: hunger lowers health, health
lowers output, and the firm bears the loss in batches. It must survive any fix to B-1 and
B-2.

---

# L. Firm entry and capital formation

## L-1. Founding is genuinely paid for, and the split conserves money — sound, preserve

**Where:** `firm_entry.rs:204` `found_firm`, `:363` `distribute_capital_outlay`, `:395`
`plan_regional_capital_shares`.

**What it does:** the founder's `actor_cash` is debited by `capital_cost + working_capital`
and refused outright if the cash is not there (`WorldError::InvalidActorCash`). The working
capital becomes the firm's opening cash. The capital cost is distributed to the founding
region's cohorts in proportion to population by the largest-remainder method, so the total
paid equals the total spent to the minor unit. The founder receives a `FULL`/`FULL`
ownership stake and a chief-executive appointment, a startup policy is written, an
employment agreement is registered, and a production target is set — all inside a cloned
world applied atomically.

**Why this matters:** it is the healthiest transaction in the engine, and it is the one real
outlet for firm profit that exists today:

> firm profit → owner distributions (ADR 0183) → actor cash → founding a works → household
> wealth → consumption.

Everything else in section A mints or burns. This path only moves money between named
balance sheets. Any repair must keep it.

## L-2. The plant itself is built out of nothing — Major

**Where:** `found_firm` calls `Firm::new(…, plan.capacity_batches, …)` directly;
`distribute_capital_outlay` moves only money.

**Why it is wrong:** the error message on the outlay says capital spending "needs local
households to build the works", but nothing is actually built. No labour-months are
consumed, no cohort is occupied for a month, no materials leave any inventory, and no time
passes. Capacity exists at full size the instant the money is paid.

**Who pays:** the founder pays cash; **nobody pays in real resources.** The households
receive a payment for work they never stopped doing something else to perform.

**Consequence:** a region's capital stock can grow without diverting one unit of output from
consumption. Investment has a monetary cost and no physical cost. Combined with J-4, where
capital never wears out, the capital stock is a ratchet. Note that `capacity_investment.rs`
does at least impose `CONSTRUCTION_MONTHS = 3`; entry imposes nothing.

## L-3. A factory costs exactly two batches of its own output — Major

**Where:** `firm_entry.rs:838` in `foundation_economics`:
`capital_cost = output_value × CAPACITY_VALUE_MULTIPLIER × batches`, with
`CAPACITY_VALUE_MULTIPLIER = 2` (`:38`), `output_value` priced at the regional reference
price, and a `.max(1)` floor.

**Why it is wrong:** an unnamed constant declares that building productive capacity costs
twice the sale value of one batch of what it will make. There is no construction sector, no
bill of materials, no engineering, and no relation to the cost of anything. The identical
multiplier appears again at `capacity_investment.rs:13`.

**Second-order effect:** because the cost is proportional to the output price, a doubling of
the good's price doubles the cost of the plant that makes it, so the profitability of entry
is invariant to price. The one signal that should govern entry is algebraically cancelled.

## L-4. Entry is judged at reference prices, before tax — Major

**Where:** `foundation_economics` reads `regional_prices` for the output good and for every
input; `working_capital = input_cost × batches + wage × 3 × staff`
(`WORKING_CAPITAL_WAGE_MONTHS = 3`).

**Why it is wrong:** the same defect as B-4. The founder evaluates the venture with a price
that E-5 shows is a pressure-driven random walk rather than a price anyone paid, and the
20 % sales tax of A-7 is absent from the calculation. Every founding therefore overstates
revenue by a fifth before the first brick.

**Sound part worth keeping:** the working-capital rule costs materials for *every* batch the
works will run, and the comment explains exactly why costing one batch and building many
opens a firm that cannot buy what it must convert. That reasoning is correct.

## L-5. A founded firm is capped at one worker forever — Critical

**Where:** `STARTUP_WORKERS = 1` (`firm_entry.rs:36`) is passed as `plan.workers` into
`Firm::new`, and `Firm.workers` is immutable (J-1).

**Why it is wrong:** the works may open with several `capacity_batches`, but
`firm_labor_vacancy` caps required staff at `workers = 1` and `change_employment_workers`
refuses any more. Whatever size of hunger justified the entry, the answer is one person, and
it stays one person for the life of the firm.

**Consequence:** ADR 0187 states that a works opens at the size of the hunger it answers.
Its capacity does; its workforce does not. Every entered firm is therefore permanently
labour-limited, producing `1_000 / labor_milli_worker_months` batches regardless of the
capacity it paid for. The capital cost of L-3 buys batches that can never be run.

## L-6. Founding clones the whole world — Minor

**Where:** `let mut next = self.clone();` in `found_firm`, in addition to the two full world
clones already taken every month in `commerce.rs`. Correct for atomicity, expensive.

---

# M. Firm credit

This is the best-built subsystem in the engine. Every money movement is a transfer between
named balance sheets, and the failure modes below are gates and starvation, not leaks.

## M-1. Lending and repayment conserve money — sound, preserve

**Where:** `insolvency.rs:371` `underwrite_firm_credit_offer`, `:562`
`accept_firm_credit_offer`, `issue_scheduled_firm_credit`, `execute_monthly_firm_debt_service`.

**What it does:**

- **Issue:** the creditor's `actor_cash` is checked against the principal, debited by it, and
  the firm's cash is credited by the same amount via `apply_cash_delta`. A
  `FirmCreditorClaim` records firm, creditor, priority, principal, accrued interest and
  schedule.
- **Service:** `interest_charge = ceil(principal × annual_bps / 120_000)` on the outstanding
  balance; `paid = min(firm cash, accrued interest + principal installment)`; interest is
  satisfied first; the firm's cash is debited and **the creditor's `actor_cash` is credited by
  exactly the same amount**. The claim is deleted only when principal and interest both reach
  zero.
- Unpaid installments stay outstanding while `installments_remaining` still decrements, so at
  term end the whole remaining balance is overdue.

Contrast with A-1: a household borrowing for food mints the money, while a firm borrowing for
working capital takes it from a named lender. The two mechanisms in the same engine follow
opposite rules.

## M-2. A firm may hold exactly one loan in its lifetime until it repays — Major

**Where:** `credit_market.rs:185`, applicant filter:

```rust
if next.is_firm_insolvent(firm)
    || next.firm_operating_history.get(&firm).map_or(0, Vec::len) < 3
    || next.firm_creditor_claims.keys().any(|(candidate, _, _)| *candidate == firm)
{ continue; }
```

plus `issue_*_firm_credit`, which refuses a duplicate `(firm, priority, creditor)` key.

**Why it is wrong:** any existing claim, at any priority, from any creditor, disqualifies the
firm from the entire credit market. A firm hit by a second shock while still amortising the
first loan cannot borrow at any price. There is no syndication, no second lien, no
refinancing, and no prepayment.

**Who pays:** nobody. No lender declines and no covenant forbids it; the market simply skips
the firm.

## M-3. The best-informed lenders are excluded by rule — Major

**Where:** `credit_market.rs`, candidate filter:
`request.actor != creditor && !next.ownership_stakes.contains_key(&(request.firm, creditor))`,
together with `request.country == creditor_country`.

**Why it is wrong:** an owner cannot lend to the firm they own, and no one can lend across a
border. Owner lending is the most common form of small-firm finance in reality, and it is
forbidden here by a filter rather than by any cost, law, or authority in the model.

**Who pays:** nobody. `AGENTS.md` requires institutional resistance to be represented by
actual authority or enforcement, not by a silent veto.

## M-4. The credit market is starved by the profit defect, not broken — diagnosis

**Where:** `autonomous_lender_capacity` reads `actor_cash`, retains
`LENDER_LIQUIDITY_RESERVE_BPS = 5_000` (half), and caps exposure at
`MAX_PORTFOLIO_SHARE_BPS = 4_000` of liquid wealth plus claims.

**Why the market is observed dead:** actor cash exists only through owner distributions
(ADR 0183), which require distributable profit. B-4 lets a firm agree to wages that consume
the whole margin, and A-7 then takes 20 % of revenue regardless of profit, so firms rarely
have profit to distribute. No profit → no actor cash → no lending capacity → no credit,
whatever the demand for it.

**Conclusion:** do not repair `credit_market.rs`. It will start working by itself once B-4
and A-7 stop consuming the margin. This also explains why
`demo_credit_market_rescues_one_case_and_refuses_concentrated_second_case` is fragile: it
depends on a lender happening to hold cash in a world where profit is structurally absent.

## M-5. A directly issued scheduled loan carries zero interest — Minor

**Where:** `issue_scheduled_firm_credit` writes `annual_interest: BasisPoints::ZERO` into the
schedule; `accept_firm_credit_offer` overwrites it with the agreed rate immediately after.

**Why it matters:** the rate lives outside the issuing function, so any future caller that
issues scheduled credit without going through offer acceptance creates a free loan. Today
both call sites (`credit_market.rs:358`, `distress.rs:1498`) go through the command, so this
is latent rather than active.

## M-6. Sound details worth keeping

- Applications are gathered before any lender commits, so canonical firm order does not decide
  who gets scarce capital.
- Lenders rank candidates by `rate + collateral cushion − distress charge`, with the distress
  charge (`400 bps` per distressed month) deliberately heavier than the rate premium, so a
  desperate borrower cannot outbid a sound one for scarce capital. The comment says exactly
  that.
- The borrower accepts the cheapest offer covering at least `MINIMUM_GAP_COVERAGE_BPS = 2_500`
  of its gap — a borrower's choice, correctly located in behaviour rather than in a bound.
- Monthly interest rounds up, in the creditor's favour, consistently.

---

# N. Price formation

## N-1. Correction to E-1: price formation is not the victim — amendment

**Where:** `pricing.rs:135` in `observed_market_pressure`:

```rust
entry.unmet = entry.unmet.max(outcome.unmet_market_demand.get());
```

with the doc comment: "`unmet` is a market-wide figure that every seller in the region
repeats, so it is taken as a maximum rather than a sum."

**Amendment:** section E-1 is correct that every seller's outcome carries the whole market's
shortage, but it overstated the damage. Price formation **knows** the figure is repeated and
deliberately de-duplicates it. The consumers that do **not** de-duplicate are:

| Consumer | Line | Treatment | Verdict |
| --- | --- | --- | --- |
| Price formation | `pricing.rs:135` | `.max()` across all sellers in the market | correct |
| Production advice | `production.rs:677` | `.max()` across **one firm's** observations | J-5, wrong |
| Capacity investment | `capacity_investment.rs:178` | reads `average_unmet_market_demand` from that same per-firm evidence | inherits J-5 |
| Stockout test | `market.rs:99` | `offered > 0 && unsold == 0 && unmet > 0` | correct as a flag |

So the E-1 chain runs into **capacity building**, not into prices. That is worse, not better:
an inflated price would at least self-correct through demand, whereas an inflated plan becomes
a physical plant that then has to be fed.

## N-2. The tax-aware break-even price is dead code — Critical

**Where:** `pricing.rs:232` `firm_break_even_unit_price`, and `pricing.rs:~270`
`observed_taxed_sales_share_bps`.

**Verified by search:** `firm_break_even_unit_price` has **exactly one occurrence in the whole
crate — its own definition.** Nothing calls it. The only cost floor actually applied is
`firm_variable_unit_cost`, at `market.rs:161`.

**Why it is wrong:** the function is correct and its documentation states the problem
precisely: a shop that ignores the levy "pays it out of capital and empties its till at a
steady rate no amount of selling can arrest." The share of taxed turnover is even learned from
the firm's own observed final sales rather than assumed. All of that reasoning is written,
tested by construction, and then not connected.

**Consequence:** every retail seller prices at variable cost while owing 20 % of revenue in
sales tax, so it is structurally guaranteed to lose money on every unit sold at the floor.
This is the exact mechanism the `circuit_probe.rs` run observed as "the farm dies of sales tax
paid out of capital while households get richer." **The fix already exists in the file; only
the wiring is missing.** This is the single highest-value repair found so far, because it is
one call site rather than a redesign.

## N-3. Wholesale trade is credited to the buyer's region, not the seller's — Major

**Where:** `pricing.rs`, in `observed_market_pressure`:

```rust
for (&(buyer, good), &(quantity, _)) in &self.monthly_firm_procurement_purchases {
    let Some(firm) = self.firms.get(&buyer) else { continue; };
    let entry = pressures.entry((firm.region(), good)).or_default();
    entry.sold = entry.sold.saturating_add(quantity.get());
}
```

**What it does:** `firm` here is the **buyer**, so the quantity is added to the sales figure of
the **buyer's** regional market.

**Why it is wrong:** when a mill in region 3 buys grain from a farm in region 1, region 3's
grain market records a sale that never happened there, and region 1's grain market never sees
the sale it actually made. The importing region reads as well-supplied and its price falls;
the exporting region reads as slack and its price falls too. Both prices move the wrong way,
and the more inter-regional trade there is, the more wrong they get.

**Who pays:** nobody. It is an attribution error, not a rule.

**Note:** the shortfall loop immediately below it uses `(region, good)` keys taken from
`monthly_firm_effective_input_shortfalls`, which appear to be recorded against the correct
region. Confirm the recording site in the `procurement.rs` pass.

## N-4. The reference price never learns what was actually paid — Major (mechanism for E-5)

**Where:** `market.rs:161` sets the ask as `reference × (1 + markup)`, then
`.max(firm_variable_unit_cost)`. `pricing.rs` revises the reference only from quantities:
`offered`, `sold`, `unsold`, `unmet`.

**Why it is wrong:** when a seller's variable cost exceeds the marked-up reference, every unit
trades at the cost floor — but the reference never observes that. The whole market can trade
at 80 while the reference sits at 50, and the reference will only move if quantities happen to
signal scarcity. Since the reference is the anchor for the ask, for demand budgeting (D-6),
for entry economics (L-4), and for inventory valuation (A-6), a stale reference propagates
everywhere at once.

**Consequence:** this is the mechanism behind the observed drift of the reference price from
50 to 34 over a year while goods were selling. Quantities alone cannot locate a price level;
they can only push it around.

## N-5. Sound parts worth preserving

- **Shortage raises a price only when the shelf emptied:**
  `shortage_share` is non-zero only if `unmet > 0 && traded > 0 && unsold == 0`. The comment is
  exactly right — unfilled need beside unsold stock is unpaid need, not effective demand, and
  households without money cannot bid a price up. This is the correct treatment of D-3.
- **No monthly cap on the move,** with the reason named: "A ceiling would mean somebody chose
  to hold a price down, and in this model nobody is paying for that choice." This is `AGENTS.md`
  applied correctly.
- **Costs deliberately excluded from the reading** (ADR 0189): a seller that cannot cover cost
  withdraws supply rather than pricing under it, and the withdrawal is what price observes.
- **Sub-unit pressure is carried,** with the remainder clamped to ±999 so a bound-suppressed
  price cannot bank pressure and jump later.
- **Asymmetric sensitivity** (`2_500` up, `700` down) is an elasticity, not a bound, and the
  comment says so — though neither number is derived from anything.

---

# O. Capacity investment

## O-1. Every sold-out firm builds for the whole market's shortage — Critical (E-1 → J-5 → here)

**Where:** `capacity_investment.rs:178` in `observed_capacity_expansion_terms`:

```rust
let wanted = proposal.average_unmet_market_demand.get().div_ceil(output_per_batch);
```

**Why it is wrong:** `average_unmet_market_demand` arrives from
`production.rs:597`, which took it from `summarize_market_evidence`, which took the `.max()`
of a figure that E-1 already stamped on every seller in the region: the **market-wide**
shortage. So each firm sizes its expansion to cover the entire market's unfilled demand by
itself. With N sold-out firms, the industry commissions N times the missing capacity, all in
the same month, all with a three-month lag before any of it appears.

**Who pays:** each firm pays cash for its own share, so the money is real — but the *reason*
is counted N times.

**Consequence:** a synchronised overbuild three months after every shortage, followed by a
glut, followed by A-6 valuing the unsold stock as regional output. This is where the E-1
chain does its real damage; N-1 shows price formation was never the victim.

## O-2. Purchased capacity can never be staffed — Critical (the decisive consequence of J-1)

**Where:** the project completes and calls `Firm::add_capacity`, raising `capacity_batches`.
`Firm.workers` is immutable (J-1), and `labor.rs:1226` caps required staff at
`.min(definition.workers())` while `labor.rs:1535` refuses any hire beyond it.

**What actually happens, end to end:**

1. The firm sells out, expects a positive margin, and commits cash.
2. `distribute_capital_outlay` pays that cash to local households as building income.
3. Three months later `add_capacity` raises `capacity_batches`.
4. `plan_monthly_production_internal` computes `labor_batches` from the **unchanged** worker
   count, and the limiting factor becomes `"labor"` permanently.
5. Output does not rise by one unit.

**Who pays:** the firm pays in full and receives nothing. Nobody is on the other side of the
failure — no builder went bankrupt, no worker refused, no skill was missing. It is J-1's
unowned cap collecting the bill.

**Consequence, and this is the important one:** capacity investment is a **pure one-way
transfer from firm tills to household wealth with zero productive return.** Combined with
A-6 (unsold stock counted as output) and A-4 (the state handing out cash it never collected),
this is a direct, mechanical explanation of the pattern observed in every long run:
households accumulate money while firms decapitalise and die. The firm's reward for reading
the demand signal correctly is to be drained.

**Corollary:** since `observed_capacity_investment_need` feeds the credit market's funding
gap, a firm can also **borrow** to build capacity it can never staff — then owe interest on
dead capital until it defaults.

## O-3. The investing actor is whoever sorts first — Minor

**Where:**

```rust
fn investment_actor(&self, firm: FirmId) -> Option<ActorId> {
    self.actors().keys().copied().find(|actor|
        self.can_perform_corporate_action(*actor, firm, CorporateAction::ProposeMajorInvestment))
}
```

**Why it is wrong:** the decision is attributed to the lowest-numbered actor holding the
authority, not to the owner, the chief executive, or anyone with a stake in the outcome.
Deterministic, but the identity is an artefact of identifier order.

## O-4. A batch of capacity costs two batches of output, again — Major

**Where:** `batch_cost = quantity_value(reference_price, output_per_batch) × CAPACITY_VALUE_MULTIPLIER`,
with the same constant `2` as `firm_entry.rs`. Identical defect to L-3: construction has no
cost basis of its own, and because the cost is proportional to the output price, a price rise
cannot make expansion relatively more attractive. The comment even points at `firm_entry` for
consistency — the two are consistently unfounded.

## O-5. Sound parts worth preserving

- **The money is real and it moves:** `CommitFirmInvestment` takes it from the firm, and
  `distribute_capital_outlay` pays it to local households. The comment states the principle
  exactly: "Without this the money would sit outside every balance in the world." Contrast
  A-1 and A-4, which do not bother.
- **The gates are the firm's own evidence,** not a general market impression:
  `stockout_observations > unsold_observations`, a positive expected operating margin,
  `market_demand_ceiling_batches >= capacity_batches`, no project already open, and not
  insolvent.
- **The firm can only build what it can pay for:** `affordable = cash / batch_cost`, and the
  comment correctly notes that this till already includes anything borrowed for exactly this
  purpose.
- **A three-month construction lag** is declared as a physical lag with an honest note that
  construction is not yet a modelled industry — the right way to record a placeholder.

---

# P. Public spending, reserves and programs

This answers open question 4. The state's **purchases** are real. The state's **borrowing** is
not.

## P-1. Reserve procurement is a genuine purchase — sound, preserve

**Where:** `public_reserve.rs:171` `procure_government_reserve`.

**What it does:** refuses if `treasury < cost` (`InsufficientTreasury`), then
`debit_inventory(good, quantity)` from the selling firm, `apply_cash_delta(cost)` **to** the
selling firm, and `set_treasury(treasury − cost)`. Goods leave a named seller, money leaves
the treasury, and the seller is paid the same amount.

This is exactly what A-4 fails to do. The state can buy grain correctly and hand out cash
incorrectly in the same run.

## P-2. Emergency relief is a genuine transfer — sound, preserve

**Where:** `relief.rs:238` `fund_emergency_relief`. Requires a current office holder's
authority, refuses if `treasury < amount`, then `set_treasury(treasury − amount)` and
`credit_wealth(amount)` on the receiving cohort. Money moves; nothing is created.

## P-3. The state borrows from nobody — Critical

**Where:** `relief.rs:334` `issue_emergency_relief_debt`:

```rust
let debt     = indicators.public_debt().minor_units() + amount.minor_units();
let treasury = indicators.treasury().minor_units()    + amount.minor_units();
country_state.indicators_mut().set_public_debt(Money::from_minor_units(debt));
country_state.indicators_mut().set_treasury(Money::from_minor_units(treasury));
```

**What it does:** raises the debt and the treasury by the same amount in the same breath.

**Why it is wrong:** there is no lender. No actor's cash falls, no household's wealth falls,
no bank exists. The money is created at the moment of borrowing. There is no bond market, no
buyer to refuse the issue, no interest rate set by anyone, and therefore no crowding out: the
state never competes with firms for the savings that M-4 shows are so scarce that the credit
market cannot function.

**Who pays:** nobody. The gates are an authority check, a policy-strategy check, and
`emergency_debt_headroom` — all rules, none of them a creditor.

**Compare M-1 directly.** In the same engine:

| Borrower | Mechanism | Lender's balance |
| --- | --- | --- |
| A firm | `issue_scheduled_firm_credit` | falls by the principal |
| A household | `borrow_for_survival` (A-1) | does not exist |
| The state | `issue_emergency_relief_debt` | does not exist |

**Consequence chain:** the state creates money by borrowing (P-3), pays interest on it to
households (F-3), and writes off 40 % of the principal when the burden gets heavy (A-3, F-5).
Every leg of the public debt cycle creates or destroys money outside any transaction. This,
not spending itself, is why fiscal restraint never bites in a run.

## P-4. A debt-funded program spends money that never enters the treasury — Critical

**Where:** `government_program.rs:482` `appropriate_government_program`:

```rust
let new_treasury = match source {
    ProgramFundingSource::Treasury  => opening.treasury().minor_units() - amount.minor_units(),
    ProgramFundingSource::PublicDebt => opening.treasury().minor_units(),   // unchanged
};
let new_debt = match source {
    ProgramFundingSource::Treasury  => opening.public_debt().minor_units(), // unchanged
    ProgramFundingSource::PublicDebt => opening.public_debt().minor_units() + amount.minor_units(),
};
```

and then `next_program.record_appropriation(year, amount)` credits the program's
`carryover_funding` with the full amount either way.

**Why it is wrong:** with a `Treasury` source the appropriation is correct — cash leaves the
treasury and becomes program funding. With a `PublicDebt` source **nothing is debited
anywhere**: the treasury is untouched, the debt rises, and the program acquires spendable
funding that exists in no cash balance in the world. It is worse than P-3, which at least
routed the invented money through the treasury.

**Consequence:** any program funded by debt pays its wages (P-5) out of money that is created
at the moment of payment.

## P-5. Public works pay one cohort everything and take no one out of the labour pool — Major

**Where:** `government_program.rs:~855` in `execute_government_program`:

```rust
let recipients: Vec<_> = next_cohorts.values()
    .filter(|cohort| cohort.region() == *region
        && cohort.employment() == crate::EmploymentStatus::Unemployed)
    .map(HouseholdCohort::id).collect();
if let Some(cohort) = recipients.first() {
    next_cohorts.get_mut(cohort)?.credit_wealth(*wages)?;
}
```

Three separate defects in nine lines:

1. **The whole regional delivery is paid to one cohort.** `wages` is `*delivered` — the entire
   delivered funding for that region — and it goes to `recipients.first()`, the lowest
   `CohortId`. There is no wage per worker; the amount paid is the budget, however many people
   worked.
2. **B-1 makes the eligibility filter meaningless.** Since no cohort is ever marked `Employed`,
   every cohort in the region qualifies, so "unemployed people get the work" is decided by
   identifier order.
3. **The workers are never withdrawn from the labour pool.** The count goes into
   `government_program_labor_usage`, but no employment agreement is created and
   `available_cohort_workers` (J-7) is not reduced. The same people remain fully available to
   private firms in the same month. **Public works cost the private economy no labour at all.**

**Who pays:** nobody. A real public-works programme competes with firms for hands and bids
wages up; here it is free.

## P-6. Program materials are properly consumed — sound

**Where:** `execute_government_program` reduces government reserve stock by the consumed
quantity and records `material_consumption`. Those goods were bought from firms with treasury
cash in P-1, so the physical leg of a programme is honest end to end: bought, held, consumed.

## P-7. The public-debt ceiling is an unowned output multiple — Critical

`emergency_debt_headroom` sets the limit to exactly twice current annual regional output minus
public debt. There is still no creditor, auction, interest-rate bid, maturity structure or buyer
balance sheet. Output does not hold the line and loses nothing when the state borrows.

The ceiling therefore hides unlimited unsecured money creation behind a global 200% coefficient.
The correct replacement is a funding market and named balance sheets, not a different ratio.

---

# Q. Political economy: the sensor that drives the money pump

## Q-0. Why this section matters more than it looks

The chain is short and it is the largest money creator in the model:

> cohort pressure (`political_economy.rs`) → `country_social_pressure` → `legitimacy_effect`
> → `legitimacy` → `spending_bps = 2_200 + (10_000 − legitimacy)/10 − debt_brake`
> → `spending` → `distribute_public_outlay` → household cash (A-4).

Every input to that chain is broken, and each one is stuck at or near its maximum, so the
pump runs at full pressure permanently.

## Q-1. All three pressure components are pinned — Critical

**Where:** `political_economy.rs` `derive_regional_social_pressure`.

**Component 1 — chronic unemployment:**

```rust
let unemployment = if cohort.employment() == EmploymentStatus::Unemployed {
    experience.unemployment_months().min(24).saturating_mul(250).min(10_000)
} else { 0 };
```

B-1 proves the condition is **always true** — nothing outside tests ever sets `Employed`. So
every cohort accrues unemployment months regardless of whether its people are working under
an employment agreement, and after 24 months the term saturates at `24 × 250 = 6_000` bps
forever. Note the `.min(10_000)` is dead code: the product can never exceed 6 000.

**Component 2 — public service shortfall:**
`service_shortfall = 10_000 − mean(healthcare, infrastructure, administration)`. Observed runs
sit at 9 334–10 000, i.e. essentially the maximum, because services are barely delivered.

**Component 3 — livelihood stress:** `(unrest × 3 + debt_distress) / 4`, the only component
that actually varies.

**Combined:** `(unemployment × 2 + livelihood × 3 + service_shortfall) / 6`. With components 1
and 2 pinned, the floor is `(6_000 × 2 + 0 + 10_000) / 6 = 3_667` bps before anything real
happens in the world.

**Then:** `legitimacy_effect = (3_000 − pressure) / 20`. At the structural floor of 3 667 the
effect is already `−33` bps per year, and it can never be positive unless combined pressure
falls below 3 000 — which the two pinned components make arithmetically impossible.

**Consequence:** **legitimacy can only fall.** Falling legitimacy raises `spending_bps`, which
raises `spending`, which A-4 hands to households as money nobody collected. The single most
important feedback loop in the fiscal system is driven by a sensor whose reading is a
constant, and the constant points in the inflationary direction.

## Q-2. The unemployment sensor cannot ever read "employed" — Critical (B-1 consequence)

This is worth stating separately from B-1 because of where it lands. B-1 is a bookkeeping
omission; Q-2 is its price. Even if every worker in the world were under contract and paid,
this function would still report 6 000 bps of chronic unemployment and still push legitimacy
down every year. **No amount of successful economic policy can register in the political
system.**

## Q-3. Region population and cohort population are weighted inconsistently — Major

**Where:** `derive_regional_social_pressure` weights each cohort by `cohort.people()`, while
`country_social_pressure` weights each region by `region.population()`.

**Why it is wrong:** C-1 records that the annual monetary population formula overwrites region
population independently of what happens to cohorts, so the two figures drift apart over a
run. After the drift, a region's political weight is not the number of people whose pressure
was measured. Nothing reconciles them.

## Q-4. The state's borrowing limit grows with the glut — Critical (new chain)

**Where:** `relief.rs:486` `emergency_debt_headroom`:

```rust
let output = Σ region.annual_output() for the country;
let limit  = output × 2;
headroom   = (limit − public_debt).max(0)
```

**Why it is wrong:** this is the only quantity standing between the state and unlimited money
creation in P-3 and P-4, and it is an unowned bound — no creditor refuses, no saver runs out.
Worse, its input is corrupted: A-6 counts **unsold inventory** as regional output, and O-1
guarantees synchronised overbuilding, so:

> firms overbuild (O-1) → unsold stock → measured output rises (A-6) → the state's debt
> ceiling rises → more money created out of nothing (P-3, P-4) → more household cash → more
> nominal demand.

A glut of unsellable goods **increases** the state's capacity to create money. That is the
opposite of every real fiscal constraint.

## Q-5. Office holders are selected by identifier order — Minor

**Where:** `relief.rs:522` `emergency_relief_actor` takes `.min()` of the political-office
holders; `capacity_investment.rs` `investment_actor` takes the first authorised actor (O-3).
Deterministic, but the identity of the decision-maker is an artefact of numbering rather than
of standing, and both will silently change behaviour if identifiers are ever reordered.

## Q-6. Unnamed weights

`unrest × 3 + debt / 4`, `unemployment × 2 + livelihood × 3 + shortfall / 6`, the neutral
point `3_000`, the divisor `20`, `250` bps per unemployed month, `125` per distressed month,
and the 24-month experience cap are all undefended constants sitting directly upstream of the
fiscal pump.

---

# R. Firm accounting, earnings and distributions

## R-1. The firm's accounts have no cost side at all — Major (structural)

**Where:** `accounting.rs`, `struct FirmMonthlyAccounts`:

```rust
sales_revenue, final_sales_revenue, wages_owed, wages_paid, wage_arrears, produced_batches
```

**Why it is wrong:** there is no field for input costs, sales tax, interest, freight, or
capital spending. **A profit and loss statement does not exist anywhere in the engine.** The
only earnings figure is computed ad hoc in `observation.rs` at month close, and every other
consumer that needs profitability has to rebuild it from scratch — which is why
`expected_operating_cash_margin` (J-3), `plan_cash_constrained_staffing` (R-6) and
`observed_firm_obligation_terms` each net out a different set of items.

## R-2. Earnings exclude tax and debt service — Critical

**Where:** `observation.rs:~197`:

```rust
let earned = sales_revenue − wages_paid − input_spend;
retained_earnings += earned;
```

**What is missing from that line:**

| Obligation | Charged where | In `earned`? |
| --- | --- | --- |
| Wages | `execute_monthly_payroll` | yes, but see R-3 |
| Inputs | `monthly_firm_procurement_purchases` | yes |
| **Sales tax, 20 %** | `plan_firm_sales_taxes`, once a year | **no** |
| **Loan interest and principal** | `execute_monthly_firm_debt_service` | **no** |

**Why it is wrong:** retained earnings therefore overstate what the firm actually earned by
the whole of its tax and debt bill. Since `distributable = min(prudent_cash, retained)`, the
firm is licensed to hand owners money it already owes the state and its creditor.

**Consequence — this closes the loop on the observed farm death:** the firm books a profit
monthly, distributes it monthly, and then meets a 20 % annual sales-tax bill from a till that
has already been emptied. It pays the tax out of capital and dies solvent-looking right up to
the moment it is not. `circuit_probe.rs` observed exactly this. Note that N-2 already contains
half the fix — a break-even price that includes the levy — and it is not wired in either.

## R-3. Unpaid wages increase distributable profit — Critical

**Where:** the same line uses **`wages_paid`**, not `wages_owed`.

**Why it is wrong:** B-5 already lets a firm defer wages at no cost. Here that deferral is
rewarded twice over: the unpaid amount is not deducted from earnings, so retained earnings
rise by exactly the wages the workers did not receive, and the owners may then take that money
out as a dividend.

**Who pays:** the workers, who are owed the money, and who have no mechanism to object.

**Consequence:** the cheapest way for a firm to show a profit is to stop paying its staff. In
a model whose whole purpose is a believable economy, this is the single most perverse
incentive found so far.

## R-4. Multi-cohort arrears are overwritten, not summed — Major

**Where:** `accounting.rs`, `record_firm_payroll`:

```rust
FirmMonthlyAccounts::add(&mut row.wages_owed, owed, "firm wages owed")?;
FirmMonthlyAccounts::add(&mut row.wages_paid, paid, "firm wages paid")?;
row.wage_arrears = arrears;      // assignment, not accumulation
```

**Why it is wrong:** `wages_owed` and `wages_paid` accumulate across the firm's employment
agreements, but `wage_arrears` is **assigned**. A firm employing two cohorts reports only the
arrears of whichever agreement was processed last — `BTreeMap` order, so the highest
`CohortId`. The reported arrears of the earlier cohort silently vanish from the accounts.

## R-5. Staffing advice can tell a one-worker firm to fire its only worker — Major

**Where:** `accounting.rs`, `plan_cash_constrained_staffing`:
`affordable = workers × coverage_bps / 10_000`, truncating.

**Why it is wrong:** with `workers = 1`, any coverage below 10 000 — even 9 999 bps, a shortfall
of one part in ten thousand — yields `affordable = 0`. Because L-5 gives **every** firm founded
by `firm_entry.rs` exactly one worker, and J-1 forbids it from ever having more, every entered
firm is one rounding error away from being advised to dismiss its entire workforce.

## R-6. The solvency forecast omits the same obligations — Major

**Where:** the forecast branch of `plan_cash_constrained_staffing`:

```rust
available = firm_cash + expected_sales_revenue + expected_financing − expected_input_costs
```

**Two problems:** tax and debt service are again absent, so the firm believes it can afford
staff it cannot; and `expected_financing` is the sum of **outstanding credit offers**, counted
as though already drawn. A firm can therefore feel solvent on the strength of an offer it has
not accepted and, under M-2, may not even be allowed to accept.

## R-7. Dividend remainder goes to the lowest-numbered owner — Minor

**Where:** `board.rs`, `execute_dividend`: shares are computed by truncation and the leftover
minor units are added to `payouts.first_mut()`. Deterministic and money-conserving, but the
beneficiary is an artefact of `ActorId` ordering — the same pattern as O-3 and Q-5.

## R-8. What is genuinely well built here — preserve

The distribution path is one of the most carefully reasoned parts of the engine, and the
reasoning is written down in the code:

- **Capital cannot be distributed.** `distributable = min(prudent_cash, retained_earnings)`
  with the comment: "A full till proves nothing on its own: the opening endowment, an owner's
  injection against arrears and a lender's principal all sit in the same till as trading
  income. Only the trading income was ever profit." (ADR 0186.)
- **Prudence comes before profit.** `prudent_cash = free_cash − observed_prudence_reserve`,
  where the reserve is what the firm's own worst remembered month would cost. (ADR 0185.)
- **Losses are booked as readily as profits,** so a loss-making firm cannot distribute.
- **A firm with no trading record distributes nothing** — "no evidence is no ground for taking
  money out of it."
- **Ownership must total exactly 100 %** before any payout, and the distribution is exact to
  the minor unit.
- **The purpose is stated and it is correct:** "Without this, profit piles up inside firms for
  ever. Owners never accumulate the means to found anything... household savings can never
  become capital." This is the mechanism that makes L-1 possible.

The defects above (R-2, R-3) do not undermine this design — they corrupt its **input**. Fix
the `earned` line and the rest of the machinery is already correct.

---

# S. Firm management

## S-1. The production target inherits the whole market's shortage — Critical (E-1 chain, third exit)

**Where:** `management.rs` `execute_observed_firm_management` sets the target from
`proposal.advisory_batches`, produced by `plan_observed_production_adjustments`, whose
expansion branch uses `demand_ceiling = (average_sold + average_unmet).div_ceil(output_per_batch)`.

`average_unmet` is the market-wide figure E-1 stamped on every seller. So the E-1 chain now has
**three** confirmed exits, all of them physical:

1. `production.rs` — the monthly plan (J-5)
2. `capacity_investment.rs` — the size of new plant (O-1)
3. `management.rs` — the standing production target (here)

And exactly one place that handles it correctly: `pricing.rs` (N-1).

## S-2. A firm that cannot get inputs is forbidden to cut production — Major (unowned bound)

**Where:** `management.rs` `bound_target_for_procurement_shortfall`:

```rust
if !shortfall || previous == 0 { return advisory; }
let floor = previous.saturating_sub(1).max(1);
advisory.clamp(floor, previous)
```

**What it does:** when the firm recorded a procurement shortfall this month, it may not raise
its target at all and may not lower it by more than one batch.

**Why it is wrong by `AGENTS.md`:** ask who pays for the floor. Nobody. No supplier holds the
firm to a contract, no manager is penalised for cutting, no order book obliges it. The comment
states the *purpose* honestly — "keeps next month's procurement order alive instead of
trapping the firm at zero after a temporary logistics or supplier disruption" — but a purpose
is not a payer. This is a textbook unowned constraint: a real firm starved of inputs cuts to
zero and the supplier loses the order, which is precisely the feedback the floor suppresses.

**Consequence:** firms keep placing orders they cannot fill, which keeps `unmet` alive, which
feeds S-1, J-5 and O-1. A magic floor here manufactures the demand signal that drives the
overbuild.

## S-3. The shortfall test rescans the entire event log every month — Major (performance)

**Where:** the same function:

```rust
let shortfall = self.events.events().iter().rev().any(|envelope| {
    envelope.date() == self.date && matches!(envelope.event(), ...)
});
```

**Why it is wrong:** this walks the whole chronicle — every event ever recorded — once per
managed firm, once per month, and only to answer "did this firm have a shortfall today?". Over
a 50-year run the chronicle grows without bound, so the cost of this single test grows
quadratically in simulated time. It also uses the event log as working state, which
`architecture.md` treats as a record, not a queryable store. The same fact is already held in
`monthly_firm_effective_input_shortfalls`, which `reset_monthly_firm_accounts` deliberately
does **not** clear for exactly this kind of reader.

## S-4. Default target is full capacity, which J-1 makes unreachable — Major

**Where:** `previous = firm_production_targets.get(&firm).unwrap_or(firms[&firm].capacity_batches())`.

**Why it is wrong:** a firm with no explicit target is assumed to aim at its full installed
capacity. After O-2 raises `capacity_batches` while J-1 freezes `workers`, that default is a
target the firm can never physically reach, and the gap is permanently attributed to the
`"labor"` limiting factor rather than to the missing hiring mechanism.

## S-5. Authority is again resolved by identifier order — Minor

**Where:** `operations_actor` prefers `OperationsManager`, then `ChiefExecutive`, via
`firm_appointments.keys().find(...)` — `BTreeMap` order — and finally falls back to
`self.actors().keys().find(|actor| can_perform_corporate_action(...))`. Fourth instance of the
same pattern: O-3, Q-5, R-7, S-5. Worth one shared fix rather than four.

## S-6. Sound parts worth preserving

- **Insolvent firms are excluded** from management decisions.
- **A firm with no observed history is not managed at all** — decisions require evidence.
- **Every target change crosses `WorldCommand::SetFirmProductionTarget`,** the same boundary a
  player would use, so the whole month remains replayable. This discipline is consistent across
  `management.rs`, `capacity_investment.rs` and `credit_market.rs` and is one of the engine's
  real strengths.
- **A three-month forecast horizon** is applied consistently with `MANAGEMENT_FORECAST_HORIZON_MONTHS`.

---

# U. Firm distress and the death spiral

## U-0. Corrected composite chain — unpaid wages inflate earnings, but not immediate dividends

The previous version of this subsection overstated the direct death spiral. The missing check
has now been read in `credit_market.rs:62-87` and `board.rs:423-442`.

1. A missed payroll still records wage arrears without interest, penalty, or worker withdrawal
   (B-5).
2. `observation.rs` still computes `earned = sales_revenue − wages_paid − input_spend`, so the
   missed amount incorrectly raises `firm_retained_earnings` (R-3).
3. **But the firm cannot immediately distribute that phantom profit.**
   `FirmObligationTerms::obligation()` includes current payroll, wage arrears, inputs and planned
   investment; `free_cash = cash − obligation`. `board.rs` limits dividends to
   `min(free_cash − prudence_reserve, retained_earnings)`. While arrears remain, they therefore
   block the cash side of the distribution test.
4. The accounting defect is deferred rather than neutralized. Paying arrears removes cash and the
   claim together, but it does not repair the previously inflated retained-earnings ledger. Later
   operating cash can consequently be classified as distributable against earnings that were
   never earned.
5. The firm's actual distress spiral remains U-1 through U-3: compulsory owner rescue resets the
   clock; failure to rescue causes repeated layoffs; arrears survive the layoffs; insolvency is
   reachable only after the workforce reaches zero. Dividends are not the immediate trigger.

**Correction:** the earlier claim that a firm pays dividends out of cash currently owed as wages
was wrong. The stronger verified defect is temporal: unpaid wages permanently overstate retained
earnings and can authorize a later distribution after the wage claim has been cleared.

## U-1. Insolvency is unreachable while the firm has a single worker — Major

**Where:** `distress.rs` `respond_to_firm_distress`:

```rust
if months >= INSOLVENCY_TRIGGER_MONTHS && self.active_firm_workers(firm) == 0 {
    return self.declare_distressed_firm_insolvent(firm, arrears);
}
self.downsize_distressed_firm(firm, arrears)
```

**Why it is wrong:** insolvency is gated on the workforce already being **zero**, so a firm can
never fail while employing anyone, no matter how deep its arrears or how long its default.
Death is only ever reached by attrition through the 25 % monthly cut. A real firm with
unpayable debts fails while it still has staff; here the staff must evaporate first.

**Consequence:** the observed pattern of firms lingering for many months in a visibly dead
state is structural, not incidental. `months` is capped at 6 by `.min(INSOLVENCY_TRIGGER_MONTHS)`,
so the counter sits at its maximum while the workforce is ground down one quarter at a time.

## U-2. Owner recapitalization is compulsory and unlimited — Critical

**Where:**

```rust
let available = self.actor_cash.get(&owner).copied().unwrap_or_default();
let amount = Money::from_minor_units(available.minor_units().min(arrears.minor_units()));
if amount.minor_units() > 0 { return self.recapitalize_distressed_firm(...); }
```

**What it does:** the largest economic owner's cash is taken, up to the full arrears, every
time the firm is in distress. Then `firm_distress_months.remove(&firm)` **resets the clock**.

**Why it is wrong:** there is no decision anywhere in this path. The owner is not asked, has
no threshold, applies no test of whether the firm is worth saving, and cannot refuse. Compare
M-1, where a lender weighs a rate, a cushion and a distress charge before parting with money;
here the owner simply hands it over. It is a rule, not a choice.

**Who pays:** the owner — and this is the damaging part. Per L-1, `actor_cash` is the **only**
source of money for founding new firms. So the distress mechanism systematically drains the
exact pool that pays for firm entry.

**Consequence:** dying firms consume the capital that would have started new ones. A region
losing its firms cannot grow replacements — which is precisely the failure mode
`board.rs` says the distribution machinery exists to prevent. Two mechanisms, in the same
engine, pulling on the same pool in opposite directions.

**Second consequence:** a firm whose owner stays liquid can be recapitalized indefinitely,
resetting the distress counter each time, and will never reach insolvency however unviable it
is. Rescue is unowned in the other direction too.

## U-3. Downsizing does not reduce the debt that caused it — Major

**Where:** `downsize_distressed_firm`:
`reduction = (workers × DISTRESS_DOWNSIZING_BPS).div_ceil(10_000).max(1)`, applied to every
active agreement.

**Why it is wrong:** the trigger is wage **arrears**, and firing workers reduces future payroll
but leaves the arrears untouched. The trigger condition therefore remains true next month by
construction, so the cut repeats — 25 % per month, minimum one worker — until the workforce is
gone. There is no exit except recapitalization (U-2) or extinction.

**The `.max(1)` is doubly harsh in this engine:** L-5 gives every firm founded through
`firm_entry.rs` exactly one worker and J-1 forbids it from ever having more, so for every
entered firm the "bounded 25 % reduction" is in fact **total dismissal on the first cut**.

**Sound part worth preserving:** the workers' claim survives the layoff — the arrears remain
owed to the cohort after dismissal. That is correct and rare.

## U-4. Distress has only two triggers, and the expensive obligations are not among them — Major

**Where:** the entry condition is `arrears > 0 || matured_default`.

**What cannot make a firm distressed:** an empty till, an unpaid sales-tax bill, an inability
to buy inputs, sustained trading losses, or capital spent on capacity it cannot staff (O-2).

**Consequence:** the two obligations that R-2 shows are missing from the earnings line — tax
and debt service — are also missing from the distress test. A firm can be destroyed by its tax
bill (`circuit_probe.rs`) without ever being recorded as distressed, and therefore without ever
triggering recapitalization, downsizing, or a credit application.

## U-5. The credit-evidence grace month is an unowned bound — Minor

**Where:**

```rust
let observations = self.firm_operating_history.get(&firm).map_or(0, Vec::len);
let awaiting_credit_evidence = (2..4).contains(&observations);
if months < RECAPITALIZATION_TRIGGER_MONTHS
    || (months <= RECAPITALIZATION_TRIGGER_MONTHS + 1 && awaiting_credit_evidence) { return Ok(None); }
```

A firm holding exactly two or three months of history gets one extra month before any distress
response, so that the credit market has time to see it. The intent is reasonable; the
implementation is a bare window with no party bearing its cost, and it makes a firm's fate
depend on the **length of its history vector** rather than on anything economic.

## U-6. Sound parts worth preserving

- **Recapitalization moves real money** from a named owner's `actor_cash` into the firm's till,
  with no creation — contrast A-1 and P-3.
- **Worker claims survive layoffs and liquidation** (`worker_claim_survives_layoff`), and the
  liquidation tests confirm workers rank before ranked creditors and owners.
- **Distress months reset when arrears clear** and the counter is capped, so the state cannot
  run away.
- **Insolvency requires a named administrator** via `operations_actor`; without one, nothing
  happens rather than something arbitrary.
- **Matured debt default is a trigger in its own right,** so a firm that pays wages but defaults
  on a loan is still caught.

## U-7. Five full world clones per month — Major (performance)

`let mut next = self.clone()` now appears in `production.rs` (J-6), `firm_entry.rs` (L-6),
`capacity_investment.rs`, `management.rs` and `distress.rs`, plus the government and reserve
paths. The whole world — every cohort, firm, agreement, price, claim and the entire event
chronicle — is deep-copied at least five times per simulated month. Combined with S-3's full
chronicle rescan, this is the dominant cost of a 50-year run and it grows with history rather
than with activity.

---

# V. Planned expansion is treated as a present obligation

## V-1. A market-wide shortage locks each firm's cash and enlarges its loan request — Critical (fourth E-1 exit)

**Where:** `capacity_investment.rs:176-220` and `credit_market.rs:408-486`.

**What it does:** `observed_capacity_investment_need` converts
`proposal.average_unmet_market_demand` into `wanted` capacity batches and values the entire works.
`observed_firm_obligation_terms` then adds that amount to payroll, wage arrears and inputs as if it
were already a dated liability. The same sum is used twice:

- `free_cash = cash − (payroll + arrears + inputs + investment)` blocks owner distributions;
- `funding_gap = obligation − cash` sizes the autonomous credit request, provided observed recurring
  operations show a surplus.

**Why it is wrong:** the expansion is not a contract, approved project, purchase order, or existing
claim. It is a fresh advisory calculation. More importantly, its `wanted` term inherits E-1: every
seller reads the whole market's shortage as its own. One duplicated signal therefore reserves the
full build cost inside every competing firm and asks lenders to finance every copy.

**Who pays:** owners lose access to otherwise distributable cash; lenders surrender actor cash;
and the same actor-cash pool is the only source that can found replacement firms (L-1). Nobody has
yet approved or contracted the works whose cost receives seniority over those uses.

**Consequence:** E-1 now has a fourth confirmed exit:

> duplicated market shortage → duplicated expansion need → cash locked inside every seller +
> duplicated credit demand → fewer owner distributions → less founder cash → weaker firm entry.

This is the exact failure mode the dividend machinery was designed to prevent: profit again piles
up inside incumbent firms, while new firms cannot accumulate founders' capital. The credit market
then finances sterile capacity which J-1 says cannot be staffed.

## V-2. The recurring-surplus gate omits tax and debt service — Major

**Where:** `FirmObligationTerms::observed_surplus` is
`monthly_sales − payroll − inputs`.

**Why it is wrong:** the lender calls a firm operationally viable before sales tax and existing
debt service. This repeats R-2 at the credit gate. A business whose entire post-tax margin is
negative can still borrow for expansion because the underwriting calculation sees a positive
surplus. The new loan then adds debt service to a firm the test had already misclassified.

## V-3. Sound part worth preserving

The actual firm loan remains a transfer between named balance sheets, and scheduled service moves
principal and interest back to the creditor. The defect is the underwriting signal and the purpose
of the requested funds, not the cash settlement.

---

# W. Intermediate procurement

## W-1. Exhausted sellers inherit demand that was not economically effective — Critical

**Where:** `procurement.rs:469-512` classifies the reason for each buyer shortfall, but
`record_procurement_outcomes` at `:675-695` writes the full `unmet_by_market` into every
seller whose offer was exhausted.

**What it does:** `effective_unmet_by_market` correctly excludes three cases: the buyer
refused the price, had no cash, or was blocked by route capacity. That clean signal is stored
for regional input-shortfall evidence. Seller operating evidence ignores it and receives the
unclean total instead.

**Consequence:** a supplier sees profitable scarcity when buyers explicitly said the input
was not worth its price, when buyers were bankrupt, and when stock existed but transport was
full. That false signal flows through the same J-5/O-1/S-1 expansion chain as E-1. Pricing is
protected; production, management, capacity investment and credit demand are not.

## W-2. Every positive B2B sale is recorded as a 100% sell-through — Major

**Where:** `procurement.rs:662-696` iterates only sellers with recorded sales and constructs
`MarketOfferOutcome { offered: quantity, sold: quantity, unsold: 0 }`.

**Why it is wrong:** `quantity` is the quantity sold, not the pristine offer. A seller that
offered 1,000 and sold 1 is recorded as having offered 1 and sold all of it. Sellers that sold
nothing receive no outcome at all. The evidence layer therefore cannot distinguish a thin sale
from a stockout and repeats the missing-denominator defect in E-3/J-6.

## W-3. Local inputs outrank cheaper delivered imports absolutely — Major

**Where:** `settle_procurement_orders`: local candidates are appended first in offer-price
order; all imports are appended afterwards in delivered-price order.

**Why it is wrong:** a local input is bought before an import at any delivered price. This is
not a modeled tariff, preference or resilience policy, so nobody pays for the protection. It
repeats D-5 on the industrial side and can force a firm to reject a viable recipe before it is
allowed to inspect a cheaper import.

## W-4. The input ceiling double-spends the batch margin — Critical

**Where:** `input_value_ceiling`, `procurement.rs:133-202`.

**What it does:** for each input independently, the maximum bid is gross output revenue minus
all *other* materials. It deliberately deducts neither wages nor sales tax.

**Why it is wrong:** independent ceilings are not a budget identity. With two inputs, each can
be authorized to consume the residual needed by the other; the wage ceiling in B-4 is then
authorized against the same gross margin a third time. Sales tax is absent from every ceiling.
A batch can therefore pass every individual affordability test while the sum of approved
inputs, wages and tax exceeds revenue.

## W-5. Procurement accounting does include freight — sound, closes T-5 question 2

`settlement.purchases` accumulates `fill.spend`, and `spend = goods_spend + freight_spend`.
`monthly_firm_procurement_purchases` therefore contains the delivered cost. R-2 is not
additionally overstated by missing B2B freight.

## W-6. Household grievance ignores why the household could not buy — Major

**Where:** `collect_household_grievance_evidence`, `procurement.rs:726-762`.

Any unmet survival order plus any foreign offer over a registered direct route accrues a
bilateral grievance. Unlike the firm path, it receives no `capacity_limited`, `refused_on_price`
or `starved_of_cash` evidence. A household too poor to buy food, unwilling to pay the price, or
blocked by domestic route congestion can therefore drive its country to hostility against the
foreign supplier. At 500 bps per month and a 7,500 threshold, fifteen such months create war
from a bookkeeping ambiguity.

---

# X. Migration

## X-1. Merged migrants leak skill rows; stress rows are transient — Major (corrected)

**Where:** `migration.rs:136-169` calls `copy_persistent_cohort_state(source, migrant_id)`
before deciding whether the migrant joins an existing host cohort.

When a host exists, the temporary cohort is never inserted. Health and experience rows are
removed explicitly. Skill entries keyed by `(migrant_id, skill)` are not removed and no code
transfers those proficiencies to the host. They therefore remain as durable state for an entity
that does not exist.

The previous version of this entry overstated the persistence of every copied map.
`derive_monthly_social_stress`, `accumulate_monthly_social_stress`, and market settlement replace
their complete maps from current cohorts, so orphan `social_stress`, `social_stress_memory`, and
`deprivation_pressure` rows survive only until those stages next run. The skill leak is permanent;
the social leak is transient.

**Gameplay consequence:** the migrant's skills are lost rather than blended into the destination
cohort, while unreachable skill rows continue to enlarge snapshots and fingerprints.

## X-2. The migration reserve is controlled by the frozen income field — Major (B-2 chain)

**Where:** `household_monthly_income_share`, `migration.rs:508-515`.

The household must keep one month of `annual_income` per migrated head in addition to the
relocation fee. Since employment never updates that field and dismissal sets it permanently to
zero, the same real wage can produce opposite migration legality depending on fixture history.
A formerly unemployed household needs only one minor unit of reserve; a fixture with stale high
income can be trapped despite holding the same cash.

## X-3. New households are assumed fully fed until measured — Major

**Where:** `regional_survival_fulfillment`, `migration.rs:304-310`, defaults missing health to
10,000 bps.

A destination containing unmeasured cohorts is treated as a perfect refuge. This default can
trigger an annual move before the destination has survived one market cycle. The mover pays a
real fee and gives up employment against evidence that was never observed.

## X-4. Sound parts worth preserving

Migration conserves people, wealth and debt through a one-household split; the relocation fee
moves from the household to the destination treasury; housing capacity and cash are real
constraints; famine migration re-evaluates destination feeding after every arrival rather than
using a fixed exodus cap.

---

# Y. Insolvency and liquidation

## Y-1. Credit underwriting measures pre-tax surplus — Major

**Where:** `insolvency.rs:426-470` computes
`surplus = monthly_sales − input_cost − payroll`, then lets debt service consume half.

Sales tax is absent. Existing service is subtracted only after this overstated surplus is
formed. A firm with a negative post-tax operating margin can therefore receive a loan, and the
loan is sized from cash flow the state will later collect. This independently confirms V-2 in
the explicit underwriting path.

## Y-2. Collateral is valued at gross reference-price output, not liquidation value — Major

**Where:** `insolvency.rs:471-495` values installed capacity as one month of gross output at the
regional reference price and lends up to 70% of total collateral.

The valuation ignores tax, inputs, wages, the frozen worker ceiling J-1, and the absence of a
buyer. In liquidation the same plant may be unsaleable; nevertheless it supported cash credit
before failure. The lender's loss is therefore created by the appraisal formula, not by an
actor's mistaken forecast.

## Y-3. Liquidation is a fixed-price, identifier-ordered allocation, not an auction — Major

**Where:** `sell_liquidation_inventory` and `sell_liquidation_capacity`,
`insolvency.rs:1311-1574`.

Buyers are traversed in `BTreeMap` firm-ID order. Inventory sells at the regional reference
price and capacity at one month of reference-price gross output. There are no bids. The lowest
ID gets first access until its cash or target is exhausted; later firms see the remainder.
Thus both price and winner are administrative artifacts, even though the comments call the
process an auction.

The capacity path adds an explicit `integration_limit = current capacity`, so a buyer may at
most double. No actor bears this limit or chooses it; it is exactly the unowned concentration
cap forbidden by `AGENTS.md`.

## Y-4. Administration freezes the estate for exactly twelve months — Major (unowned bound)

**Where:** `ADMINISTRATION_LIMIT_MONTHS = 12` and
`execute_observed_firm_liquidations`, `insolvency.rs:1275-1307`.

Until the twelfth complete month, an unreorganized estate cannot liquidate; after it, liquidation
is automatic. No creditor vote, administrator policy, storage cost, spoilage, court capacity or
financing holds the line. Inventory and plant sit physically unchanged while creditors wait.
The duration is therefore a timer, not an institution.

## Y-5. Worker and creditor waterfalls conserve money — sound, preserve

Liquidation pays workers first, then secured and unsecured creditors pro rata with a
largest-remainder allocation, then owners. Cash debits equal named recipient credits, and unpaid
claims are explicitly written off and recorded as lender losses. This is one of the strongest
ledger paths in the engine.

---

# Z. Logistics and freight

## Z-1. Formal freight service destroys operating-cost money — Critical

**Where:** `world_logistics.rs:184-264` uses `evaluate_freight_economics`.

The shipper is debited the full freight `revenue`, but the carrier is credited only `margin =
revenue − operating_cost`. No fuel seller, worker, maintainer, inventory, or treasury receives
`operating_cost`. The difference disappears from the money stock. If margin is negative, the
carrier also pays the shortfall to nobody.

This is a direct conservation failure. The operating-cost profile names three costs but none
has a counterparty.

## Z-2. Immediate market imports use the opposite accounting — Critical

Household and procurement imports do not create `InventoryShipment`s. They transfer the good
instantly, ignore `transit_days`, terminals and reliability, and credit the carrier the **gross**
tariff without deducting `RouteOperatingCost`.

The same route therefore has two incompatible realities:

- explicit shipment: takes days and burns operating-cost cash;
- market import: arrives immediately and pays the carrier gross revenue with no cost.

Actors can avoid time and operating costs merely by using market clearing rather than the
shipment API. This makes all logistics strategy downstream of the chosen code path, not the
world's physical network.

## Z-3. Route reliability is dead data — Major

**Where:** `LogisticsRoute.reliability_bps` is constructed, exposed and hashed, but never read
by routing, shipment progress, market clearing or capacity allocation.

A route with reliability 0 delivers exactly as quickly and surely as one at 10,000. Reliability
changes fingerprints while changing no gameplay.

## Z-4. Freight contracts never expire automatically and ignore their start date — Major

`expire_freight_contracts` has no production caller. Activation does not check `start_month`,
and all logistics readers test only `status == Active`. Once activated, a contract reserves
capacity and grants its discount immediately and indefinitely unless external code manually
calls expiry with a month value.

## Z-5. Contracts can promise more than the route can carry — Major

Registration checks each contract's reservation against full route capacity but never checks
the sum of active/proposed contracts. Contract ledgers then enforce each reservation separately.
The aggregate physical ledger eventually rejects a shipment, so money is not created, but several
shippers can hold simultaneously valid promises to the same capacity with no breach, penalty or
priority rule.

## Z-6. Monthly capacity is modeled as whole-trip occupancy — Major (unit mismatch)

A `monthly_capacity` quantity is reserved on every leg at shipment start and released only when
that leg completes. A 90-day leg therefore consumes three months of the nominal monthly pool,
while a downstream leg is blocked before the cargo reaches it. In contrast, immediate market
imports reset a copied pool every month. One field is used as monthly throughput in one path and
as concurrent in-transit stock in another.

## Z-7. Formal logistics is a manually driven subsystem — Critical

No monthly or annual system calls `advance_inventory_shipment` or `admit_terminal_shipments`.
Those functions are reachable only through explicit commands and tests. Transit time therefore
does not pass when world time passes. See AC-1 through AC-4 for terminal consequences.


---

# AA. Investment, housing, and public signals

## AA-1. Route expansion destroys money and creates infrastructure — Critical

**Where:** `route_investment.rs:75-145` `try_fund_route_expansion`.

The carrier's cash is debited by `cost`, then `LogisticsRoute::add_capacity` creates physical
capacity. No builder, worker, material supplier, treasury or capital account receives the money.
Unlike firm-capacity investment and housing, this path has no `distribute_capital_outlay` step.

**Who pays:** the carrier pays; nobody receives. The money supply contracts by the full project
cost while physical infrastructure appears.

## AA-2. Route investment listens only to the shortcut logistics path — Major

**Where:** `current_route_freight_revenue` scans current-date events and counts only
`MarketFreightPaid` and `FirmProcurementFreightPaid`.

The formal `InventoryShipment` path records `ShipmentStarted`, not either route-specific payment
event. A route used by real timed shipments can therefore be constrained and profitable without
producing the revenue evidence required for expansion. Conversely, instantaneous market imports
from Z-2 are the only traffic that teaches the route to grow.

## AA-3. Route expansion is governed by three unowned constants — Major

Expansion requires exactly three consecutive constrained revenue months, adds exactly one tenth
of existing capacity, and prices the works at exactly twelve months of gross tariff revenue.
Operating cost, expected utilization, finance, construction supply and carrier policy do not enter.
These are behavioral decisions presented as universal engine law.

## AA-4. Generic investment is conserved only in the autonomous wrapper — Major

`execute_investment_commitment` moves firm cash into `committed_investments`; launching a project
moves the amount out of that account and into a project budget. `InvestmentProject::advance`
merely changes `spent`, and completion creates capacity. The generic API itself names no recipient
for the budget.

The autonomous capacity-investment wrapper repairs this by immediately calling
`distribute_capital_outlay`, paying local cohorts. A caller using the public generic commands
`CommitFirmInvestment` + `LaunchInvestmentProject` + `AdvanceInvestmentProject` does not. Player
and autonomous actors therefore do **not** share one accounting rule: the same valid project can
be a real transfer or a money sink depending on which API path initiated it.

Committed/project money is also absent from `MoneyStockMeasured`, whose categories are only
households, firms, owners and treasuries. Even before completion, the annual measurement reports a
false contraction whenever investment funds are in transit through these ledgers.

## AA-5. Housing conserves money but manufactures the real work — Major

**Where:** `housing.rs:149-247`.

Treasury outlay is distributed to all local cohorts by population, so cash is conserved. But no
construction firm, worker availability, skill, material, transport or production capacity is
required. Every resident receives building income and dwellings appear after two annual ticks.
The monetary circuit is closed while the material circuit is absent.

The starting stock (`population / 2`), trigger (90% occupancy), project size (10% of stock), price
(one year of base rent) and duration (two years) are all global constants. No government policy,
builder bid or resource bottleneck owns them.

## AA-6. Social health and physical health disagree on an empty ledger — Major

**Where:** `social.rs:61-79` versus `health.rs`.

When a cohort has neither recorded survival consumption nor recorded unmet survival demand,
`derive_monthly_social_stress` sets health risk to zero. The health system deliberately treats the
same absence of purchases as zero fulfillment and applies hunger damage. A cohort can therefore
lose functional capacity and die while the political/social sensor records no health pressure.

## AA-7. Every employed cohort accumulates unemployment history — Major (B-1 exit)

`update_monthly_cohort_experience` increments `unemployment_months` solely from
`EmploymentStatus::Unemployed`. Because employment agreements never set `Employed`, working
cohorts accumulate an uninterrupted unemployment history. This is another downstream reader of
the stuck B-1 sensor and can contaminate any future policy using experience duration.

## AA-8. Regional fiscal incidence omits most regional public flows — Major

**Where:** `regional_interests.rs:283-347`.

A region's fiscal position is only `RegionalPublicServicesUpdated.service_budget −
FirmSalesTaxPaid.paid`. It omits relief, reserve procurement, reserve releases, housing outlay,
government programs, public debt payments and other transfers. That partial figure changes
regional satisfaction, country confidence and ultimately legitimacy.

A region can receive a large housing project and emergency aid yet be labelled a net contributor;
a region selling heavily into public reserves can be labelled ignored. Politics responds to an
accounting subset, not fiscal incidence.

## AA-9. Service allocation preserves player freedom — sound, preserve

`CountryServiceAllocation` permits zero shares and a 100% share for one region, and the explicit
command checks authority and nationality rather than imposing fairness. This correctly follows the
simulation-freedom rule. The remaining defect is upstream: the service budget itself is the second
use of already distributed spending (F-2), and autonomous need weights read broken pressure
sensors.

---

# AB. Public reserves and government programs

## AB-1. Reserve carrying costs destroy treasury money — Critical

**Where:** `public_reserve.rs:641-719`.

Monthly maintenance allocates an assessed carrying cost, debits each treasury, and then only uses
the paid fraction to reduce `neglect_spoilage`. No storage firm, warehouse worker, utility,
landowner or other account receives the payment. The baseline and neglect spoilage are legitimate
physical destruction; the carrying-cost payment is unexplained monetary destruction.

This is structurally the same fault as route investment AA-1: an expense is modeled as a debit
without a counterparty.

## AB-2. Manual reserve procurement accepts any positive transfer price — Major

`procure_government_reserve` validates stock, authority and treasury cash but not the price against
the regional price, seller offer, cost or an approved procurement contract. An authorized actor can
pay almost the whole treasury for a tiny inventory quantity. The autonomous wrapper happens to
pass the current regional reference price, but the shared command boundary does not enforce that
rule or record a corruption/favoritism consequence.

This is not merely freedom to overpay: there is no competing bid, negotiation, audit consequence,
or political ownership of the deviation.

## AB-3. Scarce reserve release is cohort-ID rationing — Major

`distribute_government_reserves` walks the ordered `unmet_demand` map and satisfies each survival
claim until regional stock is exhausted. Equal survival claims are therefore prioritized by
`CohortId`, not hunger severity, children, health damage, population, queue time, political policy,
or an explicit first-come event. The stable identifier silently decides who eats.

The positive part should be preserved: released goods are physically removed from stock, added to
consumption, removed from unmet demand, and deprivation pressure is recomputed.

## AB-4. Reserve policy repeatedly scans the complete chronicle — Moderate

Both `current_reserve_policy_evidence` and `current_reserve_priority_evidence` iterate every event
ever recorded and then discard all events whose date is not the current date. Monthly cost grows
with world age even though the required evidence is bounded to one month. Regional interest and
route-revenue readers repeat this pattern, making the event log a growing operational database.

## AB-5. Program appropriations and investment commitments are omitted from money stock — Major

`observed_money_stock` counts only cohort wealth, firm cash, actor cash and treasuries. It excludes:

- `GovernmentProgram::carryover_funding`;
- `committed_investments`;
- budgets held by active `InvestmentProject`s.

Treasury appropriation therefore appears to destroy money until delivery; a firm commitment appears
to destroy money until construction income is distributed. The annual event explicitly claims to
count every minor unit, so this is a false macroeconomic contraction, not merely presentation.

## AB-6. Programs never enter Completed or Failed states — Major

`GovernmentProgramStatus` exposes `Completed` and `Failed`, and command validation treats them as
terminal. No production code assigns either value. `record_execution` only updates carryover and
delay; it never checks the end year or fulfillment. After the appropriation window closes, the
program remains `Active` forever but can no longer receive an appropriation and therefore cannot
execute again.

The only reachable terminal transition is manual cancellation.

## AB-7. Cancelling a program permanently strands its carryover — Critical

Cancellation explicitly retains `carryover_funding`, while cancelled programs cannot receive
appropriations or execute. There is no refund to treasury, creditor repayment, reassignment, or
write-off event. The retained balance becomes unreachable money and is also absent from
`MoneyStockMeasured`.

A delayed program can therefore lock an arbitrarily large fiscal balance forever.

## AB-8. Temporary program labor double-uses employed people — Critical (B-1 exit)

Program labor availability sums every person whose cohort reports `Unemployed`. Because labor
agreements never change that status, ordinary employed workers remain fully available to every
public program. `government_program_labor_usage` prevents reuse across programs in the same year,
but it never subtracts private employment.

The program can consume the same person's private working capacity and public temporary capacity
simultaneously.

## AB-9. All program delivery is paid to the first eligible cohort — Major

For each region, the complete delivered monetary amount is credited to
`recipients.first()`. Worker count and cohort population do not divide the payment. With B-1, the
recipient is simply the first regional cohort by stable ID, potentially one whose members already
work elsewhere.

If `temporary_workers_required == 0`, no `temporary_labor` row is created at all: delivered funding
is deducted from carryover and public services improve, but nobody receives the money. Such a
program is another direct monetary sink.

## AB-10. Zero-capacity administrations still deliver one quarter of a program — Major

Administrative absorption has a hard 25% floor and infrastructure absorption a hard 50% floor.
Consequently a region with zero administration and infrastructure still turns 25% of committed
funding into service improvement, provided optional labor and materials do not bind. No actor,
minimum viable institution or emergency contractor owns this floor.

Political influence can add or subtract up to 20 percentage points, also as a global clamp rather
than an accountable implementation decision.

---

# AC. Terminal integration, distress, and remaining control loops

## AC-1. Formal shipments do not advance with simulation time — Critical

`advance_inventory_shipment` has no production caller outside `WorldCommand`; likewise terminal
admission is only a command. The monthly and annual simulation cycles never advance days on active
shipments and never admit queued cargo automatically. A shipment started through the formal API
therefore remains in transit forever unless an external client manually drives each route and
terminal transition.

This is the structural reason Z-2's instantaneous market path became the real economy: the
physically detailed path is not connected to the clock.

## AC-2. Terminal operators provide free service and storage cost is dead data — Critical

A terminal has an operator firm and `storage_cost_per_unit_day`, but the storage cost is read only
by the stable fingerprint. Neither shipment start, queueing, admission, handling nor delivery
charges the shipper or credits the operator. Handling consumes capacity and time but no money,
workers, energy or maintenance.

The operator owns no revenue stream and bears no cost. Terminal ownership is gameplay decoration.

## AC-3. An oversized shipment can block a terminal forever — Major

Queue admission is strict FIFO and stops when the head does not fit. Enqueueing and shipment start
do not verify `shipment.quantity <= terminal.capacity`. An oversized head can therefore never be
admitted, while every smaller shipment behind it is permanently blocked. No rejection, splitting,
rerouting, demurrage or cancellation resolves the queue.

## AC-4. Terminal “daily throughput” is reserved as concurrent stock — Major

`daily_handling_capacity` is reserved for the full multi-day handling phase and released only when
transfer completes. It is not renewed each simulated day. The same quantity is therefore named a
daily flow but enforced as work-in-process capacity, repeating Z-6's dimensional mismatch.

## AC-5. Distress layoffs leave the firm's productive workers untouched — Critical

`downsize_distressed_firm` reduces workers in `EmploymentAgreement`s. Production is constrained by
the separate immutable `Firm.workers` field. A firm can lay off every contractual worker while
continuing to produce with the old embedded workforce. When active agreements reach zero, the
distress system may declare the firm insolvent even though the production system still sees its
full crew.

Liquidation can finally transfer or write off `capacity_batches`, so plant is not an absolute
ratchet; ordinary distress cannot mothball or sell it, and ordinary layoffs do not reduce its labor.

## AC-6. Distress always removes one quarter of every agreement — Major

After the trigger, automatic management cuts 25% of every active agreement, rounded up with a
minimum of one. The manager does not choose a target, protect skills, compare marginal products or
weigh severance and morale. No party owns the 25% coefficient; the engine chooses it universally.

## AC-7. Emergency relief is rationed twice by stable identifier — Major

Affordability-gap capture allocates the copied unsold-lot book across survival intents in iteration
order. Execution then walks `monthly_affordability_gaps` by `CohortId` and lets early cohorts drain
the remaining treasury. With scarce goods or cash, identity determines both who is recognized as
affordably rescuable and who receives funding.

The transfer itself remains conserved (P-2); selection is the defect.

## AC-8. Public-service dynamics are fixed score motion without production — Major

Every region begins at 50% healthcare, infrastructure and administration. Recurring funding moves
administration by exactly 250 basis points and the other services by 500, while administration
creates a hard delivery ceiling exactly 1,000 points above itself. These numbers are not derived
from staff, facilities, procurement, maintenance, population or an institution's policy.

The budget is a target for a persistent score rather than demand for real service inputs. Together
with F-2, public spending is first distributed as income and then independently moves free service
capacity.

## AC-9. `annual_income` is never rebuilt from wages received — Critical (B-3 confirmed)

A full search finds no setter or monthly update that assigns actual wages to `annual_income`.
Production code only splits it during migration/training, combines it when cohorts merge, zeros it
when employment ends, and rescales it after deaths. Household cashflow reads the field; labor does
not write it.

The wage/income cycle is therefore definitively circular: contracts derive offers from stale annual
income while household earnings continue to come from that same stale seed, not payroll received.

---

# K. Coverage

**Read line by line:** `value.rs`, `accumulator.rs`, `rationing.rs`, `health.rs`,
`coping.rs`, `cohort.rs`, `demand.rs`, `simulation.rs` (whole), `commerce.rs` (whole),
`market.rs` (whole), `labor.rs` (wage formation, matching, shedding, payroll).

**Also read in this pass:** `production.rs` (whole), `firm_entry.rs` (founding, capital
outlay, foundation economics), `credit_market.rs` (whole), `insolvency.rs` (credit issue,
acceptance, monthly debt service).

**Not yet read:** `pricing.rs` (partly), `procurement.rs`, remainder of `insolvency.rs`,
`distress.rs`, `credit_market.rs`, `firm_entry.rs`, `capacity_investment.rs`,
`route_investment.rs`, `investment.rs`, `public_reserve.rs`, `government_program.rs`,
`service_allocation.rs`, `relief.rs`, `housing.rs`, `migration.rs`, `logistics.rs`,
`freight.rs`, `terminal.rs`, `world_logistics.rs`, `board.rs`, `management.rs`,
`accounting.rs`, `observation.rs`, `political_economy.rs`, `regional_interests.rs`,
`social.rs`, `business.rs`, `chronicle.rs`, `world.rs`.

**Open questions for the remaining pass:**

1. Does any code path create a `Child` cohort? If not, C-3 is terminal for long runs.
2. Does `credit_market.rs` move cash between named balance sheets, or does it mint like A-1?
3. Does `production.rs` consume anything priced, or is output purely physical?
4. Do `public_reserve.rs` and `relief.rs` buy goods with treasury money, or issue them?
5. Where does `firm_entry.rs` get a new firm's starting capital, and whose balance falls?

---

# T. Coverage, updated (supersedes section K above)

## T-1. Answers to the open questions

| # | Question | Answer | Where |
| --- | --- | --- | --- |
| 1 | Is a `Child` cohort ever created? | **Still open.** No creation path found so far; C-3 remains a live risk for long runs. | — |
| 2 | Does firm credit mint money? | **No — it transfers.** The creditor's `actor_cash` falls by the principal and the firm's till rises by the same amount; service moves it back. | M-1 |
| 3 | Does production consume anything priced? | **No — it is purely physical.** `execute_monthly_production` moves no money at all: no depreciation, no energy, no rent. | J-3, J-4 |
| 4 | Do reserves and relief buy goods, or issue them? | **They buy.** Reserve procurement debits the treasury and pays the selling firm; relief debits the treasury and credits the cohort. **But state borrowing mints.** | P-1, P-2 vs P-3, P-4 |
| 5 | Where does startup capital come from? | **The founder's own `actor_cash` falls** by `capital_cost + working_capital`, and the capital cost is paid out to local households. | L-1 |

## T-2. Files read line by line

`value.rs`, `accumulator.rs`, `rationing.rs`, `health.rs`, `coping.rs`, `cohort.rs`,
`demand.rs`, `simulation.rs`, `commerce.rs`, `market.rs`, `production.rs`, `credit_market.rs`,
`pricing.rs`, `capacity_investment.rs`, `political_economy.rs`, `accounting.rs`,
`management.rs`, `procurement.rs`, `migration.rs`, `freight.rs`, `logistics.rs`,
`world_logistics.rs`, `route_investment.rs`, `investment.rs`, `housing.rs`,
`service_allocation.rs`, `social.rs`, `regional_interests.rs`, `public_reserve.rs`,
`government_program.rs`, `business.rs`, and `insolvency.rs`
(including liquidation).

## T-3. Files read in the parts that matter

- `labor.rs` — wage formation, matching, shedding, payroll, vacancy caps
- `firm_entry.rs` — founding, capital outlay, foundation economics
- `insolvency.rs` — credit issue, acceptance, monthly debt service
- `relief.rs` — relief funding, debt issuance, debt headroom, authority
- `public_reserve.rs` — procurement and treasury arithmetic
- `government_program.rs` — appropriation, execution, wage payment
- `board.rs` — dividends and owner distributions
- `observation.rs` — the monthly earnings line

## T-4. Still unread

`distress.rs`,
`terminal.rs`, `world_logistics.rs`, `regional_interests.rs`, `social.rs`, `business.rs`,
`chronicle.rs`, `world.rs`, remainder of `public_reserve.rs` (policy review, distribution),
remainder of `government_program.rs` (political modifiers, memory).

## T-5. New open questions raised by this pass

1. **Does any code path ever set `EmploymentStatus::Employed`?** Confirmed no, outside tests
   (B-1). Then what was the field for, and which other readers silently depend on it? Q-2 found
   one; there may be more.
2. **Is freight included in `monthly_firm_procurement_purchases` spend?** **Yes.** The stored
   purchase is total delivered `spend`, including freight (W-5).
3. **Does `distress.rs` ever reduce `capacity_batches`?** If capacity only ever grows (O-2) and
   never shrinks, the physical stock of plant is a ratchet.
4. **Who buys public debt when the state restructures?** F-5 writes off 40 % of a principal
   that P-3 created from nothing, so the write-off destroys money that was never anyone's
   asset. Confirm against the money-stock measurement.
5. **Is there any path by which a cohort's `annual_income` is updated from wages actually
   received?** B-3 suggests the wage is derived from income which is derived from the wage.

## T-6. The four repeated structural patterns

Rather than 60 unrelated defects, this audit keeps finding the same four mistakes wearing
different clothes. Fixing the pattern is cheaper than fixing the instances.

1. **A quantity is created or destroyed without a counterparty.**
   A-1 household borrowing, A-4 public outlay, P-3 state borrowing, P-4 debt-funded programs,
   L-2 the plant that appears from nothing, J-2 free labour, P-5 free public-works labour.
2. **A market-wide figure is read as if it were one firm's own.**
   E-1 the source, then J-5, O-1 and S-1 as the three physical exits. Only N-1 handles it
   correctly.
3. **A sensor is stuck, so the feedback it controls never arrives.**
   B-1 employment status → Q-1/Q-2 political pressure → the fiscal pump; N-4 the reference
   price that never learns; R-2/R-3 earnings that omit tax and reward unpaid wages.
4. **An identity or a limit is decided by identifier order or by a bare constant.**
   O-3, Q-5, R-7, S-5 for authority; S-2, Q-4, Q-6 and the unnamed weights for limits.

## T-7. Cheapest high-value repairs, in order

These are recorded as observations, not as an instruction to act. No code has been changed.

1. **Wire up `firm_break_even_unit_price`** (N-2). The function is written, documented and
   dead; connecting it to the retail offer floor addresses the observed farm death directly.
2. **Set `EmploymentStatus::Employed` when an agreement is registered** (B-1). One assignment
   repairs the political sensor (Q-1, Q-2) and the public-works eligibility filter (P-5).
3. **Subtract sales tax and debt service from the `earned` line, and use `wages_owed`
   rather than `wages_paid`** (R-2, R-3). Three terms in one expression; the distribution
   machinery around it is already correct.
4. **Give `Firm.workers` a mutator and let hiring raise it** (J-1). Until then, every capacity
   investment in the model is a pure loss (O-2).
5. **Divide market-wide `unmet` by the number of sellers, or attribute it per seller at
   source** (E-1). One change closes J-5, O-1 and S-1 together.

---

# AD. Residual cohort and accounting defects

## AD-1. Annual population rescaling breaks the cohort's own household invariant - Critical

`HouseholdCohort::new` requires every non-empty cohort to have `1..=people` households and an
empty cohort to have zero households. Annual closure does not preserve that invariant. It plans a
new `people` count and calls `set_people`; `households`, liquid wealth, debt and annual income are
left untouched. `set_people` has no validation and this annual closure is its only production call.

A declining cohort can therefore reach zero people while retaining households, money and debt. A
rising cohort receives free people without new households or household balance sheets. The damage
is not cosmetic: per-household demand, health requirements, training and migration continue to read
the stale household count. A zero-person cohort can still create household demand, while severe
population loss can leave more households than people. `validate_population_accounting` checks only
regional people totals and cannot detect this state.

This is a second independent demographic overwrite after C-1. Population cannot be repaired by
rescaling one field; births, deaths, household formation, household dissolution, inheritance and
migration must be explicit stock-flow operations.

## AD-2. Retirement converts contractual wages into payerless fallback income - Critical

At annual closure, a cohort entering `Senior` has every active employment agreement set to zero, but
`annual_income` is preserved. In the next monthly cashflow, contracted workers are now zero, so the
whole stale annual-income field is paid through `uncontracted_monthly_income`. No employer, pension
fund or treasury is debited.

The same mechanism affects ordinary layoffs: reducing agreement workers increases the
"uncontracted" share of stale annual income. Losing a job can therefore replace named payroll with
A-2's money creation instead of reducing income. Retirement and unemployment need explicit income
sources - pension transfers, savings drawdown, benefits, informal work or zero income - rather than
a fallback percentage of a frozen seed.

## AD-3. The money-stock observation is not a ledger invariant - Major

`observed_money_stock` sums only four liquid locations: cohort wealth, firm cash, actor cash and
national treasuries. It does not represent claims, escrows or committed cash, and AB-5 already shows
that program appropriations and investment commitments can live outside those four balances. The
annual event is also emitted before sales-tax settlement and the rest of annual fiscal closure, so
it cannot prove that the closing sequence conserved money.

The snapshot is useful telemetry, but its comment promises "every minor unit" and the engine never
compares opening stock, closing stock and authorized issuance/write-off. It must not be used as proof
of conservation until every transaction posts balanced entries to a canonical ledger.

## AD-4. Cohort ageing is a batch transition with no replacement population - Major

The residual search confirms that production code never constructs an `AgeBand::Child` cohort.
`AgeBand::Child` appears only in ageing and health rules; all other creation sites are content or
tests. Existing cohorts advance as indivisible blocks after annual population rescaling. There are
no births, household formation, parent-child links or entry of new child cohorts.

Consequently C-3 is no longer an open question. A long run mechanically empties the younger age
bands while the population-rate formula can simultaneously add anonymous people to older cohorts.
The demographic total and the age structure are two contradictory histories.

---

# AE. Audit closure and evidence boundary

This closes the economy/formula pass at the source level. All modules that own money movement,
production, demand, prices, labor, firms, credit, insolvency, investment, public finance, health,
demography, migration, logistics, reserves, programs and the monthly/annual orchestration have been
read either line by line or through every economically active path. `chronicle.rs` was checked as an
event consumer; identity, RNG, serialization and display-only code were excluded unless they owned
an economic transition.

Residual cross-file searches established the following negative facts:

- no production path creates a child cohort;
- no production path assigns `EmploymentStatus::Employed` after registration;
- annual closure is the only production caller of `HouseholdCohort::set_people`;
- terminal storage cost has no economic reader;
- terminal admission is exposed only through an explicit command, not the simulation clock;
- the money-stock snapshot has exactly four liquid categories and no balancing journal.

The defect register should now be treated as architecture input, not a patch queue. Several cheap
local edits in T-7 would improve symptoms, but applying them independently would preserve the same
split ledgers and contradictory sensors. The implementation sequence is therefore moved to a
separate rebuild plan with invariants, gates and vertical slices. No Rust source was changed during
this diagnostic pass.
