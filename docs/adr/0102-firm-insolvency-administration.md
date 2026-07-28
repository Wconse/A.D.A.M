# ADR 0102: Firm insolvency administration

## Status

Accepted.

## Context

Repeated distress downsizing can reduce a cashless firm to zero workers while it still owns inventory, cash, routes, and unpaid worker claims. Letting that shell trade forever is unrealistic; deleting it would silently destroy property and claims.

## Decision

After six consecutive payroll-distress months, a zero-workforce firm with no liquid owner rescue enters insolvency administration. The authorized operations actor becomes administrator. Declaration records an immutable snapshot of cash, inventories, and wage arrears, clears the distress counter, and emits a typed journal event.

An insolvent firm cannot produce, form market offers, change autonomous production targets, procure production inputs, or fund route expansion. Its assets remain authoritative world state, and payroll may still settle preserved worker claims if estate cash arrives. No inventory, cash, route, or claim is deleted.

## Consequences

- Persistent failure reaches a legible institutional state instead of an immortal shell.
- Operations freeze without destroying recoverable assets or senior worker claims.
- The snapshot is serialized, fingerprinted, replayable, and narrated in the chronicle.
- Liquidation, creditor ranking, asset auctions, and governed reorganization remain separate gates.
