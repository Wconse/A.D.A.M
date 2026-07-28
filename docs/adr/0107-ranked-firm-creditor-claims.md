# ADR 0107: Ranked firm creditor claims

## Status

Accepted for Stage 0.

## Context

Firm liquidation already preserves worker claims, sells usable inventory to solvent local producers, and distributes residual value to owners. Ordinary lenders were absent, so owner value could be returned while a real creditor had no authoritative claim on the estate. Adding an abstract debt counter would not be enough: credit must move existing money, survive save and replay, and change the liquidation waterfall.

## Decision

An operating firm may receive a positive loan from a registered actor through the shared command boundary. Issuance atomically debits creditor cash, credits firm cash, and creates one outstanding principal claim for that creditor, firm, and priority.

Stage 0 supports two contractual ranks:

1. secured creditor principal;
2. unsecured creditor principal.

Liquidation keeps worker wage claims senior to both classes. Remaining estate cash pays secured claims pro rata, then unsecured claims pro rata, using deterministic largest-remainder allocation with canonical key tie-breaking. Owners receive only the final residual. Every paid and written-off creditor claim is journaled, and settled claims are removed from authoritative state.

Firms in insolvency cannot take new credit. Reorganization does not erase creditor principal; a reopened firm continues to owe its existing claims.

## Invariants

- Loan issuance conserves money between the actor and firm.
- Principal must be positive and fully funded by creditor cash.
- Worker wage claims remain senior to all financial creditors.
- Secured claims are paid before unsecured claims.
- Equal-rank shortages are allocated proportionally and deterministically.
- Owners cannot receive liquidation cash while a senior claim can still be paid.
- Liquidation explicitly records both creditor recovery and write-off.
- Commands, persistence, equality, and the stable fingerprint include creditor claims.

## Consequences

Credit now creates a meaningful risk tradeoff: firms gain liquidity before distress, while lenders can lose principal if worker claims and estate value exhaust the recovery pool. The model remains intentionally narrow. It has no interest accrual, maturity, collateral asset matching, covenant, tradable debt, institutional bank balance sheet, or post-reorganization repayment schedule. Those belong in later finance slices after creditor principal has demonstrated useful causal behavior.
