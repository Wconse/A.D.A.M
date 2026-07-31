# ADR 0161: Step 100 anniversary acceptance gate

## Status

Accepted. Step 100 is complete.

## Context

The anniversary milestone joins a large authoritative simulation slice with the first graphical client. Completion requires proving that graphics do not fork the model and that the complete promise-to-consequence chain remains deterministic.

## Decision

Accept Step 100 when all workspace tests, Clippy with warnings denied, formatting, documentation, release compilation, and diff checks pass together. Add an end-to-end observatory acceptance test that applies program commands to one world, replays the same command sequence into an equal world, and compares both core fingerprints and complete visual snapshots.

The accepted vertical includes program charters, treasury/debt decisions, cancellation, contested physical execution, materials, temporary labor, infrastructure, regional memory, delayed political consequences, shared chronicle, Bevy map, inspector, overlays, command desk, and graphical timeline.

## Invariants

1. Equal command histories produce equal core fingerprints and equal visual snapshots.
2. The graphical client never owns simulation rules.
3. Player and replay commands use the same transition boundary.
4. Chronicle and timeline remain derived from typed evidence.
5. Release acceptance includes the entire workspace, not only the new application.

## Consequences

A.D.A.M now has its first complete playable governing-state observatory. The current graphics are an engineering visual baseline, not the final art direction; future steps may replace composition and styling without changing the accepted simulation boundary. `SIMULATION_VERSION` remains 94.
