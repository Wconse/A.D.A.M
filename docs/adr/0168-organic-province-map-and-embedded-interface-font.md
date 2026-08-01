# ADR 0168: Organic province map and executable-embedded interface font

## Status

Accepted. Supersedes the map geometry of ADR 0163 and the font loading mechanism of ADR 0167.

## Context

Two defects blocked the visual pass.

First, no interface text rendered at all. ADR 0167 bundled Noto Sans and loaded it with `asset_server.load("fonts/NotoSans-Regular.ttf")`. Bevy resolves asset paths relative to `CARGO_MANIFEST_DIR`, which for this binary is `apps/adam-observatory`, and the font lives in the workspace-root `assets/` directory. The handle never resolved, and the font-application system then assigned that broken handle to every `TextFont` in the interface, so all text disappeared. The strings themselves were always valid UTF-8; the earlier "mojibake" reading was a console artifact.

Second, the province blockout from ADR 0163 was a hand-placed rectangle grid. It read as a spreadsheet rather than as a strategic map, and it could not express coastlines, contiguous regions, or terrain.

The visual reference target is HOI4 / Age of History 3 **for the map only**. The interface will be redesigned separately around the breadth of simulation mechanics, so no reference styling is applied to panels here.

## Decision

Embed the font in the executable with `include_bytes!`, parse it via `Font::try_from_bytes`, insert the handle as a resource in `PreStartup`, and apply it in `Update` filtered by `Added<TextFont>` so late-spawned text such as map tooltips is also covered.

Generate the map in a dedicated `map` module as a deterministic seeded lattice rather than authored geometry:

- A flat-top hex lattice whose corners are perturbed by a value-noise jitter, with corner positions quantized into shared integer keys so adjacent provinces reuse identical vertices and never leave seams.
- A value-noise landmass mask with a sea margin, producing an irregular continent and coastline.
- Two-level partitioning: farthest-point seeds followed by balanced multi-source BFS, first into countries and then into regions, so every authoritative region owns one contiguous province area.
- Per-cell relief classes (plain, forest, hills, mountains) that shade province fill and place terrain marks.
- Borders classified as province, coast, region, or country, each with its own width, color, and depth, giving the layered outline hierarchy the reference maps use.

Provinces render as triangulated `Mesh2d` polygons with one `ColorMaterial` each, so overlay recoloring mutates materials instead of rebuilding geometry. Region and country ownership always derives from the authoritative snapshot; the map module only decides shape, never who owns what.

## Invariants

1. Cyrillic rendering depends on neither an external `assets` directory nor system fonts.
2. Text spawned after startup receives the bundled face.
3. Map generation is deterministic for a given seed and ownership vector.
4. Adjacent provinces share exact border endpoints.
5. Every authoritative region owns a non-empty contiguous province area.
6. Country borders only ever separate provinces of different countries.
7. Province hit-testing at a province center resolves to that province.
8. Map geometry never invents ownership, population, or economic values.

## Consequences

The map now reads as an organic strategic theatre with coastlines, terrain, layered borders, capitals, and zoom-dependent labels, while remaining a pure projection of authoritative state. Interface panels are deliberately left as-is and will be replaced wholesale in a later mechanics-driven pass. Future map work — supply lines, ports, weather, and data-driven map modes — can attach to the existing per-cell and per-border structures. No simulation-version change is required.
