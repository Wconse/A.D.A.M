# 0187 - A works opens at the size of the hunger it answers

## Status

Accepted.

## Context

ADR 0186 stopped firms from handing their capital to their owners, and the
collapse moved six years later. It was a delay, not a cure: output still fell to
nothing. The next question was why people starved from the very first year while
firm tills were full, and the answer turned out not to be about money at all.

Three measurements settled it.

Payroll was innocent. `execute_monthly_payroll` pays `min(till, owed)` for every
standing agreement, credits the cohort, and carries the remainder as arrears.
Wages reach households exactly as they should.

The hunger was geographic. `survival_fulfillment_bps` compares what a cohort
actually consumed against the requirement written in its own profile, so zero
means no food arrived, not that no order was placed. Only Eastport starved. The
other two regions grew throughout.

And the shortage was structural, visible in the demo content itself. Two grain
farms produce a million milli-units a month; three bakeries need one and a half
million. Eastport is the only region of the second country, has no grain of its
own, and its bakery's opening stock covers exactly one month.

The world did try to answer. Entry pressure accumulated, founders were found,
and new grain works opened - each with `STARTUP_CAPACITY_BATCHES = 1`, a single
batch of four thousand milli-units against a deficit of five hundred thousand.
Every new firm was a hundred and twenty-five times too small to matter. The
queue at the counter could never clear, however much cash stayed inside firms.

A fixed startup size is a magic constraint of the plainest kind. Nothing in the
world measures it, and nothing can argue with it.

## Decision

Entry size is derived from three real quantities and nothing else:

- **The hunger that called the firm into being.** The unmet quantity already
  carried by the entry decision, divided by the recipe's output per batch.
- **The hands the region can spare.** `EntryRecruitment` now reports
  `available_workers` - idle people for an idle hire, the incumbent's crew for a
  poached one - converted into the batches they could run at one worker-month
  per thousand milli-units of recipe labour.
- **The money the founder can actually commit.** The founder is now chosen
  before the size, because how large a works can open is partly the question of
  whose wallet stands behind it. The cost of one batch is priced first, and the
  wallet divided by it gives the affordable count.

The smallest of the three is built. Cost scales with it in full: a month of
materials for every batch that will run, and three months of wages for every
worker hired. A firm no longer opens already unable to buy what it must convert.

Two consequences are kept honest rather than smoothed over. A crew taken from an
incumbent is removed from that employer in the same number it is hired in -
releasing one worker while hiring five would put more people to work than the
cohort contains, and the fifty-year run said so immediately with
`InvalidEmployment`. And a founder who cannot afford even one batch is still
refused with `NoSolventFounder`, exactly as before.

Plan validation moved into `check_founding_plan` and was strengthened: a plan
whose crew the region can no longer supply is refused, as is any plan carrying a
zero scale. A stale plan is never quietly repaired, because a founding that
changed its own terms would be a firm nobody agreed to build.

## Consequences

Over fifty years (version 114, fingerprint 5960198914824426029, 231 tests) the
supply side finally answers the demand it was shown. Output peaks at 1769829 in
2039 against 413437 in the previous version - more than four times higher - and
2038 alone reaches 813618 where the whole old economy never passed 274000. Fewer
firms are founded (21 supply-growth reports against 38) but each is built to a
size that can matter. Households end holding 1238463 rather than 979344, and
owners take 933131 over 45 years instead of 1457940 over 50, because capital now
goes into capacity rather than into purses. Deaths fall slightly, 3132 against
3140. The run is also faster: 29.8 seconds rather than 39.3.

The circuit is still not closed. Output collapses to nothing by 2043 after the
boom, and Eastport keeps starving with livelihood stress pinned at 7497 basis
points. What changed is the shape of the failure: it is no longer a world unable
to respond, but a world that responds hugely and then falls over. That is the
next thing to measure.

### Known gaps

- The boom-and-collapse of 2039-2043 is unexplained. A fourfold surge followed
  by zero suggests capacity is built against demand that the newly paid wages
  themselves created, then abandoned - but this is a guess until measured.
- Total money still grows over the run (3460015 to 3870378), most likely through
  government borrowing creating deposits. Carried from ADR 0184.
- Capital spending is still not counted as a cost when earnings are computed,
  so a firm that has just built capacity over-distributes. Carried from 0186.
- Employment agreements stop being active after annual closure and are not
  renewed. Carried from 0185.
- `RaiseExceedsWorkValue` still blocks most entry openings when nobody is idle,
  now the binding constraint on entry rather than firm size. Carried from 0184.
- Entry still picks the first recipe producing a good, and a converter cannot
  choose to protect its own input supply. Carried from 0183.
