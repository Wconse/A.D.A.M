# 0186 - A firm distributes what it earned, never its capital

## Status

Accepted. Simulation version 113.

## Context

ADR 0185 gave firms a prudence reserve and found it almost inert. The
measurement that mattered was this: in the first simulated year owners were paid
979072 minor units - approximately the entire opening capital of every firm in
the scenario. No reserve sized by a monthly wage bill could restrain a payout
sized by a founding endowment.

The cause was that the world could not tell capital from profit. `accounting.rs`
recorded only the current month and `reset_monthly_firm_accounts` wiped it. The
day after a firm was founded, the money its founder had put in sat in the same
till as trading income and was indistinguishable from it. Every dividend rule
built on "cash the firm can spare" was therefore a rule for handing back capital.

A diagnostic run of the boardroom test world proved how deep this went: over
twelve months of trading the bakery sold 100 minor units, earned -20, and its
till fell from 10000 to 5568 - and the old rule still paid a dividend every
year. Two long-standing tests asserted that this payout must happen. They were
asserting that a loss-making firm must be liquidated into its owner's purse.

## Decision

A firm may hand over what it has earned and never the capital it was founded
with.

1. `World::firm_retained_earnings` records, per firm, trading income less the
   cost of the work: `sales_revenue - wages_paid - input_spend`. It is accrued
   in `capture_monthly_firm_observation`, the point at which the month is
   closed and all three are known. A loss is recorded as readily as a profit.
2. Money put *into* a firm never touches the ledger: the opening endowment, an
   owner's injection against wage arrears and a lender's principal all raise the
   till without raising earnings, because none of them was earned.
3. `execute_observed_owner_distributions` distributes the policy share of
   `min(free_cash - prudence_reserve, retained_earnings)`, and subtracts what it
   pays from the ledger so the same profit cannot be distributed twice.

No constant was introduced. The bound is the firm's own trading record.

## Consequences

Measured over fifty years at seed 1, version 113, fingerprint
5952391117261998594 (39.329 s, 786.6 ms per year), 229 tests green:

- The first-year liquidation is gone. Owners end year one holding 3193 minor
  units instead of 979093, and firm tills hold 3312078 instead of 2336178.
- Firm tills are no longer emptied. They end at 1156973 rather than 4666 - a
  working economy's worth of capital stays where the work is done.
- Owners are paid in all 50 years rather than 32, but out of earnings: 1457940
  in total against 2840491 before. Founding continues; 38 years record supply
  growth.
- The collapse is delayed by six years. Output was nothing from 2038; it is now
  nothing from 2044, and 2041 records 413437 against the 264000 that the old
  economy managed at its best.
- Excess deaths are 3140 against 3085, and the total money stock is 3549035
  against 3520448.

## Known gaps

- **The collapse is delayed, not cured.** Output still falls to nothing, and
  households still end holding less than owners. Keeping capital inside firms
  buys years; it does not close the circuit.
- **The boardroom test world barely trades.** Twelve months of `hungry_market_world`
  produce 100 minor units of sales against a standing wage bill. Whether this is
  a defect of the scenario or of household purchasing is not yet known, and it
  makes that world a poor witness for anything about profit.
- Capital spending is not treated as an expense, so a firm that builds capacity
  keeps the earnings that paid for it on its ledger. This overstates what may be
  distributed in the years after an investment.
- A firm's payroll obligation, and with it the prudence reserve, falls to zero
  when its employment agreements lapse (ADR 0185).
