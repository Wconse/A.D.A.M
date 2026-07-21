# Contributing

A.D.A.M is currently a solo project assisted by code-generating agents. Every change must remain reviewable and reproducible.

## Change protocol

1. Define one narrow outcome and its acceptance test.
2. State affected invariants before implementation.
3. Keep domain rules inside `adam-core`; keep adapters at the edges.
4. Add deterministic tests with the implementation.
5. Run `scripts/check.sh` or `scripts/check.ps1`.
6. Explain intentional history/snapshot changes in an ADR or commit message.

## Commit style

Use Conventional Commits, for example:

- `feat(core): add calendar progression`
- `fix(rng): isolate diplomatic random stream`
- `test(core): lock seed 47 history fingerprint`
- `docs(adr): record archive storage decision`

## Review checklist

- No hidden player-only path.
- No wall-clock or unseeded randomness.
- No reliance on unordered iteration.
- No Bevy dependency in `adam-core`.
- No domain logic in CLI or future UI crates.
- New public behavior has tests and documentation.
