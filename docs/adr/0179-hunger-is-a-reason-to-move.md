# 0179 — Hunger is a reason to move, and an exodus stops only at a real limit

## Status

Accepted. Simulation version 105.

## Context

Internal migration already existed and already ran every year. It was not dead
code: `execute_annual_internal_migration` is called from the annual stages. It
simply could never fire in a world like the demo, because of what it asked for.

A household could leave only if all of the following held:

- its region had carried unemployment pressure for at least six months,
- some other region had carried *vacancy* pressure for at least six months,
- and the household itself was **unemployed** and free of any standing
  employment agreement.

Every cohort in the demo scenario is employed. No region ever advertises
vacancies once firms are fully staffed. So no household could ever move, no
matter what was happening to it. Eastport starved to zero output beside two
regions that still had food, and its people stayed and died there.

That is a limit nobody pays for. Nothing in the world enforces "you may leave
only once somebody elsewhere is hiring". Famine has emptied regions in every
century without a labour contract waiting at the other end. Under the project's
own rule — every limit must name the party that pays for it — the rule had to
go rather than be tuned.

A second unowned limit sat next to it: the annual pass moved **at most one
household per region per year**. Nothing pays for that either. A region losing
its food supply does not lose exactly one family a year.

## Decision

### Hunger opens the door the labour market keeps shut

A household may now leave because it cannot eat where it is, whether or not
anybody is hiring and whether or not it holds a job.

- A region's provisioning is measured by `regional_survival_fulfillment`: the
  people-weighted survival fulfilment of the cohorts living there. This is the
  same observed monthly health record that already decides who loses capacity
  and who dies, not a new forecast invented for migration.
- Below `HUNGER_FULFILLMENT_BPS` (5 000) the region is not merely poor: its
  households cannot feed themselves in place.
- A refuge must be better fed by at least `HUNGER_RELIEF_MARGIN_BPS` (1 000).
  Moving to an equally hungry neighbour buys nothing and costs a fee, so it
  does not happen.
- The refuge must have a dwelling free, exactly as the labour-market path
  already required.

### Both sides pay

- **The mover pays twice**: the relocation fee, and the job. A household that
  leaves the region that employed it gives up its work — the standing agreement
  is reduced at the employer it abandons (`release_migrating_workers`) and the
  household arrives unemployed with no income (`leave_employment`). The whole
  reduction is planned before any of it is applied, so a move that cannot be
  unwound cleanly from an employer is abandoned instead of left half-done.
- **The employer left behind pays** by losing the staff, immediately, without
  compensation and without a replacement waiting.
- **The refuge pays** in housing pressure and in more mouths competing for the
  same food. Arrivals enter the destination's own provisioning average, which
  is what eventually closes the gap that attracted them.

### An exodus stops at a real limit, not at a counter

The per-region annual cap of one household is gone. Migration now continues
while there is still somebody who wants to leave and can, and it terminates on
owned constraints only:

- the destination runs out of dwellings,
- the remaining households cannot fund the reserve plus the fee,
- or arrivals have dragged the refuge's provisioning down until it is no longer
  materially better fed than the place being left.

The destitute stay behind. Selection prefers the best-provisioned hungry
household, because moving costs money, and the households with nothing left are
precisely the ones who cannot buy their way out.

### Arrivals join the people they resemble

The first measured run exposed a cost that had nothing to do with economics: a
permanent new cohort per migrating family. After a 500-person exodus the world
carried hundreds of one-household cohorts forever, and a simulated year slowed
from 139 ms to 870 ms.

Arrivals now join an existing destination cohort that lives the same way — same
need profile, age band, years in band, household type, education and employment
status (`HouseholdCohort::lives_like` / `absorb`). People, dwellings, money and
debts are carried across; nothing is created or lost. Health records are merged
by the number of people behind each (`CohortHealth::blended_with`), so a
starving family joining a fed group carries its famine into the joined record
instead of having it washed away by the arithmetic of merging. When nobody at
the destination lives that way, a new cohort is still created — the first
refugees found the group that later ones join.

## Consequences

Measured on the 50-year demo run, seed 1:

| | before (ADR 0178, v104) | after (v105) |
|---|---|---|
| fingerprint | 6457675414769505211 | 15330073222654020652 |
| household migrations | none, ever | 247 households / 733 people |
| excess deaths | 3179 | 3151 |
| speed | 139.4 ms/year | 140.3 ms/year |

The exodus is visible in the chronicle: in 2040 one hundred households leave
Northreach for Southvale at once, followed by three more waves, and it stops
when projected housing pressure reaches the top of the scale. That is the first
time in this project's history that a population has responded to a regional
collapse by going somewhere else.

It is not a rescue. Deaths fell by 28 out of 3179. Moving relieves the mover,
but the demo scenario's real defect is elsewhere: households can still only eat
bread, no household can produce anything for itself, and entry into production
remains blocked by `RaiseExceedsWorkValue` and `NoSolventFounder`. Migration
can redistribute people across a set of regions that are all failing; it cannot
make any of them produce.

## Known gaps

- The relocation fee is near zero in the demo, because housing prices are near
  zero, because construction never happens. The fee is real machinery attached
  to a market that is not yet alive.
- A migrant's recorded skills are those of the group it joins; skills held by
  the arriving household are lost in the merge. This is a loss, not a free
  gain, but it should eventually be blended like health.
- Migration is internal to a country only. Cross-border movement is not
  modelled.
- Hunger is judged on the annual pass. A famine that begins and ends between
  two annual checks moves nobody.
