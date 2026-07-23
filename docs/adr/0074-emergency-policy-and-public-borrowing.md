# ADR 0074: Emergency relief is an explicit policy with bounded public borrowing

- Status: Accepted foundation
- Date: 2026-07-23

Each country now retains an authoritative emergency-response strategy selected by a current political-office holder through a replayable command. The initial Stage 0 strategies are `TreasuryOnly`, `BorrowWithinDebtLimit`, and `Inaction`. Treasury-only response preserves the previous behavior; inaction deliberately leaves observed affordability exposure unfunded; borrowing response may issue public debt when current treasury cash is insufficient.

Emergency borrowing is limited to twice the country's current aggregate annual regional output minus existing public debt. Issuance increases public debt and treasury by the same amount, after which the ordinary relief command debits treasury and credits the affected cohort. The monthly response remains atomic and allocates debt headroom across exposed cohorts in canonical order. Policy, debt, treasury, transfers, and events all participate in save/replay and stable fingerprints.

Tests prove qualitatively different outcomes from the same material crisis. Under borrowing policy, treasury 5 plus newly issued debt 5 funds a survival gap of 10, leaving treasury zero, public debt 5, and the cohort funded for next month. Under inaction, treasury remains 100, the cohort receives nothing, and survival fulfillment remains zero. This is the first explicit political branch where policy changes history rather than merely changing a displayed indicator.

The debt-to-output ceiling and automatic adherence to stored policy are Stage 0 approximations. There are not yet bond buyers, interest rates, currency issuance, legislative vetoes, administrative capacity, electoral incentives, coalition conflict, or default risk. Physical shortages still require procurement, rationing, imports, or production support rather than monetary relief.
