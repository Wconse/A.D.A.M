# ADR 0110: Observed priced firm credit

## Status

Accepted for Stage 0.

## Context

Scheduled firm principal created monthly liquidity pressure, but lending still arrived as an externally specified transfer with no price and no evidence-based limit. Management forecasts also assumed zero financing even when a lender was ready to fund the firm. A useful credit system needs a concrete offer, a lender with real cash, a borrower decision, and a repayment burden derived from observable operations rather than privileged knowledge.

## Decision

A lender may underwrite a two-month firm credit offer after at least three monthly operating observations. The offer records requested and approved principal, annual interest in basis points, term, expiry, observed monthly operating surplus, and observable collateral value.

Underwriting uses only authoritative evidence already available to actors:

- average realized sales and production from bounded firm history;
- observed recipe-input prices;
- active payroll obligations;
- scheduled service on existing firm debt;
- inventory valued at regional prices;
- installed capacity valued as one month of reference-price gross output;
- lender cash.

Input costs and payroll are deducted from observed sales. Existing scheduled debt service is deducted next. At most half of the remaining monthly surplus may support a new loan. The term and interest rate convert that payment capacity into a bounded principal. Secured credit is additionally capped at 70% of observable collateral; unsecured credit relies on cash flow. The approved amount can never exceed the request or the lender's current cash.

An authorized majority owner or chief executive accepts a live offer through the shared command boundary. Acceptance rechecks funding, transfers real lender cash to the firm, creates the ordinary ranked creditor claim, and removes the offer. Active offers become concrete expected financing in firm forecasts; expired offers do not.

Interest accrues monthly on outstanding principal using the contractual annual basis-point rate. Payments are applied to accrued interest first and principal second. Unpaid interest persists, maturity does not forgive either balance, insolvency freezes ordinary service, and liquidation settles principal plus accrued interest at the claim's existing priority after worker claims.

## Invariants

- An offer cannot exist without an identified lender, borrower, rate, term, evidence, and expiry.
- Underwriting never creates money or promises more cash than the lender currently owns.
- Fewer than three observations cannot support an offer.
- Payroll, input costs, and existing scheduled debt reduce new borrowing capacity.
- Secured principal cannot exceed 70% of observable collateral.
- Only a live accepted offer moves money or creates a claim.
- Interest is charged only on outstanding principal and paid only with real firm cash.
- Interest is senior to principal within one scheduled payment, while worker payroll remains senior to all debt service.
- Unpaid interest survives maturity and enters the existing liquidation waterfall.
- Offers, rates, accrued interest, commands, events, persistence, and fingerprints are deterministic.

## Consequences

Credit becomes a strategic bridge rather than a free rescue. Strong but temporarily illiquid firms can borrow against demonstrated surplus and assets; weak firms receive smaller offers or none. Higher rates reduce affordable principal and raise monthly cash pressure. Secured lending is cheaper to justify but exposes a larger ranked claim against the estate. The model deliberately omits negotiated covenants, collateral liens by individual asset, refinancing, variable rates, guarantees, credit bureaus, and institutional bank balance sheets until those systems close additional causal loops.
