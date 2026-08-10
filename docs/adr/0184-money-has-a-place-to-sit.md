# 0184 - Money has a place to sit, and the world could not see where

## Context

ADR 0183 closed the owning side of the circuit: firms pay their owners, owners
found firms, and 152 firms appeared in fifty years where none ever had. The
world still died - output nothing from 2038, every region emptied - and the
usual suspects had all been cleared. Payroll genuinely credits household
wealth every month. Households genuinely budget from that wealth. Health is
measured after trading, not before it. Goods were produced and sold.

What could not be answered was simpler than any of those: **where is the money**.
Nothing in the simulation ever counted it. A drained household purse and a
shortage of bread produce the same symptom - unmet survival demand - and the
world had no way to tell them apart.

## Decision

Count it. `observed_money_stock` sums every minor unit in the world into four
hands: household liquid wealth, firm tills, owner purses, and national
treasuries. It is a pure measurement - no rule reads it, nothing branches on
it - emitted once at the close of each economic year as `MoneyStockMeasured`
and rendered by the CLI.

One invariant is pinned in a test: a payout to owners moves money between
hands and never makes any. Whatever the owners gain, the firms lose, to the
coin, and the total is unchanged.

## Consequences

The first run with the counter in place answered the question immediately.

| year | households | firm tills | owners | treasuries | all |
| --- | --- | --- | --- | --- | --- |
| first | 144744 | 2332482 | 982789 | 0 | 3460015 |
| +5 | 681667 | 378743 | 2432376 | 2305 | 3495091 |
| +8 | 777935 | 216466 | 2508116 | 2325 | 3504842 |
| last | 1121959 | 4666 | 2387768 | 6328 | 3520721 |

The total is near enough constant. Nothing leaks out of the world. But inside
it, the money walks one way: out of firm tills, which fall from 2.33 million to
four thousand, and into owner purses, which take and keep roughly **68% of all
the money in the world** for the rest of the run. Households never hold more
than a third of it, and the whole economy trades against that third.

The reason is structural, and this slice only makes it visible: an owner has
exactly one way to spend. Founding a firm pays capital outlay to local
households - and founding is blocked by `RaiseExceedsWorkValue` once there is
no unemployment left to hire from. An owner does not eat, does not rent, does
not buy anything, and does not lend a coin. Every dividend that reaches an
owner therefore leaves circulation permanently. The bust after 2035 is not the
world running out of goods; it is the world running out of buyers with money,
while two and a half million minor units sit in seven purses.

## Tests

- `paying_the_owners_moves_money_without_making_any`: after a profitable year,
  a distribution raises the owners' holding, lowers the firms' holding by the
  same amount, and leaves the total identical.
- Workspace green: 227 library tests in adam-core, 9 in adam-content, all
  integration targets, 0 failures. fmt, clippy `-D warnings` and rustdoc
  `-D warnings` clean.

## Measurement

Simulation version 111. Fifty years, seed 1: final fingerprint
`16134026149314799983`, 35.521 s (710.4 ms per year). Behaviour is unchanged
from version 110 - the trajectory of output is identical, year for year - as a
pure observation should be.

## Known gaps

- An owner is a sink. Owner cash returns to circulation only through founding,
  and only households and firms buy anything. The next slice must give owners
  the ordinary demands of a person who lives somewhere.
- Treasuries hold almost nothing and the total money stock rises slightly in
  the early years, which is public borrowing creating deposits. That is worth
  its own measurement later; it is small compared to the owner sink.
- `RaiseExceedsWorkValue` still bars entry once unemployment is gone.
