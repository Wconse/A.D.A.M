# 0090. Shared survival supply planning

## Status

Accepted (step 041).

## Context

Step 040 quoted each household's survival basket as a deterministic local-first blend, but every
cohort quoted the offer book independently. Two cohorts facing the same scarce offer could each
reserve its full quantity during planning, overstating aggregate reservation relative to what
canonical market clearing would actually allocate. The roadmap flagged planning-time source
capacity allocation across ordered household intents as the next gate.

## Decision

`plan_monthly_household_demand_against_offers` now builds one sorted planning copy of the offer
book (region, good, unit price, seller — the exact supply order the market uses) together with a
mutable remaining-quantity ledger shared across all cohorts. Cohorts are planned in the same
canonical order the market fills buyers (region, then cohort id), and every survival quote
decrements the ledger, so a later cohort can no longer quote quantity already claimed by an
earlier one. Any unquotable remainder falls back to the regional reference price, preserving
prior no-supply behavior. `monthly_survival_cost` keeps quoting against an empty,
non-consuming ledger.

The ledger consumes quantities only while every stored government emergency policy uses the
default `MarketAllocation` physical-shortage strategy. Under `ProportionalRationing` planning
keeps the previous independent per-cohort quotes, because rationing rewrites order quantities
with proportional quotas before clearing, and shared planning-time consumption would fight that
allocation.

## Consequences

With one scarce foreign offer of 1000 milli-units at delivered price 11 and two identical
cohorts, the first canonical buyer reserves 11 and buys the whole offer; the second cohort
reserves the reference price 10, its need reaches the market as explicit unmet demand, and the
month replays bit-identically. The proportional-rationing gate is unchanged. The full gate
passes with 116 tests; version 34 and release fingerprint `12100901864703017553` remain
unchanged (35.7 ms per simulated year).

Known limits: only survival tiers consume the ledger, non-survival tiers still quote reference
prices; emergency-relief costing remains reference-priced; and routes remain immediate
accounting links rather than capacity-constrained shipments.
