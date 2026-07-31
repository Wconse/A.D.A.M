# ADR 0132: Solvency-gated employment retention

## Status
Accepted.

## Decision
An incumbent may match a materially better competing wage only when it has a forecast of at least three months, no wage arrears, no insolvency, and forecast cash after inputs and financing covers the complete retained payroll over that horizon. Retention uses a replayable command and typed event; it changes the agreement wage but does not prepay wages or consume the competitor's cash.

## Consequences
Profitable firms can retain established workers, while distressed firms cannot promise wages they are unlikely to pay. Equal offers favor employment stability. Counteroffers remain limited to matching rather than bidding wars.
