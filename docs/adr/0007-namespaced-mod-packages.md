# ADR 0007: Namespaced deterministic mod packages

- Status: Accepted foundation
- Date: 2026-07-21

Mods are folders with strict `mod.toml` manifests. Mod IDs are lowercase dotted namespaces and content IDs use `namespace:local`. Dependencies and load-order hints are resolved deterministically with lexical tie-breaking; duplicates, missing dependencies, self-dependencies, cycles, and unknown manifest fields fail before world construction. Future patching and content hashes build on this boundary. Arbitrary native libraries are excluded; behavioral extensions will use declarative rules and later sandboxed WebAssembly.
