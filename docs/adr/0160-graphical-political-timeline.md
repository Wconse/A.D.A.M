# ADR 0160: Graphical political timeline from authoritative chronicle

## Status

Accepted for Step 100.9.

## Context

The observatory needs to show how present conditions emerged. Reinterpreting raw events in Bevy would duplicate chronicle logic and risk disagreement with the CLI.

## Decision

Render a persistent political-timeline panel from `ObservatorySnapshot.chronicle`. The panel shows the four latest deterministic annual entries with year, importance, and complete event-derived narration. It refreshes only after a successful command rebuilds the snapshot.

The client performs presentation ordering and truncation only. Program promise, appropriation, execution, losers, polarization, and legitimacy wording continues to originate in the shared core chronicle.

## Invariants

1. Bevy does not reinterpret domain events into a competing history.
2. Timeline entries are ordered by authoritative chronicle year.
3. Rejected commands do not invent timeline entries.
4. Successful commands refresh the timeline from the same snapshot as map and desk.
5. Empty histories render an explicit empty state.

## Consequences

The graphical client now connects player decisions to visible historical consequences. No simulation-version change is required. Step 100.10 is the anniversary release gate and integration audit.
