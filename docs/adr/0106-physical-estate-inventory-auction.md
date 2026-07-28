# ADR 0106: Physical estate inventory auction

## Status

Accepted.

## Context

Terminal liquidation wrote off all estate inventory even when a solvent local producer had an immediate productive use for it. This destroyed useful physical stock and reduced worker recovery without a market attempt.

## Decision

Before worker claims and owner residuals are distributed, liquidation offers each inventory good to solvent firms in the same region whose production recipe consumes it. Buyers are considered in stable firm order. Demand is bounded by the buyer's production target, existing stock, and cash. Trades settle at the regional reference price, move physical inventory and cash atomically, and emit a typed sale event. Sale proceeds join the estate cash pool, preserving worker priority. Stock without an eligible, solvent, liquid buyer is still explicitly written off.

## Consequences

Liquidation can now preserve productive capacity and improve worker recovery without inventing buyers or money. The first implementation is a deterministic administered auction rather than competitive bidding: it is local-only, uses reference prices, excludes insolvent buyers, and does not transfer capital equipment or legal ownership. The rule changes authoritative history, so `SIMULATION_VERSION` advances from 44 to 45.
