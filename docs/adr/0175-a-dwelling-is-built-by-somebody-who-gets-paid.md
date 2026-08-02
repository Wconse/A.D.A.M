# 0175. A dwelling is built by somebody who gets paid

## Status

Accepted.

## Context

ADR 0170 closed the fiscal circuit for taxation: public spending stopped being
deducted and destroyed, and began reaching households as income. ADR 0172 closed
the same hole for the capital cost of founding a producer, paying the works to
the households that build them.

Housing was never audited that way. `execute_annual_housing_investment` debits
the treasury for the committed cost of a construction project and credits
nobody. The money leaves the world.

This was not found by reading the code looking for leaks. It was found by
extending the `fiscal_circuit` invariant to count every claim on the treasury
during a year, which produced a stubborn twelve-unit discrepancy in a world
small enough that twelve units mattered. The invariant had been comparing the
annual closure against treasury movement while two other channels - reserve
procurement and housing construction - were quietly spending from the same
purse.

## Decision

The money committed to a housing project is paid to the households of the region
that builds it, split by population with the same largest-remainder method used
for founding capital, and reported as a typed `HousingOutlayDistributed` event
per cohort.

A region with no households does not start a project at all. There is nobody
there to do the building, and a project with no builders would be the leak again
under a different name.

The split routine `plan_regional_capital_shares` is shared with firm entry rather
than copied. Two ways of dividing public money across the same cohorts would
eventually disagree, and the disagreement would be a rounding leak.

## Consequences

Public housing investment is now a transfer rather than a disappearance. A
government that builds is a government that puts money in local pockets, which
is how construction spending works in life and is a channel the model can now
reason about: building income raises household wealth, which raises what
households can buy, which reaches firms as revenue.

The fiscal invariant now names reserve procurement and housing construction as
claims on the treasury instead of ignoring them, so a future channel that spends
public money without paying anybody will show up as a failure rather than as a
quietly shrinking treasury.

## Known gaps carried forward

- Building income is paid in full at the moment the project starts, though the
  project takes years to complete. Real construction pays as work proceeds, and
  the schedule should follow the work once construction exists as an industry.
- Nobody is employed to build: the money arrives as household income without a
  job, an hour, or a firm behind it. ADR 0172 carries the same gap for capital
  works, and both point at the same missing thing - construction as a sector
  with its own firms, recipes, and labor demand.
- Reserve procurement pays a firm rather than households and is correct as it
  stands, but it is still not part of the annual fiscal closure the chronicle
  reports, so the narrative understates what the state actually spends.
