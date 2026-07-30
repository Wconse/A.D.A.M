# ADR 0118: Reserve coverage, budget, and chronicle evidence

## Status

Accepted for Stage 0.

## Context

Observed government procurement could close one currently visible survival shortage, but it had no explicit policy depth: every active government pursued exactly one month of coverage and could spend the entire treasury independently on each good. The event journal also recorded successful purchases and releases without preserving why no purchase occurred or whether the binding obstacle was local supply or the authorized budget. The chronicle therefore described outcomes but could not explain the state's material decision.

## Decision

`GovernmentEmergencyPolicy` now carries two authoritative reserve controls:

- a coverage target of one to twelve observed shortage-months;
- a monthly automatic procurement ceiling of zero to 10,000 basis points of opening treasury.

The ceiling is shared across all reserve goods and regions of one country in canonical requirement order. Existing reserve stock counts toward the target. Procurement buys only the remaining gap and remains bounded by eligible local producer inventory, the current treasury, the shared policy ceiling, and the regional reference price. Bought stock may satisfy the current shortage and leave the rest physically stored for a later month.

Every active regional-good review emits typed evidence containing the observed shortage, target stock, opening reserve, eligible local supply, available budget, purchased quantity, spending, remaining gap, and independent supply/budget limitation flags. A zero-purchase review is evidence rather than silence. The yearly chronicle aggregates this evidence and explicitly connects post-market shortages, attributed public purchases, in-kind releases, unresolved coverage gaps, and downstream grievance/stress/health ordering.

## Invariants

- A coverage target never creates demand, inventory, money, prices, sellers, or authority.
- Procurement cannot exceed target gap, eligible supply, treasury cash, or the country's remaining monthly ceiling.
- The country ceiling is consumed once across competing goods; it is not reset per region or good.
- Existing reserve stock reduces the quantity bought and survives the current release if coverage exceeds current need.
- Supply and budget limitations are reported independently, so both may bind the same review.
- Requirement ordering, seller ordering, policy state, evidence, replay, serialization, and fingerprints are deterministic.
- Rejected commands and failed monthly stages remain atomic.

## Consequences

A two-month policy facing one month of observed shortage buys two physical units when stock, treasury, and budget permit, releases one immediately, and retains one for the next emergency. A 10% ceiling on a treasury of 100 authorizes only 10 minor units across all competing goods, so the first canonical requirement may exhaust the common envelope and later requirements are explicitly recorded as budget-limited. Removing producer stock records a supply-limited zero purchase without inventing an alternative supplier.

`SIMULATION_VERSION` advances from 56 to 57 because coverage and budget controls are authoritative fingerprinted policy. The focused gate covers retained future stock, shared-country budget competition, exact supply/budget constraint evidence, attributed chronicle prose, replay, and atomic rejection. The full workspace passes 157 tests; two seed-1 50-year timelines produce identical chronicles and fingerprint `17917500942299006191` at 91.2 ms/year.

Deliberate limits: targets extrapolate current observed shortage rather than forecasting seasonality. Reserves still have no carrying cost, physical spoilage, interregional public transport, supplier bidding, corruption, or competing legislative appropriations. The next gate should impose bounded storage loss and treasury carrying cost so a deep buffer creates a real preparedness-versus-fiscal-resilience tradeoff before public logistics expands.
