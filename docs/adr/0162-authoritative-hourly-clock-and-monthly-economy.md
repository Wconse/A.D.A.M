# ADR 0162: Authoritative hourly clock with monthly economic settlement

## Status

Accepted.

## Context

The first observatory advanced whole economic years and rendered its date only during startup. That made the header stale after a transition and left no time scale for decisions that should occur between economic settlements.

## Decision

Add `hour_of_day` to the authoritative `World`, include it in serialization and the stable fingerprint, and expose replayable `WorldCommand::AdvanceHour`. Twenty-four hourly commands advance one calendar day. Crossing the final hour of a month atomically executes one monthly economic cycle before entering the next month. Existing explicit monthly and yearly APIs remain available for tests and non-realtime clients.

The Bevy client runs the authoritative clock continuously and supports 1, 4, 12, 24, and 72 simulated hours per real second. Space pauses, Up/Down changes speed, and H advances exactly one hour. Player program commands continue to use the same world boundary at the current authoritative hour.

The date header is now a reactive component rebuilt whenever the snapshot or time-flow state changes. The snapshot exposes calendar day, month, year, and hour.

## Invariants

1. The UI does not maintain a competing calendar.
2. Equal hourly command histories produce equal worlds and fingerprints.
3. A month-end economic failure prevents the boundary hour from committing.
4. Exactly one monthly economic cycle is executed at each crossed month boundary.
5. Pausing and speed selection are presentation state; elapsed hours are authoritative commands.

## Visual pass

The observatory now uses a structured top command bar and distinct inspector, program, and timeline surfaces over a consistent navy/teal political palette. This remains an iterative game-interface pass rather than final art.

## Compatibility

The new serialized and fingerprinted clock changes authoritative state shape. `SIMULATION_VERSION` advances from 94 to 95. Older serialized worlds default the hour to midnight when loaded through serde-compatible paths.
