# ADR 0159: Command-backed playable government-program desk

## Status

Accepted for Step 100.8.

## Context

The observatory must become playable without creating a second simulation inside Bevy. UI controls must use the same authorization, fiscal limits, atomic rejection, events, and replay boundary as AI and tests.

## Decision

Add an observatory scenario with a legitimate cabinet actor, political office, and announced national-renewal program. Scenario setup is content construction; every player decision after startup is an existing `WorldCommand`.

The Bevy client retains the authoritative `World` behind a resource and exposes treasury funding (`T`), debt funding (`D`), execution (`E`), cancellation (`C`), and annual advance (`Y`). Successful commands rebuild the immutable `ObservatorySnapshot`; rejected commands leave the world untouched and display the exact domain error.

The desk displays status, promised funding, cumulative appropriation, delivered funding, carryover, and delayed years. A focused test proves an authorized UI-equivalent command changes the authoritative fingerprint before the refreshed snapshot exposes the result.

## Invariants

1. UI actions never mutate program fields directly.
2. Every decision passes through `WorldCommand::apply`.
3. Rejected commands do not refresh from invented state.
4. Successful commands refresh all presentation from the authoritative world.
5. UI state and error messages remain outside saves and fingerprints.
6. The core remains independent of Bevy.

## Consequences

The graphical client now supports the first real player decisions and honestly exposes fiscal or legal rejection. No simulation-version change is required because no core transition or save shape changed. Step 100.9 adds a graphical political timeline sourced from chronicle snapshots.
