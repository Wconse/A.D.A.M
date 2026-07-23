# ADR 0067: Atomic monthly commercial cycle

- Status: Accepted foundation
- Date: 2026-07-23

The simulation now exposes a replayable `ExecuteMonthlyCommercialCycle` command that deterministically executes already-authorized commercial decisions in one explicit sequence: physical production, seller offer formation, household demand planning, market clearing, settlement, firm observation capture, and monthly firm-account reset.

The cycle derives household orders from desired quantities and reserved budgets. This preserves unmet physical needs when income or offer prices make part of the desired quantity unaffordable. Seller offers remain policy-derived, and production remains constrained by the authorized target, labor, capacity, and inputs.

Execution is atomic. The cycle is evaluated on a cloned authoritative world and committed only after every planning, settlement, observation, and accounting step succeeds. Missing prices, insufficient household cash, invalid clearing, or arithmetic overflow leave the original world and event archive unchanged.

At most one commercial cycle may be executed for a simulation date. The last completed date is authoritative save state and participates in stable fingerprints. Successful execution appends the existing detailed production, trade, and observation events plus a typed cycle summary. Management targets and firm policies remain separate actor decisions; the cycle only executes them.
