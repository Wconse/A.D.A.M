# ADR 0014: Binary authoritative world snapshots

- Status: Accepted foundation
- Date: 2026-07-21

The complete authoritative `World` is serialized with Serde and bincode inside the versioned compatibility envelope. Binary encoding supports typed and tuple map keys that text formats cannot represent safely. Decode validates format and exact mod metadata before returning state. A continued loaded world must be bit-for-bit behaviorally identical to uninterrupted simulation. Human-readable metadata remains separable from the binary payload in the future file container.
