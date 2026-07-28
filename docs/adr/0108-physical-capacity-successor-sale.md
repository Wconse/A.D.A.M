# ADR 0108: Physical capacity successor sale

## Status

Accepted.

## Context

Terminal liquidation already sells usable inventory before applying the worker, secured-creditor, unsecured-creditor, and owner waterfall. Installed production capacity still vanished when the estate closed, even when a solvent local producer used the same recipe and had both demand for expansion and cash. That destroyed a real productive stock without exposing who acquired it, what was paid, or why no transfer occurred.

Stage 0 does not yet track machines, buildings, depreciation, appraisal markets, or recipe-conversion costs. A narrow rule must preserve compatible capacity without pretending that all firms can absorb arbitrary plants or that an auction can spend cash needed for the buyer's next payroll.

## Decision

Before claim settlement, a liquidating estate offers installed capacity to solvent firms in canonical firm order. A buyer is eligible only when it:

- operates in the same region;
- uses the same production recipe;
- is not insolvent;
- has selected a production target at its current installed ceiling; and
- can pay while retaining one complete month of active payroll.

One capacity batch is priced at one month of that batch's reference-price gross output. This is an administered Stage 0 reserve price grounded in the existing recipe and regional output price, not a claim that capital markets price assets this way. A buyer may acquire at most its current installed capacity in one liquidation, preventing an instantaneous expansion beyond double scale.

The transfer atomically reduces estate capacity, increases successor capacity, moves cash to the estate, and records a typed event. Sale proceeds enter the existing liquidation waterfall. Capacity that attracts no eligible funded buyer is explicitly retired and reported. The auction does not raise the buyer's production target: management must separately choose to use the acquired capacity.

## Consequences

- Productive capacity can survive a firm's legal death without creating output, cash, or ownership rights.
- Worker and creditor recovery can improve from a real asset sale.
- Buyer payroll is protected from fire-sale overextension.
- Recipe and regional compatibility are deliberately strict; relocation, conversion, partial machinery, depreciation, competing bids, and independent appraisals remain future work.
- The transition changes authoritative firm capacity and the serialized event stream, so the simulation version advances from 46 to 47.
