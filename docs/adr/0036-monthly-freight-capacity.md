# ADR 0036: Monthly freight capacity periods

- Status: Accepted foundation
- Date: 2026-07-22

Route capacity is renewable flow, not a permanent stock. A monthly ledger isolates contract and spot reservations by positive simulation month, releases against the original period, and permits deterministic pruning of closed periods. Authoritative shipments still use the legacy non-period ledger until departure-month and arrival-period semantics are introduced explicitly.
