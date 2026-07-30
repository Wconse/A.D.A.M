# ADR 0116: Government physical reserves

## Status

Accepted for Stage 0.

## Context

Emergency policy could transfer cash, borrow, ration scarce supply, or remain inactive, but government could not own a physical good. Cash relief therefore failed honestly during a physical shortage, while the state had no way to prepare inventory before the shortage or release previously acquired stock. The next societal-adaptation arc requires government action to conserve and move the same physical resources used by firms and households.

## Decision

A political-office holder may procure a positive quantity of a good from a domestic firm into a regional government reserve through the shared replayable command boundary. The purchase:

- debits the seller's real inventory;
- debits the country's real treasury;
- credits the seller's real cash;
- credits a canonical reserve keyed by region and good;
- journals the actor, seller, quantity, and settled cost.

The transaction is atomic. Quantity and unit price must be positive, the fixed-point cost must settle above zero, the seller and good must exist, the seller must be domestic to the authorizing office, inventory must be sufficient, and treasury cash must cover the full purchase.

`PhysicalShortageStrategy::ReserveRelease` makes stocked goods available as in-kind emergency aid. After the household market settles, reserves cover residual survival unmet demand in canonical cohort/good order. Release moves physical stock into monthly consumption, reduces unmet demand and deprivation pressure, and is journaled per receiving cohort. It happens before grievance, social stress, and health derivation, so a shortage actually covered by the state cannot still produce mortality or foreign grievance as if no food arrived.

## Invariants

- Procurement conserves money between treasury and the selling firm.
- Procurement conserves physical inventory between the firm and the public reserve.
- Distribution cannot exceed either reserve stock or residual survival need.
- Reserves are regional; this slice introduces no free cross-region government transport.
- Only a current political-office holder may authorize procurement.
- Rejected purchases leave treasury, firm cash, inventory, reserves, events, and fingerprints unchanged.
- Reserve state is serialized, ordered, replayed, inspectable, and fingerprinted.
- Market allocation and proportional rationing retain their existing behavior unless reserve release is selected.

## Consequences

Government can now prepare a real buffer rather than responding only with money after a crisis. The focused gate buys one physical food unit from a firm for 10 minor currency units, moving treasury from 100 to 90 and firm cash from 0 to 10. The reserve then supplies one unit of residual survival need, reducing reserve stock and unmet demand to zero and restoring deprivation pressure to zero. A purchase whose cost exceeds treasury is rejected atomically.

`SIMULATION_VERSION` advances from 54 to 55 because reserve stock and the shortage-response rule are authoritative fingerprinted state. The demo does not yet configure a reserve procurement policy, so this slice establishes the command and causal release boundary without scripting a bailout. The full workspace gate passes 152 tests; two seed-1 timelines produce identical 50-year chronicles and fingerprint `4615256241328481519` at 78.1 ms/year.

Deliberate limits: procurement is currently an explicit decision, purchases are domestic and immediate, reserves do not spoil or incur storage cost, and release is canonical rather than politically targeted. The next gate should derive bounded procurement proposals from observed shortages, treasury capacity, expected monthly need, available domestic stock, and reserve coverage, then execute accepted proposals through this same command.
