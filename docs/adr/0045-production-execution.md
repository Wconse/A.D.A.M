# ADR 0045: Authoritative monthly production execution

- Status: Accepted foundation
- Date: 2026-07-22

Monthly production is now an authoritative command, not only a planner. The world computes every firm's feasible batches first, then consumes physical recipe inputs and credits output inventory in canonical firm order. Each successful output emits a typed event. Wages, energy bills, depreciation, and sales remain separate settlement layers.
