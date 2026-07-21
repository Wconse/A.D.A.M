# ADR 0002: Canonical numeric values and identity width

- Status: Accepted
- Date: 2026-07-21

## Context

The simulation will contain many actors and many more relationships. Identity width and numeric semantics affect memory, cache density, save compatibility, determinism, and every future system. Floating-point values and oversized identifiers would create avoidable ambiguity or bandwidth cost.

## Decision

- Entity identifiers use typed `u32` wrappers. Zero is reserved by content validation. A single identity domain can address more than four billion records while keeping graph endpoints compact.
- Population uses an unsigned 64-bit integer count.
- Money uses signed 64-bit minor units. Negative balances and debt remain representable without floating point.
- Normalized shares and influence weights use unsigned basis points (`0..=10,000`).
- Arithmetic that may overflow is explicitly checked.
- Canonical fingerprints serialize fixed-width integers in little-endian field order.
- Ordered maps remain the correctness-first canonical storage for Stage 0. Dense slot maps or structure-of-arrays storage will replace hot paths only after representative profiling.

## Consequences

- An influence edge carries two 32-bit IDs and a 16-bit weight before container overhead.
- Saves must version any later change in ID width or monetary scale.
- Exchange rates, inflation indexes, probabilities, and fractional production will require named fixed-point types rather than raw integers.
- `BTreeMap` is not claimed to be the final high-performance store; it provides deterministic iteration and clear invariants while the model is still changing.
