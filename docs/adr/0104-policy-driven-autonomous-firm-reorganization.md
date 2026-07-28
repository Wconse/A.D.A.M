# ADR 0104: Policy-driven autonomous firm reorganization

## Status

Accepted.

## Context

Funded reorganization exposed a valid administrator command, but insolvent firms remained frozen unless an external caller assembled the plan. The autonomous economy needs owners to make the same recovery decision from authoritative evidence without receiving free capital or bypassing worker claims.

## Decision

Each monthly economic cycle considers insolvent firms in stable identifier order after payroll and distress response, before household cashflow and commerce.

For each firm, the administrator derives a minimum viable one-batch reopening plan. The plan requires installed capacity, enough available workers from existing employment agreements, and an economic owner. The owner with the greatest economic rights is selected, with the lowest actor identifier breaking ties.

The owner participates only when the firm's authorized reinvestment policy is positive. The contribution is exactly the shortfall between estate cash and the sum of all preserved wage claims plus one month of minimum viable payroll. It must fit both the owner's liquid cash and the reinvestment share of that cash. Staffing is assembled canonically from available cohorts and the production target is one batch.

The derived proposal is committed through the existing funded-reorganization transition, so administrator authority, sponsor ownership, claim settlement, payroll reserve, employment restoration, target validation, events, and atomicity are not duplicated.

## Consequences

- Insolvency can recover endogenously when real owner liquidity and an authorized appetite for reinvestment exist.
- High reinvestment policy creates a meaningful resilience-versus-liquidity tradeoff; cautious owners preserve personal cash but may leave productive assets frozen.
- Worker claims retain absolute priority and no debt is forgiven or money created.
- The minimum viable plan limits speculative reopening and lets later market evidence determine expansion.
- Firms without affordable plans remain in administration. Creditor priority, ownership dilution, asset sales, and liquidation remain later gates.
