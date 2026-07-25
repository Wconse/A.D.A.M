# 0084. Replayable firm registration

## Status

Accepted (step 035).

## Context

Every authoritative transition in A.D.A.M. is supposed to be expressible as a
`WorldCommand`, so that any history can be replayed bit-for-bit from a journal
of commands. Firm registration violated this: `World::register_firm` existed
only as a direct method, with no command wrapper. The gap was not theoretical -
two gate suites in a row (`bilateral_grievance.rs` in step 033 and
`hostility_deescalation.rs` in step 034) needed to introduce a firm mid-history
and had to work around the missing command by applying the same direct
registration to both the direct and replayed timelines, beside the journal
rather than inside it. ADR 0083 recorded this as a deliberate limit.

## Decision

- Add a `RegisterFirm(Firm)` variant to `WorldCommand` that delegates to the
  existing validated `World::register_firm` transition. `Firm` already derives
  the serde traits, so the command serializes with the rest of the journal.
- The variant is appended at the end of the enum so that previously recorded
  command journals keep their encoding under index-based serialization formats.
- No new world state and no rule change: registration validation (duplicate
  firm, unknown region, recipe, or inventory good) is unchanged, and failed
  registration remains atomic. `SIMULATION_VERSION` stays at 34 and the
  seed-1/50-year baseline fingerprint is unchanged (`8818694516742230572`).
- The step 034 de-escalation gate now registers its mid-history domestic
  producer through `RegisterFirm` on both timelines, exercising the command in
  a real scenario.

## Consequences

- Structural change to the economy - a new producer entering an existing
  history - is now journaled like every other transition, so scenarios with
  firm entry replay from commands alone.
- A new command-boundary test (`firm_registration_is_replayable_and_atomic`)
  pins replay equality (world and fingerprint) and atomic rejection of a
  duplicate registration.
- Deliberate limits: firm entry remains an exogenous act with no in-simulation
  cause (no entrepreneur model, no entry costs, no capital raising); ownership
  stakes, appointments, and actor registration still have no command wrappers
  and can be added the same way when a gate needs them.
