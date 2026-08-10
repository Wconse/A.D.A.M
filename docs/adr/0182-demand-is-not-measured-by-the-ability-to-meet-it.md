# 0182 - Demand is not measured by the seller's ability to meet it

## Context

Since ADR 0181 the world could finally show a supplier that buyers had queued
at its gate and left empty-handed. Yet the fifty-year run still launched only
two capacity projects, and output collapsed to nothing by 2045. The remaining
obstacle was not credit, and not evidence: it was the yardstick itself.

`plan_observed_production_adjustments` built `market_demand_ceiling_batches`
from what the firm sold plus what its market wanted and did not get - and then
clamped that figure with `.min(physical[firm_id])`, the number of batches the
firm could physically make this month given its workers, its inputs and its
till.

The expansion rule then asked whether `market_demand_ceiling_batches` was at
least the firm's installed capacity. Since physical feasibility can never
exceed installed capacity, the clamped ceiling could never exceed it either.
Expansion was therefore permitted only in the single instant when a firm ran
exactly flat out with nothing binding it - no missing input, no thin till, no
unfilled shift. Any ordinary friction, and the market silently "stopped
wanting" the good. Hunger was being measured through the hungry party's
supplier's ability to feed it.

A diagnostic in a unit test made the contradiction plain: a bakery facing
9000 milli-units of unmet demand reported a market demand ceiling of zero.

## Decision

Two rules, drawn apart:

1. **The ceiling measures the market.** `market_demand_ceiling_batches` is what
   the firm sold plus what its market asked for and did not get, in batches,
   with no reference to what the firm can currently produce.

2. **The month's plan measures the firm.** The physical clamp moves to the one
   place it is truthful: the advisory production target for the coming month,
   which genuinely cannot exceed what this month's workers, inputs and cash
   allow. Monthly production behaviour is unchanged.

Alongside this, the reason to build was separated from the money to build it.
`observed_capacity_expansion_terms` reports how many batches a firm's own
trading record justifies and what one batch of capacity costs, knowing nothing
about the till. `observed_capacity_investment_need` turns that into a sum, and
that sum now enters `observed_firm_funding_gap` beside payroll, wage arrears
and inputs. A firm may borrow to grow, not only to survive; the lender judges
the application with the rules it already had - same rate, same underwriting,
same collateral, same portfolio share and liquidity reserve. No new constant
and no new threshold was introduced.

## Consequences

A firm that keeps selling out now keeps a standing reason to expand even while
it is short of inputs or cash, and can present that reason to a lender. Growth
stops belonging exclusively to firms that are already rich enough to pay
builders out of pocket.

The cost is that overshoot is now possible: construction takes three months,
and unmet demand recorded during those months does not yet know about the
capacity being built, so a firm can order a second expansion against demand the
first one will already cover. In the unit world a bakery reached 19 batches of
capacity against a market ceiling of 10. This is an honest defect of investment
under delay rather than an artificial cap, and it is left standing.

## Tests

- `a_firm_with_no_trading_record_has_nothing_to_borrow_against`: a firm that has
  never traded shows no investment need, so there is nothing to fund.
- `a_thin_till_does_not_erase_the_reason_to_build`: a profitable bakery kept at
  150 minor units still reports works it must build, the need exceeds its till,
  and the same need is reported when the till is filled with ten million -
  what the market justifies does not depend on what the till holds.
- The existing capacity, procurement and settlement suites are unchanged and
  green: 224 library tests, 0 failures across the workspace.

## Measurement

Simulation version 108. Fifty years, seed 1: final fingerprint
`11470311556797571776`, 12.175 s (243.5 ms per year; the funding gap now
consults the production planner per firm, which is the cost of the change).

Capacity projects launched across the run rose from 2 to 11. Output no longer
sits flat: 264000 in 2028, 292000 in 2035, 396000 in 2038, peaking at 521671 in
2040 - roughly double the previous ceiling. Annual excess deaths fall steadily
through the growth years, from 188 in 2027 to 15 in 2038.

## Known gaps

The circuit is still not closed. After 2040 the expansion reverses into a
collapse - 431013 in 2041, 326523 in 2042, 147035 in 2043, 4507 in 2045, and
nothing thereafter - and total excess deaths over fifty years (3153) are
unchanged from before. What was a permanent stagnation is now a boom followed
by a bust, which is a different and more informative failure: something
consumes the expanded economy rather than merely preventing it. That bust, not
the growth ceiling, is the next thing to explain.

Also still open: no new firms are founded in fifty years; households produce
nothing; `livelihood stress` holds at 7497 in Eastport for decades while every
cohort reports zero annual income.
