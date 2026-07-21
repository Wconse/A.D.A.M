# ADR 0027: Deterministic multi-leg routing

- Status: Accepted foundation
- Date: 2026-07-21

Shipment planning may traverse several directed routes when no direct service exists. Search considers only routes with sufficient per-leg capacity, avoids region cycles, respects a configurable leg limit and total budget, sums transit time and fixed-point cost, and chooses the cheapest path with lexicographic RouteId tie-breaking. Shared reservation and congestion remain execution-layer concerns.
