# ADR 0041: Deterministic terminal queues

- Status: Accepted foundation
- Date: 2026-07-22

Terminal admission uses strict FIFO per terminal. A head shipment that does not fit blocks later smaller cargo, preventing hidden priority reordering and making congestion causal. Queue entries are unique by shipment, serializable, and admitted against the shared terminal-capacity ledger. Priority contracts and emergency handling must be explicit future policies rather than incidental iteration order.
