# ADR 0163: Strategic province-map blockout

## Status

Accepted as a visual prototype.

## Context

Rectangular region cards communicated data but did not read as geography. The first strategic-map pass needs to establish the visual grammar of a grand-strategy client before committing to authored coastline assets or a full map editor.

## Decision

Render each canonical simulation region as a deterministic cluster of adjoining province tiles. Shared edges form a continuous two-country landmass while small edge tiles break the rectangular silhouette into a rough coastline. Each province has a dark border layer, overlay-driven fill, selected gold border, compact geographic label, and country-scale translucent label.

Add a subdued ocean grid, landmass shadow, and a visible Arcadia-Borealia route. These objects are presentation-only and contain no movement, ownership, adjacency, or logistics rules. Region selection and map overlays continue to derive from the immutable `ObservatorySnapshot`.

## Invariants

1. Canonical region order determines deterministic province placement.
2. Visual province geometry does not create simulation adjacency.
3. Overlay colors remain derived from authoritative regional values.
4. Selection changes border emphasis without mutating the world.
5. `adam-core` remains independent of Bevy and map artwork.

## Consequences

The observatory now reads as an early grand-strategy map instead of a grid of dashboards. The blockout intentionally uses procedural tile clusters. A future map-production step can replace these with authored polygon meshes, coastlines, rivers, terrain, zoom levels, and mouse picking while preserving the same snapshot boundary. No simulation-version change is required.
