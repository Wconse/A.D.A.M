# ADR 0112: Scenario credit portfolios and default closure

## Status

Accepted for Stage 0.

## Context

The autonomous credit market was mechanically complete but the embedded scenario gave actors no liquid portfolios, so long-history runs never originated a loan. A useful acceptance gate needs both sides of the choice in authoritative history: a viable borrower that receives working capital and a similarly viable borrower that is refused because the lender has exhausted a prudent portfolio limit.

The first scenario experiments also exposed three model defects. Legacy wage arrears were incorrectly treated as recurring operating cost when deciding viability; accepted principal covered the cash gap but not its own first payment; and a zero-workforce firm with matured debt but no wage arrears could remain an operating zombie indefinitely.

## Decision

Content schema v8 adds explicit non-negative `liquid_cash` to owners and optional regional financiers. Financiers are ordinary actors with a home region and real cash, but no implicit ownership or management rights. Initial balances are registered into authoritative world state and participate in events and fingerprints.

The demo adds the Arcadian Working Capital Trust and two deliberately small grain producers. Both producers have identical positive observed operating margins and temporary wage-arrears funding gaps. Canonical firm order presents the first case to the lender first. The trust's fifteen-unit portfolio can prudently fund a six-unit secured loan while retaining its reserve; the resulting exposure exhausts its 40% portfolio allowance, so the second case receives no acceptable offer. The accepted borrower clears arrears, preserves the worker, services six units of principal plus six units of rounded interest, and removes the claim within the first year. The refused borrower subsequently recovers from retained operating surplus without receiving invented money.

To make that decision causally reachable, a distressed firm with two or three observations receives a bounded credit-review grace through the fourth distress month. This is not a general moratorium: firms without emerging underwriting evidence retain the existing three-month response, and an unfunded firm is handled normally once the evidence window closes.

Autonomous funding calculations now separate recurring payroll from legacy wage arrears. Both belong in the cash requirement, but only recurring payroll belongs in observed operating surplus. Every lender-specific request also reserves the estimated first principal installment and first monthly interest charge, subject to all existing underwriting and lender caps. Offers that cannot meet the borrower's minimum coverage are removed immediately rather than remaining in expected financing.

A matured scheduled claim with unpaid principal or interest now counts as distress even when wage arrears are zero. A cashless zero-workforce debtor therefore reaches ordinary insolvency and the worker-first liquidation waterfall instead of accumulating an immortal overdue claim.

## Invariants

- Scenario actor cash is explicit, non-negative, finite, evented, and fingerprinted.
- Financiers receive no ownership or corporate authority merely because they provide capital.
- Wage arrears increase funding need but do not make recurring operations appear structurally unprofitable.
- A new loan reserves its first expected debt-service payment when sizing the request.
- Underwriting, collateral, liquidity-reserve, concentration, and minimum-coverage limits still cap the final principal.
- An inadequate offer is removed in the same atomic market stage and cannot inflate expectations.
- Credit-review grace is bounded to firms approaching the required observation count.
- Matured unpaid debt can trigger insolvency without a wage claim.
- The accepted demo loan moves real cash, fully services, and disappears; the refused case receives no claim.
- Canonical ordering and command replay remain deterministic.

## Consequences

The embedded 50-year history now exercises autonomous acceptance, refusal, repayment, and later market scarcity without materially replacing the existing Northreach decline. The chronicle exposes both successful working-capital provision and viable searches that ended unfunded. Scenario authors can add financiers and tune portfolios directly rather than relying on hidden actor wealth.

The content experiment is intentionally tiny: it validates the causal loop without turning a scripted lender into a macroeconomic bailout. Future work can add lender profit allocation, portfolio loss memory, refinancing, covenants, and cross-border capital only as separate choice-bearing slices.
