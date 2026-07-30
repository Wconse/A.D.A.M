# ADR 0111: Autonomous firm credit market

## Status

Accepted for Stage 0.

## Context

Priced credit offers had evidence-based underwriting, real interest, and authoritative borrower acceptance, but offers appeared only through an external command. Viable firms could become temporarily unable to fund one operating month while domestic actors held idle cash, yet neither side searched for a transaction. Automatically rescuing every shortfall would be equally wrong: credit must remain selective, priced, concentrated, and capable of refusal.

## Decision

After monthly commerce and observed production management, an autonomous credit-market stage reviews operating firms. It runs before the next month, so accepted working capital can fund the next payroll and input purchases.

A borrower enters the market only when all of the following are true:

- it is operating rather than insolvent;
- it has at least three monthly operating observations;
- it has no outstanding creditor claim;
- an authorized majority owner or chief executive can accept financing;
- average observed sales exceed the next operating month's active payroll, preserved wage arrears, and target-based input cost;
- current firm cash cannot cover that same obligation.

The resulting request equals the uncovered one-month funding gap. It is not a generic cash target and does not include speculative expansion.

Eligible lenders are domestic actors who do not control the borrower. Every lender keeps 50% of current cash as a liquidity reserve. Total firm-credit exposure cannot exceed 40% of current liquid cash plus outstanding claims. Existing exposure raises the offered annual rate; each observed month of borrower payroll distress adds a further risk premium. Stage 0 autonomous offers are secured and retain the existing collateral and cash-flow underwriting limits.

All eligible lenders may underwrite competing offers through the ordinary offer command. The borrower accepts the lowest annual rate that covers at least 25% of the funding gap, preferring larger principal and then lower actor identity when prices tie. Rejected offers for that completed search are removed. Acceptance uses the same corporate command as a player, moves real lender cash, and creates the same interest-bearing ranked claim.

The entire market stage is atomic, executes at most once per simulation month, is included in monthly economic cycles, and emits a typed completion event. Accepted credit is projected into the chronicle.

## Invariants

- Autonomous credit funds a concrete one-month operating gap, not growth by decree.
- A borrower with non-positive observed operating surplus receives no autonomous offer.
- Insolvent firms and firms with existing creditor claims do not borrow again autonomously.
- Lenders cannot lend across countries in this Stage 0 market.
- Owners cannot disguise recapitalization as autonomous third-party credit to their own firm.
- A lender retains at least half of current liquid cash.
- Total firm-credit exposure is capped at 40% of lender financial wealth.
- Distress and concentration cannot reduce the interest rate.
- A borrower may accept at most one competing offer per monthly search.
- All offers and acceptances cross the existing player/AI command boundary.
- Duplicate monthly execution is rejected without mutation.
- Canonical ordering and explicit tie-breaking preserve replay determinism.

## Consequences

Credit can now prevent a temporary liquidity failure when a materially viable firm and a sufficiently liquid domestic lender can agree. It cannot sustain a structurally loss-making firm, bypass worker priority, or pull unlimited money from wealthy actors. The model creates meaningful future failure modes: lenders may become concentrated, borrowers may reject inadequate offers, and a previously viable firm can still default when sales deteriorate after borrowing.

The current gate deliberately omits refinancing, multiple simultaneous loans, unsecured autonomous offers, cross-border capital, lender expectations, endogenous rate negotiation, covenant breaches, and portfolio loss memory. These belong in later slices only when they create distinct choices and feedback loops.
