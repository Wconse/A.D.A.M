# ADR 0105: Bounded firm liquidation

## Status

Accepted.

## Context

Insolvency administration preserved a firm's cash, inventories, and worker claims and allowed a fully funded reorganization. An estate with no viable sponsor could nevertheless remain frozen forever. That kept unpaid claims and unusable inventory in authoritative state without a terminal institutional consequence.

Stage 0 needs a bounded resolution that conserves money, preserves worker priority, remains deterministic, and exposes every loss in the event journal. Explicit creditor markets, asset auctions, secured claims, and physical buyers for estate inventory do not exist yet.

## Decision

- Administration lasts twelve complete economic months.
- The ordinary monthly cycle attempts a viable policy-authorized reorganization before considering liquidation. A newly fundable plan therefore has priority over closure.
- At the deadline, available estate cash pays worker wage claims before any owner distribution.
- When cash cannot cover all worker claims, cash is allocated proportionally to claim size. Fixed-point remainders use deterministic largest-remainder allocation with canonical agreement ordering.
- Any unpaid worker balance is explicitly written off and removed from the terminated agreement. It cannot survive as an unpayable phantom claim.
- Cash remaining after worker claims is distributed to owners in proportion to economic rights, again with deterministic largest-remainder allocation.
- Unsold inventory is explicitly written off. No buyer, money, or public inventory is invented merely to produce liquidation proceeds.
- The firm remains a terminal frozen estate, marked with its liquidation date. It cannot reorganize, produce, procure, sell, hire, or receive a new production target.
- The transition is atomic, replayable, fingerprinted, and recorded as `FirmLiquidated` with claims paid, claims written off, inventory written off, and residual owner distribution.

## Consequences

The simulation now distinguishes temporary administration from permanent business death. Workers recover value before owners, but both can suffer visible losses. Inventory destruction is a deliberate Stage 0 approximation rather than a silent disappearance.

The rule changes authoritative state and history, so the simulation rules version advances from 43 to 44.

## Deferred work

A later asset-market slice should replace blanket inventory write-off with physical auctions or transfers to real buyers, then add secured creditors, tax claims, asset-specific seniority, legal costs, successor firms, and entry by new entrepreneurs. Those systems must conserve both money and physical assets and must not weaken worker-claim auditability.
