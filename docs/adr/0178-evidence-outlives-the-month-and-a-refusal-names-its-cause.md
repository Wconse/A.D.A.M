# 0178. Evidence outlives the month that recorded it, and a refusal names its cause

## Status

Accepted.

## Context

ADR 0176 recorded every unfilled input order against the buyer's region in
`monthly_firm_input_shortfalls`, and ADR 0177 gave a founder a way to staff a
new firm without waiting for unemployment. Both were gated by unit tests and
both passed. The fifty-year demo run founded exactly zero firms, and the
chronicle offered no reason at all.

Two separate defects were behind that silence, and neither could be seen from
the test suite.

**The evidence never reached the party entitled to act on it.**
`reset_monthly_firm_accounts` cleared the shortfall map at the end of the
monthly commercial cycle, and `execute_observed_firm_entry` runs after that
cycle. Every shortage in the demo was therefore written and wiped in the same
month, before entry could read a single one of them. The unit tests drove the
stages by hand and never crossed the reset, so the channel was alive in tests
and dead in every real month. This is the same class of fault as an unowned
constraint: a rule that binds the economy while nobody in the world is
responsible for it.

**A refusal to enter said nothing about itself.** `plan_firm_entry` returned
`Option`, so four completely different economic situations - nobody knows how
to make the good, nobody can be hired, there are no prices to judge by, nobody
can fund it - collapsed into one silent `None`, published as
`entry_feasible: false`. An economy that cannot answer a shortage was
indistinguishable from one that never heard it.

## Decision

- The shortfall map is no longer cleared by the monthly accounts reset. The
  procurement stage clears it at the moment it begins recording the next
  month's evidence. Evidence now lives from the month that produced it until
  the month that reads it, which is the only lifetime that means anything.
- `FirmEntryObstacle` names why an opportunity was declined: `NoKnownRecipe`,
  `NoWorkerToBeHad`, `RaiseExceedsWorkValue`, `NoLocalPrices`,
  `NoSolventFounder`. `FirmEntryOpportunityReviewed` carries it as
  `Option<FirmEntryObstacle>`; the absence of an obstacle below the entry
  signal means the region has not yet asked loudly enough, which is an absence
  of evidence rather than an obstacle.
- `plan_entry_recruitment` returns the same verdict type, so a region where
  somebody could have moved but the raise would exceed what the work earns is
  reported as `RaiseExceedsWorkValue` and never confused with an empty region.

## Consequences

- Fingerprint moves from `8238339567197527992` to `6457675414769505211` at
  simulation version 104, 139.4 ms per simulated year. The move is behavioral:
  entry now sees grain shortages it never saw before.
- Events are not hashed into the fingerprint, so naming the obstacle costs no
  determinism.
- Gated by `shortage_evidence_outlives_the_monthly_accounts_reset`, which pins
  the lifetime directly rather than trusting stage ordering.
- Measured over twenty demo years, entry now reviews grain in all three
  regions - Eastport alone reviews it 240 times, where before the fix it
  reviewed it never - and the refusals are these:
  - Eastport bread: `NoWorkerToBeHad` 86, `RaiseExceedsWorkValue` 149.
  - Eastport grain: `RaiseExceedsWorkValue` 75.
  - Northreach grain: `RaiseExceedsWorkValue` 18.
  - Northreach bread and Southvale grain and bread: `NoSolventFounder` 177.

## Known gaps

- Still zero firms founded in the demo. The economy is now blocked by two named
  facts rather than by a silent one: the raise needed to move a worker exceeds
  what the work earns at local prices, and where it does not, no resident holds
  enough cash to fund the capital and three months of wages. Both are honest
  economics and both are the subject of the next slices; neither may be tuned
  away.
- `RaiseExceedsWorkValue` is measured against the local output price, so a
  founder in a region that imports the good cannot yet judge a venture by the
  delivered price it would displace.
