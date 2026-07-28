# ADR 0103: Funded firm reorganization

## Status

Accepted.

## Context

Insolvency freezes a failed firm without destroying assets or worker claims, but a permanent freeze is not a recovery system. Reopening must not let owners evade unpaid wages, underfund payroll, or bypass the same authority boundary used by player and AI management.

## Decision

The appointed administrator may submit a replayable reorganization plan naming an economic-owner sponsor, a cash contribution, canonical cohort staffing, and a positive production target. The transition is atomic.

Reorganization is accepted only when the contribution and estate cash can pay every outstanding wage claim immediately and still retain one full month of payroll for the proposed workforce. Staffing must use existing local agreements, fit firm and cohort limits, and remain within installed production capacity. Direct rehiring and target changes remain blocked while insolvency is active.

On acceptance, sponsor cash enters the firm, all worker claims are paid to household wealth, insolvency is removed, employment and the production target are restored through ordinary transitions, and `FirmReorganized` records financing, claims, staffing, and remaining reserve.

## Consequences

- Recovery competes for real owner liquidity and cannot externalize old wage debt.
- A reopened firm has both workers and a minimum liquidity runway, avoiding immediate mechanical relapse.
- Player and AI use the same serialized command and deterministic validation.
- New creditors, negotiated haircuts, ownership dilution, and asset liquidation remain later gates.
