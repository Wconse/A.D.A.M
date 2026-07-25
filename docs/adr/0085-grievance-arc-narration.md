# 0085. Grievance arc narration in the chronicle

## Status

Accepted (step 036).

## Context

Since step 033 the simulation derives bilateral grievance from unmet import
demand, escalates it into hostility at a deterministic threshold, and (since
step 034) de-escalates emergent hostility when the grievance has decayed to
zero. All of that is journaled through typed domain events, but the yearly
chronicle only narrated the hostility flips themselves. The causal arc -
material grievance deepening, breaking into hostility, easing, and being
resolved into peace - was invisible in the year-by-year story, even though
`BilateralGrievanceChanged { aggrieved, target, level }` events already carry
everything needed.

## Decision

- The chronicle's per-year summary now tracks, per directed country pair, the
  first and last grievance level observed during the year (a `BTreeMap` keyed
  by `(aggrieved, target)`, so narration order is deterministic).
- Each pair contributes one sentence per year, placed before the hostility
  sentences so cause precedes effect: a year ending at level zero reads
  "X resolved its material grievance against Y."; a year that ends at or above
  its first observed level reads "X's material grievance against Y deepened to
  N basis points."; otherwise it reads "... eased to N basis points."
- Grievance and hostility narration moved into one `push_conflict_narration`
  helper so the year-finishing function stays under the Clippy line budget.
- Importance tiers: a year with any hostility change now ranks 85 (between
  sub-half survival crises at 90 and rationing at 80), and a year whose only
  notable diplomacy is grievance movement ranks 50 (between household
  borrowing at 60 and ordinary production at 40).
- The chronicle stays a pure derivation from domain events: no world state, no
  new events, `SIMULATION_VERSION` stays at 34, and the seed-1/50-year
  baseline fingerprint is unchanged (`8818694516742230572`).

## Consequences

- The emergent conflict arc is now legible in the narrative: readers can see a
  grievance deepen, explode into hostility, ease after substitution, and
  resolve into peace, each attributed by country name.
- A new gate test (`chronicle_narrates_the_grievance_arc_by_country_name`)
  pins a two-year escalation-then-peace story, including importance 85 for
  both years.
- Deliberate limits: the chronicle reports first-vs-last level per year, not
  intra-year oscillations; ties (last equals first) read as "deepened to" the
  unchanged level; and grievance names only countries - regional or household
  attribution of the underlying shortage stays with the existing shortage
  sentences.
