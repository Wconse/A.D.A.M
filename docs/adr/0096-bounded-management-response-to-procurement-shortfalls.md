# ADR 0096: Bounded Management Response to Procurement Shortfalls

## Status

Accepted

## Context

Observed firm management derives a production target from market history and
the inputs currently held after monthly procurement. A partial input delivery
can make the physically feasible batch count zero even when the firm still has
sales demand and a temporary route or supplier disruption is the only cause.

If management copies that zero into the authoritative target, next month's
procurement planner sees a zero production requirement and places no input
order. The firm then cannot accumulate the missing input and becomes trapped at
zero after a temporary shortage.

## Decision

When the current month's event journal contains either
`FirmProcurementShortfall` or `FirmProcurementRouteCapacityShortfall` for a
managed firm with a positive existing target, bound the next target as follows:

- no expansion above the existing target;
- contraction by at most one batch in the month;
- a minimum target of one batch.

The ordinary observed-production proposal remains the starting advice. The
bound applies only during a current procurement shortfall and only to a firm
whose existing target is already positive. A deliberately zero target remains
zero unless ordinary market evidence later supports another decision; generic
no-sales and unsold-output contraction without an input shortfall keeps its
previous behavior.

The decision reads typed events already emitted during the same commercial
cycle. It adds no duplicate shortage state or new persistence field.

## Consequences

- Temporary input scarcity cannot extinguish the procurement order needed for
  recovery.
- Repeated shortages can still contract a large target gradually by one batch
  per month, while forbidding expansion during the disruption.
- A one-batch Bakery receiving two successive 500-unit Grain deliveries keeps
  target one, accumulates the full 1,000-unit input, and produces again in the
  third isolated commercial month.
- The rule changes authoritative production-target history, so
  `SIMULATION_VERSION` advances from 34 to 35.
- The full workspace gate passes 124 tests. The seed-1 50-year release
  fingerprint is `12134027468760220507`; runtime is 35.9 ms/year.
