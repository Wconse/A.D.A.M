# ADR 0165: Russian strategic interface and map level of detail

## Status

Accepted as the third observatory design pass.

## Context

The strategic client mixed English interface copy with Russian project communication and showed the same amount of map detail at every zoom. A grand-strategy presentation needs one coherent language, state-level information density, and zoom-dependent visual hierarchy.

## Decision

Present the complete graphical command surface in Russian. Localize map modes, camera help, time state, program actions and statuses, regional inspection, tooltips, country and region names, city/capital labels, political timeline framing, rejection messages, and current demo chronicle summaries. Core identifiers and authoritative English content remain unchanged outside the Bevy presentation adapter.

Extend `ObservatorySnapshot` with deterministic country-level treasury, public debt, legitimacy, and elite-cohesion values. Render the selected region's country as a strategic resource bar in the masthead.

Introduce zoom-dependent map detail:

- country labels dominate the strategic zoom;
- province labels remain visible through operational zoom;
- city and terrain marks appear at close zoom;
- camera state remains presentation-only.

## Invariants

1. Localization never changes authoritative names, IDs, commands, or event evidence.
2. Country resource values come from the same immutable snapshot as the map.
3. Zoom only changes visibility, never simulation state.
4. Current demo entities have explicit Russian presentation names.
5. The graphical timeline never falls back to visible English prose.

## Consequences

The observatory now presents a coherent Russian command interface and a more grand-strategy-like information hierarchy. It still does not claim final Hearts of Iron production fidelity: authored polygon geography, icon assets, animated counters, state construction slots, army fronts, shaders, and a dedicated UI skin remain separate production milestones. No simulation-version change is required because the country metrics already belonged to authoritative state and only the read-only snapshot expanded.
