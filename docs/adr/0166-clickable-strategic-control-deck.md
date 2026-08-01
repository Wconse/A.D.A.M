# ADR 0166: Clickable strategic control deck

## Status

Accepted as the fourth observatory design pass.

## Context

Keyboard shortcuts made the prototype operable but did not communicate a production grand-strategy interaction model. Map layers, time state, and state capacity need visible controls, selected-state feedback, and authoritative visual metrics.

## Decision

Add directly clickable Russian controls for all four map modes, pause, and five simulation speeds. Active controls use a gold command-state treatment, hover uses a blue treatment, and inactive controls remain dark navy. Keyboard commands remain equivalent.

Add a selected-country color flag and three authoritative regional metric bars for confidence, population, and production. Population and output bars normalize against the current canonical snapshot; confidence uses its physical basis-point range. Bars refresh when the authoritative snapshot or selected region changes.

Keep UI interactions presentation-only except for existing time controls: selecting a map mode, hovering, and choosing a region do not mutate `adam-core`; pause and speed govern how frequently the client submits authoritative hourly transitions.

## Invariants

1. Clickable controls and keyboard shortcuts produce the same presentation state.
2. Speed buttons never bypass `World::advance_hour`.
3. Metric bars contain no invented values.
4. Country flag color derives from the selected region's authoritative country.
5. Active, hover, and inactive states are visually distinct.
6. Button bindings do not collide with government-program commands.

## Consequences

The observatory now has a visible command deck instead of a keyboard-reference panel. This moves the interaction language closer to a production grand-strategy interface while preserving the core/client boundary. Final iconography, sounds, animated transitions, accessibility focus navigation, and skin assets remain future visual-production work. No simulation-version change is required.
