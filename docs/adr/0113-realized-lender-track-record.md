# ADR 0113: Realized lender track record

## Status

Accepted for Stage 0.

## Context

The autonomous credit market priced current borrower distress and current lender concentration, but a creditor forgot every completed loan and every liquidation loss. Interest already returned as real actor cash, yet no explicit portfolio record distinguished a lender that had repeatedly been repaid from one that had destroyed capital. Consequently future rates and risk appetite could not respond to realized experience.

A useful lending institution needs bounded memory of outcomes without introducing an opaque bank balance sheet, probabilistic scoring model, or money creation. The history must be derived only from authoritative cash settlement and claim resolution.

## Decision

Each creditor may now have a cumulative `LenderCreditHistory` containing:

- principal repaid;
- interest income received;
- realized claim losses;
- successfully resolved loans;
- defaulted loans.

Ordinary scheduled service records principal and interest only when real firm cash reaches the creditor. A claim counts as successful when principal and accrued interest are both cleared. Terminal liquidation splits each creditor recovery interest-first, records the recovered principal and interest, and records every unpaid principal or accrued-interest unit as a realized loss. A liquidation claim with any write-off counts as one default; full recovery counts as one success.

Every update emits `LenderCreditHistoryUpdated` with settlement deltas. Cumulative history is serialized, replayed by the ordinary command transitions, exposed for inspection, and included in the stable fingerprint. The annual chronicle narrates principal recovery, interest income, successful completions, defaults, and realized losses.

Autonomous portfolio appetite adapts deterministically:

- the baseline maximum firm-credit share remains 40% of liquid cash plus live claims;
- each successful loan adds 100 basis points of portfolio allowance, capped at 500 basis points;
- the realized loss ratio is `losses / (principal repaid + losses)`;
- half of that loss ratio reduces portfolio allowance, capped at 2,500 basis points;
- the final allowance is bounded between 15% and 45%;
- the existing 50% current-cash liquidity reserve remains an independent hard cap.

Autonomous pricing also adapts:

- the realized loss ratio adds up to 3,000 basis points;
- each successful loan subtracts 50 basis points, capped at 250 basis points;
- current concentration and borrower distress premiums still apply;
- all rates remain inside the existing basis-point bounds.

## Invariants

- History changes only after real repayment or authoritative claim resolution.
- Principal repayment is not income; principal and interest remain separate.
- Accrued interest recovered in liquidation is income, while its write-off is a realized loss.
- No history update creates or destroys actor or firm cash.
- A loan is classified exactly once, when its claim is resolved.
- Success can expand risk appetite only modestly; liquidity reserve and underwriting remain binding.
- Losses cannot reduce the portfolio limit below 15% or add more than 3,000 basis points to price.
- Canonical ordering, save/load, command replay, and stable fingerprints remain deterministic.

## Consequences

The demo trust finishes 2026 with six units of principal recovered, six units of interest income, one successful loan, and no losses. That record modestly improves future pricing and portfolio allowance. Focused liquidation coverage records separate losses for secured and unsecured creditors, and a synthetic track-record regression proves exact capacity and rate movements after success and default.

The model still does not rank simultaneous borrowers by risk-adjusted return: firms are reviewed in canonical order and can consume shared lender headroom before later firms are considered. The next slice should batch contemporaneous applications and allocate each lender's finite portfolio across competing borrowers before offers are accepted.
