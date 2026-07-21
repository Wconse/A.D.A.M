# ADR 0008: Explicit layered content registry

- Status: Accepted foundation
- Date: 2026-07-21

Mod content is merged into a deterministic namespaced registry. New definitions never silently replace existing keys. Changes to existing definitions require ordered explicit `set`, `remove`, or `append` operations over validated paths. Every resulting entry records the mod that last changed it, enabling conflict reports and content hashing. Schema-specific validation occurs after all layers are applied and before world construction.
