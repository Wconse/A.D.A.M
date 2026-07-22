# ADR 0046: Deterministic local market clearing

- Status: Accepted foundation
- Date: 2026-07-22

Local markets clear by region and good. Buyer orders are canonical by CohortId; offers are ranked by price and FirmId. Fills respect physical supply and buyer budgets, and unmet demand remains explicit. This is a pure clearing layer; authoritative inventory and cash settlement follows separately.
