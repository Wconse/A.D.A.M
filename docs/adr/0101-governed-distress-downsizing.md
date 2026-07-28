# ADR 0101: Governed distress downsizing and durable worker claims

## Status

Accepted.

## Context

Owner recapitalization gives a distressed firm a resource-conserving recovery path, but owners may have no liquid cash. Without a second response, the firm continues promising wages it cannot pay and can persist indefinitely with unchanged staffing. Immediate closure would be equally shallow: it would erase the possibility of recovery and risk silently losing worker claims.

## Decision

When a firm reaches the three-month payroll-distress threshold and its largest economic owner cannot contribute cash, an authorized operations actor reduces every active employment agreement by 25%, rounded up to at least one worker per agreement. Operations managers are preferred, followed by chief executives and strict-majority owners, using the existing corporate authority order.

The reduction is bounded and repeatable rather than an instant liquidation. It passes through the ordinary employment transition, so labor supply, production capacity, household employment, and the event journal remain coherent. A dedicated `FirmDownsizedForDistress` event records the responsible actor, workers before and after, and preserved arrears.

Inactive employment agreements with arrears remain payable. Monthly payroll now processes active agreements or any inactive agreement that still carries a worker claim. No new wages accrue after employment reaches zero, but later recapitalization or operating cash can settle the old debt through the same payroll transfer as an active worker.

## Consequences

- Cashless firms trade productive capacity for a lower future wage bill instead of receiving synthetic rescue.
- Downsizing creates visible household unemployment and production consequences through existing systems.
- Worker claims survive termination and can be paid after a late rescue.
- Canonical agreement order, fixed reduction, and existing authority selection preserve deterministic replay.
- This remains restructuring, not insolvency. A zero-workforce firm can still own inventories and assets; explicit closure, asset disposition, creditor priority, and re-entry are the next gate.
