# ADR 0109: Scheduled firm debt service

## Status

Accepted for Stage 0.

## Context

Actor-funded firm credit already moved real money and survived into a worker-first liquidation waterfall, but principal had no maturity or operating-life repayment path. A solvent firm could retain borrowed liquidity forever and the creditor could recover only after liquidation. Credit therefore expressed priority risk without creating the monthly liquidity pressure that makes term debt strategically meaningful.

## Decision

Stage 0 adds an optional amortizing schedule to a firm creditor claim. Existing unscheduled claims remain perpetual for compatibility. New scheduled credit accepts a term from 1 through 120 months, moves the same real actor cash into the firm, and makes the first principal installment due one calendar month after issuance.

Scheduled principal is serviced once per economic month after payroll and before distress response. Each due amount is the remaining principal divided by the remaining installments, rounded up deterministically; the final installment therefore absorbs any remainder. The firm pays only from available cash and the creditor receives exactly the amount debited. A partial or missed payment reduces neither the unpaid balance nor its liquidation priority beyond the amount actually transferred.

Each elapsed due date consumes one contractual installment. When no installments remain, the entire outstanding balance is overdue and remains due every later operating month. Insolvent firms stop ordinary scheduled service: the unchanged claim enters reorganization or the existing worker-first liquidation waterfall. Complete repayment removes the claim.

Every attempt emits `FirmDebtServiceSettled`, including due, paid, remaining principal, and overdue status. Schedule state is serialized and fingerprinted, and issuance and monthly service are available through the shared command boundary.

## Invariants

- Issuance and repayment conserve money between the firm and creditor.
- Payroll is senior to scheduled principal in the monthly sequence.
- A payment cannot exceed firm cash, the amount due, or remaining principal.
- Only actual payment reduces authoritative principal.
- Amortization is deterministic for principal values not divisible by the term.
- Complete repayment removes the creditor claim.
- Missed principal survives maturity and becomes explicitly overdue.
- Insolvency freezes ordinary debt service without erasing or subordinating the claim.
- Liquidation still pays workers, secured creditors, unsecured creditors, then owners.
- Commands, events, persistence, replay, and stable fingerprints include schedule state.

## Consequences

Firm credit now trades immediate liquidity for future monthly cash pressure. Payroll can crowd out debt service, missed payments can remain visible before insolvency, and lenders can recover through either ordinary repayment or the existing estate waterfall. The model remains intentionally narrow: there is no interest, origination fee, grace period, acceleration covenant before maturity, collateral revaluation, refinancing, loan trading, or institutional bank balance sheet. Those mechanisms should be added only when they close a distinct causal loop rather than decorating the claim.
