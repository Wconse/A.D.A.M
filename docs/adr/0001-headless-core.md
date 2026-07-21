# ADR 0001: Engine-independent simulation core

- Status: Accepted
- Date: 2026-07-21

## Context

The game will eventually use Bevy for 2D presentation, but its history must run headlessly, faster than real time, and identically in tests and replays.

## Decision

All authoritative simulation rules live in `adam-core`, which has no dependency on Bevy, egui, storage engines, or operating-system time. Applications communicate through explicit commands, state snapshots/read models, and domain events.

## Consequences

- Console simulation can validate the central hypothesis before graphical investment.
- Rendering can be replaced without rewriting the simulation.
- Integration code must translate between engine entities and stable domain IDs.
- Some duplication between domain storage and future render ECS is acceptable to preserve the boundary.
