# ADR 0072: Supply-aware emergency relief closes the first institutional response loop

- Status: Accepted foundation
- Date: 2026-07-23

The monthly market now distinguishes survival deprivation caused by absent supply from deprivation caused by inability to pay for physically available offers. After clearing, unsold lots are allocated in canonical price/seller order against survival demand that households could not budget. The resulting cohort affordability gap is authoritative observed state.

After health and social consequences are updated, a political-office holder may fund the observed gap from the country's existing treasury. Autonomous Stage 0 response selects the lowest-ID holder of a political office in the affected country, then applies the ordinary replayable `FundEmergencyRelief` command. Transfers debit treasury cash and credit cohort liquid wealth for the next month's market; they cannot exceed the treasury. Unheld offices and empty treasuries produce no invented capacity.

Relief is supply-aware: no transfer occurs when the survival good was unavailable, preventing repeated cash accumulation in a physical shortage. A two-month integration test proves `unaffordable unsold food -> observed gap -> authorized treasury transfer -> funded next-month purchase -> full survival fulfillment`. A separate test proves that absent supply leaves treasury unchanged. Direct monthly execution and command replay remain identical.

This is a minimal institutional response, not a complete welfare state. It assumes political-office holders automatically fund observable affordability gaps and does not yet model policy preferences, eligibility, administrative leakage, procurement, rationing, imports, debt issuance, opposition, blame, protest, or repression. Those become competing decisions after the end-to-end response path exists.
