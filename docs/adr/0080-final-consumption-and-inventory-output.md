# ADR 0080: Final consumption and inventory output

## Status

Accepted.

## Context

ADR 0078 measured regional output only from settled household purchases. That removed phantom aggregate growth, but it treated unsold production as if it had never occurred. Once intermediate goods and business-to-business trade are added, summing every sale would also double-count turnover.

Stage 0 needs an expenditure-side identity that already distinguishes final demand from changes in firm inventories.

## Decision

A full economic year captures every firm's opening inventory before the first monthly cycle and compares it with closing inventory after the twelfth cycle. Every inventory quantity is valued at the authoritative regional price for its good.

Regional output is measured as:

```text
final household consumption
+ valued change in all firm inventories
= annual regional output
```

Final consumption is the sum of settled household market spend attributed to seller regions. Inventory change is signed: unsold output increases it, while use of stored intermediate goods decreases it. This identity is equivalent to gross output minus intermediate consumption once all production flows are represented.

A missing regional price makes annual closure fail atomically. Negative measured output is retained in the evidence event but the authoritative regional output is floored at zero so fiscal spending cannot become negative.

Each material annual closure emits `RegionalOutputMeasured` with final consumption, inventory change, and recorded output. The deterministic chronicle narrates the aggregate components.

The legacy aggregate annual harness remains unchanged.

## Consequences

- Unsold production is recognized as inventory investment rather than disappearing.
- Drawdown of intermediate inventories reduces measured output.
- Future B2B purchases can move ownership without being counted as final output merely because a sale occurred.
- Government procurement and net exports are still absent and therefore contribute zero.
- Current fixed regional prices make inventory valuation deterministic; later price formation will require an explicit valuation convention.

## Validation

- The economic-year integration test proves regional output equals final consumption plus inventory change.
- The evidence event and authoritative regional state carry the same output.
- The no-trade/no-production test continues to eliminate phantom output and fiscal revenue.
- Seed-47 one-year and fifty-year CLI runs are audited after recalibrating initial output to the new accounting basis.
