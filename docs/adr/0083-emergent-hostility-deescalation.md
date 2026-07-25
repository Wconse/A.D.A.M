# 0083. Emergent hostility de-escalation

## Status

Accepted (step 034).

## Context

Step 033 made bilateral hostility reachable from inside the simulation: sustained
unmet import demand with reachable foreign supply accrues a directed material
grievance, and when the grievance level crosses 7500 basis points the world
escalates the country pair to hostility through the same journaled
`set_country_hostility` transition that commands use. Escalation was a one-way
door: once hostile, a pair stayed hostile forever unless an external
`SetCountryHostility` command intervened. The step 033 roadmap entry named the
next gate explicitly - when the grievance that caused an emergent hostility has
decayed to zero with no fresh material evidence, the hostility should
deactivate deterministically through the same journaled transition.

A naive rule such as "deactivate any hostility whose grievances are zero" would
also dissolve hostilities that were commanded from outside the simulation.
Commanded hostility is an exogenous political fact; the material grievance
system has no standing to cancel it, because nothing material caused it.

## Decision

- The world now remembers which hostilities it created itself. A new canonical
  set `emergent_hostilities: BTreeSet<(CountryId, CountryId)>` marks country
  pairs whose hostility was raised by the grievance escalation path.
  `mark_emergent_hostility` inserts the canonical pair at escalation time.
- After the monthly grievance update, `deescalate_resolved_emergent_hostilities`
  scans the emergent set. A pair whose directed grievances have both decayed to
  removal (no entry in either direction remains in `bilateral_grievances`) is
  deactivated through the existing `set_country_hostility(first, second, false)`
  transition, which emits the same `BilateralHostilityChanged` event that
  commands emit and clears the emergent marker.
- Commanded transitions own the marker: `set_country_hostility(..., false)`
  always removes the emergent marker, so a commanded peace resets the pair to a
  clean slate, and a commanded hostility (never marked emergent) is never
  touched by the de-escalation scan.
- The emergent set is canonical state: it participates in the stable
  fingerprint (length plus each canonical pair) and is serialized with the
  world, so replayed histories agree bit-for-bit. `SIMULATION_VERSION` is bumped
  from 33 to 34.

## Consequences

- Peace is now reachable inside the simulation. The full material arc is
  closed: shortage -> grievance -> hostility -> embargo -> substitution or
  recovery -> grievance decay -> peace, all through journaled, replayable
  transitions.
- Commanded hostility remains strictly stronger than the material system: it
  never de-escalates on its own, matching the intuition that exogenous
  political facts need exogenous resolution.
- Because grievances decay at 250 basis points per calm month and escalation
  requires 7500, an emergent hostility lasts at least 30 calm months before
  peace, giving embargoes real duration without hand-tuned timers.
- Gate tests (`tests/hostility_deescalation.rs`, 3 tests) pin the invariants:
  an emergent hostility de-escalates after its material cause ends (with replay
  equality), a commanded hostility without grievance never de-escalates, and a
  commanded peace clears the emergent marker.
- Deliberate limits: de-escalation only consults grievance state, not broader
  diplomacy; firm entry is still not a replayable command, so the de-escalation
  gate test registers its domestic substitute producer on both timelines
  directly; only firms with an enacted policy plan market offers, so material
  recovery in the test requires a governed domestic producer.
