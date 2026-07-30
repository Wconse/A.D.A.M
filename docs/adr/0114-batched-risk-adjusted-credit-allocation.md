# ADR 0114: Batched risk-adjusted credit allocation

## Status

Accepted for Stage 0.

## Context

The autonomous credit market previously reviewed firms in canonical id order. That made replay deterministic, but when several viable firms applied to the same finite lender portfolio, the first id could consume the available headroom before later applications were compared. Determinism alone is not an economic allocation rule.

The market needs a contemporaneous decision surface that compares observable return and risk without introducing hidden credit scores, stochastic defaults, or lender money creation.

## Decision

Every monthly credit-market stage now proceeds in three phases:

1. collect every eligible domestic firm application before any lender commits capital;
2. let each lender provisionally underwrite the complete batch, then allocate its finite portfolio headroom in deterministic risk-adjusted-return order;
3. let each borrower accept the cheapest sufficiently large committed offer through the existing authoritative command.

The provisional underwriting pass uses the existing cash-flow, collateral, liquidity, concentration, term, and evidence rules. It does not move cash or create a claim. For each supported application the lender computes a transparent ranking value in basis points:

- contractual annual interest rate;
- minus 400 basis points for each current borrower-distress month;
- plus a collateral cushion only for collateral above principal, scaled at one basis point per 20 excess collateral-ratio basis points and capped at 500 basis points.

The distress charge is deliberately larger than the 250-basis-point contractual distress premium. A high nominal rate therefore cannot automatically make the weakest borrower the preferred use of scarce capital. Over-collateralization helps only modestly. Ties resolve by larger evidence-backed principal and then stable firm id.

Each lender's initial autonomous portfolio capacity is a hard commitment budget across the whole batch. Offers cannot collectively exceed it. Borrower choice remains rate-first, then larger principal, then stable creditor id. Unaccepted competing offers are removed at the end of the stage as before.

The simulation version advances from 52 to 53 because a scenario with simultaneous constrained applications can select a different borrower even though the current demo history remains materially unchanged.

## Invariants

- Application collection is complete before allocation begins.
- No provisional decision moves cash, creates debt, or alters world state.
- A lender's committed principals never exceed its monthly autonomous headroom.
- Ranking uses only authoritative observed state and fixed integer arithmetic.
- Canonical tie-breaking makes replay independent of insertion order.
- Final acceptance uses the ordinary player/AI command boundary.
- Underwriting, liquidity reserve, collateral, cash-flow, and minimum-gap-coverage gates remain binding.
- Failure remains atomic because the market operates on a cloned world until completion.

## Consequences

Scarce credit is now allocated by an explicit economic comparison rather than firm id. A focused regression creates two simultaneous 100-unit funding gaps and a lender with only 120 units of headroom. The earlier-id borrower carries two distress months; the later healthy borrower receives the 110-unit evidence-backed offer, and command replay reproduces the decision and fingerprint exactly.

The current demo still has one accepted and one refused application in 2026, so its chronicle body is unchanged. The version bump alone changes the seed-1 50-year fingerprint to `14936349298780613292`.

The allocation is still a one-pass commitment auction. If borrowers reject offers, unused lender headroom is not re-auctioned within the same month. A later slice may add iterative clearing, but only if a measured scenario proves the extra complexity changes meaningful outcomes. The next higher-value gate is explicit borrower credit reputation: lenders should distinguish punctual service, delinquency, restructuring, and default at the firm level rather than using only current distress and collateral.
