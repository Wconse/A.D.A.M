# ADR 0150: Administratively bounded government-program execution

## Status

Accepted as the first Step 100.3 execution gate.

## Context

Appropriated money must not become public services instantly. Existing regional administration provides the first concrete execution constraint while later Step 100.3 work adds goods, labor, infrastructure, and logistics.

## Decision

- Add a replayable annual execution command after a current-year appropriation.
- Split carryover across target regions by exact charter shares using deterministic largest remainders.
- Regional administrative absorption ranges from 25% at zero administration to 100% at full administration.
- Only absorbed funding becomes delivered funding and improves the selected service priority.
- Unabsorbed commitment remains program carryover and increments persistent delay memory.
- Emit both regional delivery evidence and one aggregate execution event.
- Execution requires real political authority and may occur only once per program-year.

## Invariants

1. Delivered funding never exceeds committed carryover.
2. Execution neither creates treasury cash nor public debt.
3. Regional commitments conserve opening carryover exactly before capacity constraints.
4. Unexecuted money is retained, not silently destroyed or labeled corruption.
5. Only the program's declared service priority improves.
6. Rejected and duplicate execution attempts are atomic.
7. Direct and replayed execution produce identical state and fingerprints.

## Consequences

The promise → appropriation → execution gap is now observable. This is deliberately the administrative foundation of contested execution, not its final physical model: contractor goods, labor, infrastructure, routes, and political friction remain the next gate. `SIMULATION_VERSION` advances 87 -> 88.
