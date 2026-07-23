# ADR 0061: Forecasts derived from observed operations

- Status: Accepted foundation
- Date: 2026-07-23

A firm can deterministically derive expectations from information already present in the world. Expected sales repeat the realized sales revenue of the current accounting period across the chosen horizon. Expected input spending uses the firm's current physically feasible production plan, recipe quantities, and observed regional input prices. Expected financing is zero because concrete loan offers do not yet exist.

The derivation is a replayable command. It records the ordinary `FirmExpectationsUpdated` event with `ObservedOperations` as its source. It reads cash, accounts, inventories, employment, recipes, and prices but moves no money, goods, or workers.

This is deliberately a myopic baseline rather than a hidden prediction oracle. It assumes current sales, throughput, and prices persist. Future slices may add finite sales history, explicit financing offers, supplier contracts, and actor-specific confidence without changing the boundary between observations, beliefs, and decisions.
