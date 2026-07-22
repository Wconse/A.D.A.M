# ADR 0035: Authoritative contract and spot capacity settlement

- Status: Accepted foundation
- Date: 2026-07-22

Shipment start selects an active contract independently for each route leg. A cloned freight ledger atomically reserves either that contract's guaranteed pool or the route's residual spot pool before authoritative state changes. Shipment state records the selected pool per leg; delivery releases exactly those pools. Contract and spot usage are saved, replayed, and fingerprinted.
