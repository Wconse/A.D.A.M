# ADR 0011: Dependency-ordered disk merge and patches

- Status: Accepted foundation
- Date: 2026-07-21

Validated mod folders are merged in resolved dependency order. Definition files from `goods/` and `recipes/` enter the shared namespaced registry only when their namespace matches the declaring mod. Files under `patches/` contain ordered tagged set/remove/append operations and may target definitions from dependencies. Duplicate definitions and failed paths are errors. The canonical post-merge fingerprint is computed only after every patch succeeds.
