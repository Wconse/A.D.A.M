# ADR 0038: Authoritative per-leg shipment lifecycle

- Status: Accepted foundation
- Date: 2026-07-22

Inventory shipments now use per-leg lifecycle state. Starting a shipment reserves its planned route pools atomically; advancing time reports completed legs and immediately releases each leg's general and contract/spot capacity. Final inventory credit occurs only after the last leg. Save, replay, and fingerprint preserve current route and exact remaining route time through serialized shipment state.
