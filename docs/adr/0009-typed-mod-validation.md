# ADR 0009: Typed post-merge validation

- Status: Accepted foundation
- Date: 2026-07-21

After all mod layers and patches are applied, registry entries are decoded into strict typed schemas and then checked by domain validators. Diagnostics retain the namespaced key, final source mod, full modification history, and error text. World construction cannot begin with any issue. This separates generic patch mechanics from schema-specific invariants.
