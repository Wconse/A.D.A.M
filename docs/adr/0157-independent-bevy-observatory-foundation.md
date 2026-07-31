# ADR 0157: Independent Bevy observatory foundation

## Status

Accepted for Step 100.6.

## Context

A.D.A.M needs a visible face without allowing presentation code to become authoritative simulation logic. The first graphical client must consume deterministic state, not duplicate rules or mutate `adam-core` behind an alternative command path.

## Decision

Add `apps/adam-observatory`, a Bevy 0.16 client depending on `adam-core` and `adam-content`; core does not depend on Bevy. A read-only `ObservatorySnapshot` captures canonical date, fingerprint, regions, population, output, regional confidence, and chronicle entries. Stable region iteration assigns deterministic non-overlapping map coordinates.

The initial executable opens a 1280x720 window, renders a dark governing-state map, colors region cards by confidence, and labels population and output. The renderer owns only presentation entities; the immutable snapshot remains the boundary.

Bevy transitive dependencies are locked to Rust 1.85-compatible `image`, `skrifa`, and `font-types` versions.

## Invariants

1. `adam-core` remains independent of Bevy and graphical concepts.
2. Equal worlds produce byte-for-byte equal observatory snapshots.
3. Region positions derive only from canonical region order.
4. Rendering cannot mutate simulation state.
5. The displayed fingerprint is the authoritative core fingerprint.

## Consequences

A.D.A.M now has a runnable graphical executable and deterministic presentation adapter. No simulation-version change is needed because presentation does not alter saves or transitions. Step 100.7 adds interaction, inspector state, and switchable map overlays.
