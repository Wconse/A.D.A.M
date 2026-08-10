# 0185 - Prudence comes before profit

## Status

Accepted. Simulation version 112.

## Context

ADR 0183 gave owners an income; ADR 0184 measured where the money then sat. The
measurement was damning: firm tills fell from 2332482 to 4666 minor units while
owner purses held about 68% of every unit in the world.

The rule behind that drain was `execute_observed_owner_distributions`, which
treated as free cash everything above a single month's bill:

```
free_cash = cash - (payroll + wage_arrears + inputs + investment)
```

A firm that had survived for years on a thin margin therefore held nothing at
all against the next thin month. One month of weak sales left it unable to meet
payroll; arrears followed; the households it employed stopped being customers;
sales fell further. The firm's own payout policy was feeding the loop that
killed it.

No real firm distributes on that basis. A board keeps something back, and what
it keeps back is judged from what it has actually lived through.

## Decision

Before profit, prudence.

1. `FirmObligationTerms::recurring_monthly_cost()` names the cost of simply
   running the firm for another month: `payroll + inputs`. Wage arrears and
   investment commitments are excluded deliberately - they are one-off claims,
   not the standing cost of doing the work.
2. `observed_prudence_reserve(firm)` reads the firm's own operating memory and
   returns the worst shortfall it can remember: the largest value of
   `recurring_monthly_cost - sales_revenue` across the recorded months. A firm
   that has never known a bad month reserves nothing. Memory is bounded by
   `FIRM_OBSERVATION_HISTORY_LIMIT` (12 months), so an old scar heals.
3. The board distributes its policy share of `free_cash - reserve`, and pays
   nothing when that is not positive.

The reserve is evidence, not a rule of thumb. It contains no tuned constant: its
size is entirely a function of the firm's recorded trading history.

## Consequences

Measured over fifty years at seed 1, version 112, fingerprint
5898783262672582167 (37.211 s, 744.2 ms per year), 228 tests green:

- The mechanism is correct but almost inert in the demo world. Owner purses end
  at 2382218 rather than 2387768; firm tills still end at 4666; output is still
  nothing from 2038; the total money stock is unchanged at about 3.52 million.
- The reason is now visible in the numbers. In the very first year owners were
  paid 979072 minor units - approximately the entire opening capital of every
  firm in the scenario. A prudence reserve sized by a monthly bill of a few
  hundred units cannot restrain a payout sized by a founding endowment of
  millions.
- The private credit market, the one channel that could return owner money to
  production, moves 6, 8 and 39 minor units in the years it operates at all. It
  is four orders of magnitude too small to be the return path.

## Known gaps

- **A firm may still distribute its own capital.** The world has no notion of
  paid-in capital or retained earnings; `accounting.rs` records only the current
  month. Every unit a founder puts in is indistinguishable from profit the day
  after founding, so the first year's payout is a liquidation dressed as a
  dividend. This is the direct cause of the drain that ADR 0184 measured, and it
  is the subject of the next slice.
- A firm's payroll obligation falls to zero once its employment agreements
  cease to be active, which silently zeroes the reserve as well. Why an
  agreement lapses without renewal is not yet understood.
- Reserves could in principle starve founding by leaving owners with nothing to
  accumulate. In this run they did not - payouts continue in 32 of 50 years -
  but the risk returns once the reserve becomes large enough to bite.
