# ADR 0115: Firm borrower credit history

## Status

Accepted for Stage 0.

## Context

Lenders remembered realized portfolio outcomes, but borrowers had no durable credit reputation. A firm that repeatedly paid late could return to the market at the same rate as a firm with punctual service once current distress cleared. Conversely, a completed loan created no reputational benefit for the borrower. Current cash, collateral, and distress are necessary underwriting facts, but they are not a substitute for demonstrated conduct.

## Decision

Each firm may now accumulate a `BorrowerCreditHistory` containing:

- total scheduled debt service due and paid;
- count of fully paid service attempts;
- count of partial or missed service attempts;
- successfully resolved loans;
- defaulted loans.

Every scheduled service attempt updates the history after real cash settlement. A payment is punctual only when it covers the complete amount due in that attempt; any shortfall is delinquent. Full contractual resolution records one success. Terminal liquidation records one resolution per creditor claim and classifies any claim with a write-off as a default. Liquidation does not masquerade as an on-time scheduled payment.

`BorrowerCreditHistoryUpdated` journals each update. Cumulative history is serialized, exposed for inspection, replayed through the existing debt-service and liquidation commands, and included in the stable fingerprint.

Future autonomous pricing adds a bounded borrower-specific adjustment:

- historical payment shortfall ratio adds up to 2,000 basis points;
- each delinquent service attempt adds 100 basis points, capped at 1,500;
- each default adds 1,000 basis points, capped at 3,000;
- each successfully resolved loan subtracts 50 basis points, capped at 250.

This adjustment is added after current borrower distress and lender concentration/loss experience. All final rates remain within the existing basis-point bounds. Underwriting cash-flow, collateral, lender liquidity, portfolio concentration, and minimum-gap-coverage constraints remain independent hard gates.

## Invariants

- History changes only after authoritative debt service or claim resolution.
- Due and paid totals reflect scheduled attempts; liquidation classification does not inflate punctual-payment totals.
- A partial payment never counts as punctual.
- A resolved claim is classified once as success or default.
- Reputation changes price only; it never creates cash, collateral, approval, or forgiveness.
- Positive history grants only a small discount, while poor history remains bounded and cannot overflow the rate type.
- Save/load, command replay, canonical ordering, and stable fingerprints include borrower history.

## Consequences

A borrower that pays 50 of 100 due moves from 600 to 1,700 basis points under otherwise identical conditions. Paying the next 50-unit obligation in full and resolving the loan partially rehabilitates the rate to 1,316 basis points; a later default raises it to 2,316. The regression also proves exact history counters and fingerprint participation.

The controlled demo borrower repays its loan successfully, so its positive history becomes durable world state even though it does not seek a second loan in the current 50-year scenario. `SIMULATION_VERSION` advances from 53 to 54 and the seed-1 fingerprint becomes `11122244119561874726`.

The model does not yet distinguish restructurings, covenant breaches, collateral recovery quality, or time-decay of old incidents. The next gate should add loan-purpose and working-capital outcome attribution: lenders should observe whether borrowed cash preserved payroll and production rather than judging credit only by repayment.
