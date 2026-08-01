# ADR 0167: Bundled Cyrillic interface font

## Status

Accepted.

## Context

Bevy's embedded default font does not contain the Cyrillic glyph set used by the Russian observatory interface. The UI strings were valid UTF-8, but the renderer displayed missing-glyph squares. Depending on a system-installed font would make rendering machine-dependent and would not travel with builds.

## Decision

Bundle the OFL-licensed Noto Sans variable font under `assets/fonts/`. During startup, run a font-application system after all interface entities are spawned and assign the bundled handle to every `TextFont`, covering both UI `Text` and world-space `Text2d`.

Keep the license and provenance beside the asset. Treat failure to load the font as a release smoke-test failure by scanning captured runtime logs for asset-loader errors.

## Invariants

1. Russian glyph rendering does not depend on Windows fonts.
2. Every startup text entity receives the same Cyrillic-capable family.
3. Font application runs after interface construction.
4. The bundled font license and provenance ship with the asset.
5. Runtime smoke verification checks both process survival and asset-load errors.

## Consequences

The Russian interface now renders real Cyrillic glyphs rather than squares on clean machines and packaged builds. Typography weights and specialized display faces remain future visual work. No simulation-version change is required.
