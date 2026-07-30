# ADR 0117: Observed government reserve procurement

## Status

Accepted for Stage 0.

## Context

ADR 0116 created real regional government reserves and an authorized procurement command, but reserves could be filled only by an external decision. The simulation therefore had a physical response instrument without an institutional observation-and-action loop. A government following `ReserveRelease` should be able to react to an observed survival shortage while remaining constrained by actual treasury cash, local producer inventory, authority, and prices.

## Decision

The monthly commercial cycle now runs an observed reserve-procurement stage after household market settlement and before reserve distribution, grievance attribution, social stress, and health. It aggregates residual survival unmet demand by `(region, good)` and treats that quantity as the maximum one-month coverage requirement. Existing reserve stock is deducted before any purchase.

Only countries whose physical-shortage policy is `ReserveRelease` participate. The canonical current political-office holder acts through the existing `ProcureGovernmentReserve` command. Eligible sellers must:

- be operating and located in the affected region;
- produce the required good through their registered recipe;
- hold real post-market inventory;
- sell at the region's observable reference price.

Purchases proceed in stable firm-id order and are bounded independently by residual need, seller inventory, and treasury affordability. Each accepted purchase therefore uses the same atomic cash-and-stock transition available to player commands. Purchased goods enter the regional reserve and may be distributed against the same month's residual survival need immediately afterward.

The stage executes at most once per simulation date, is exposed through `ExecuteObservedGovernmentReserveProcurement`, and emits `ObservedGovernmentReserveProcurementCompleted` even when policy, stock, or funds produce zero purchases. Its completion date is serialized and fingerprinted with the ordinary monthly-stage dates.

## Invariants

- Observation never creates demand, money, stock, authority, or a seller.
- Procurement cannot exceed observed survival unmet demand after existing reserve coverage.
- Only firms producing the good can supply the automatic response; input inventories cannot be stripped accidentally.
- Purchases are local in this slice, so no free government transport is invented.
- Payroll, firm procurement, and household market settlement happen before the state can buy remaining stock.
- Every accepted purchase conserves treasury/firm money and firm/reserve inventory through the ADR 0116 command.
- Failure of any accepted purchase leaves the whole observed stage unchanged.
- Canonical region, good, and firm ordering makes command replay bit-identical.

## Consequences

A region with 1,000 milli-units of observed survival shortage, 1,000 milli-units of retained producer stock, a unit price of 10, and 100 treasury units now buys exactly the missing quantity for 10. Treasury falls to 90, the producer receives 10, and the public reserve receives the physical unit. Command replay produces identical state and a second execution in the same month is rejected atomically.

This is a real institutional response rather than a scripted bailout: without the policy, office holder, local stock, price, or cash, the shortage remains. Because procurement occurs before reserve release, available retained stock can avert same-month physical harm; because it occurs after ordinary commerce, the state cannot pre-empt household purchases based on information it has not yet observed.

`SIMULATION_VERSION` advances from 55 to 56 because monthly stage state and the automatic resource transition are authoritative. The full workspace gate passes 153 tests; two seed-1 timelines produce identical 50-year chronicles and fingerprint `11094233980192924180` at 88.3 ms/year. Deliberate limits: the coverage target is one observed month; procurement uses local reference prices; there is no interregional public shipment, supplier bidding, storage cost, spoilage, corruption, or budget competition. The next gate should introduce reserve coverage targets and inventory carrying costs before expanding procurement across regions.
