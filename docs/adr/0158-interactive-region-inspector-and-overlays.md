# ADR 0158: Interactive region inspector and map overlays

## Status

Accepted for Step 100.7.

## Context

A static map proves rendering but does not let a player interrogate the state. The first interaction must remain presentation-only and must not introduce an alternate simulation model.

## Decision

Add client-owned inspector and overlay resources to `adam-observatory`. Tab cycles the canonical region selection. Numeric keys switch between confidence, population, and annual-output overlays. Region sprites are recolored from immutable snapshot values, and the selected region receives a visible highlight.

A fixed inspector panel displays region and country identifiers, population, annual output, and confidence. Overlay controls remain visible at all times. Selection and overlay choice are ephemeral UI state and therefore do not enter core saves, events, or fingerprints.

## Invariants

1. Overlay switching never mutates the simulation snapshot.
2. Region selection follows canonical snapshot order.
3. Every color ratio is bounded to `0..=1`.
4. Missing or zero maxima remain safe and deterministic.
5. Inspector data is copied only from the authoritative snapshot.

## Consequences

The observatory is now interactive: the player can inspect regions and compare three foundational layers without advancing time or changing policy. No simulation-version change is required. Step 100.8 adds a command-backed government-program desk.
