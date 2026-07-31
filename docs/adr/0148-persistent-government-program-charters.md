# ADR 0148: Persistent government-program charters

## Status

Accepted for Step 100.1.

## Context

Regional service allocation is currently a recurring policy instruction. It cannot preserve the identity of a named political promise, its initiator, intended beneficiaries, promised money and outcome, duration, or the later gap between promise and delivery.

Step 100 needs a durable program arc before a graphical client can display meaningful political causality. The engine must also preserve the simulation-freedom rule: political over-promising is dangerous, but not physically impossible at announcement time.

## Decision

- Add `ProgramId` and a persistent `GovernmentProgram` collection to `World`.
- A charter records country, initiator, name, exact regional shares, promised annual funding, duration, service priority, promised improvement, start year, lifecycle status, appropriation, delivery, carryover, and delay memory.
- Declaration uses `WorldCommand::DeclareGovernmentProgram` and emits `GovernmentProgramDeclared`.
- Declaration validates identity, accounting, country/region ownership, scheduling, and actual government authority.
- Declaration does **not** reserve treasury cash and does not reject a promise merely because current cash is insufficient. Appropriation and execution are separate future transitions.
- Program state is serialized, replayable, inspectable, ordered canonically, and included in the stable fingerprint.
- Bevy remains absent from `adam-core`; the future viewer will project this state and submit the same command.

## Invariants

1. Regional shares total exactly 10,000 basis points; zero regional shares and full concentration remain legal.
2. Promised funding is non-negative and duration is positive.
3. Every target region belongs to the program country.
4. Only an actor holding real government authority may declare the charter.
5. A rejected declaration is atomic.
6. Announcement creates neither money nor delivered services.
7. Direct and replayed declarations produce identical state and fingerprints.

## Consequences

- A political promise becomes a first-class historical object instead of a one-year allocation note.
- Deliberate over-promising is representable and can later produce a broken-promise penalty.
- This slice does not yet appropriate or execute funds; the stored zeroed ledgers are explicit preparation, not simulated delivery.
- `SIMULATION_VERSION` advances from 85 to 86 because authoritative fingerprinted state changed.
