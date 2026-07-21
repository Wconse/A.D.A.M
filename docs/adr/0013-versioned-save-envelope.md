# ADR 0013: Versioned save envelope and migrations

- Status: Accepted foundation
- Date: 2026-07-21

Every save is wrapped in a versioned envelope containing exact simulation, schema, ordered mod, package-fingerprint, and content-fingerprint metadata. Decoding checks format and compatibility before exposing payload state. Migrations are explicit contiguous version steps; missing, non-advancing, cyclic, and overshooting chains fail rather than guessing.
