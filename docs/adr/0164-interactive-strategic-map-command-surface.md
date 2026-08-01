# ADR 0164: Interactive strategic-map command surface

## Status

Accepted as the second observatory design pass.

## Context

The province-map blockout established geography but still behaved like a static visualization. A grand-strategy client needs a strong default political view, direct map interaction, navigable geography, immediate inspection, and a command hierarchy that does not conflict with simulation hotkeys.

## Decision

Make political ownership the default map mode. Arcadia and Borealia receive stable country palettes while confidence, population, and production remain alternate authoritative overlays. Add deterministic province hit testing over the same visual tile geometry, mouse hover emphasis, click selection, and a cursor-following regional tooltip.

Give the map camera independent I/J/K/L panning, U/O zoom, and R reset controls. These bindings intentionally avoid T/D/E/C government-program commands and Space/Up/Down/H time controls. Camera state is presentation-only.

Recompose the HUD into:

- an A.D.A.M strategic-command masthead;
- a right-aligned reactive date, hour, speed, and run state;
- a dedicated map-mode and legend rail;
- an inspector surface;
- a government-program command surface;
- a political timeline surface;
- map-native country, province, city/capital, terrain, and route marks.

## Invariants

1. Hover, selection, pan, and zoom never mutate `adam-core`.
2. Click selection resolves against deterministic province presentation geometry.
3. Political colors derive from authoritative country identity; quantitative overlays derive from authoritative regional values.
4. Camera bindings do not trigger government-program decisions.
5. UI occlusion prevents map selection through command panels.
6. Region ordering and map hit testing are deterministic.

## Consequences

The observatory now behaves like an early grand-strategy command client rather than a passive dashboard. The retained tile-cluster coast is still a blockout; authored meshes, true shared polygon edges, scroll-wheel zoom, drag panning, map-mode buttons, and zoom-dependent labels remain future production work. No simulation-version change is required.
