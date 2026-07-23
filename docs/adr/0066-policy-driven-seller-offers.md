# ADR 0066: Policy-driven seller offer formation

- Status: Accepted foundation
- Date: 2026-07-23

Firms can now derive concrete local-market offer plans from authoritative output inventory, bounded observations of their own realized sales, regional reference prices, and their authorized firm policy.

For each firm with policy, the planner identifies the recipe output good and computes average monthly sales from periods in which the firm actually offered that good. The inventory buffer target is `average observed monthly sales × inventory_buffer_days / 30`, rounded upward and capped by inventory. Remaining output inventory is offered. The offer unit price is the current regional reference price adjusted by the policy markup.

The planner returns both the retained and offered quantities so inventory policy remains visible. A zero offered quantity produces no concrete `MarketOffer`. Missing reference prices and arithmetic overflow are explicit errors. Offer planning does not clear the market, mutate inventory, or infer sales; only settlement can turn an offer into a transaction and seller-side outcome.

This closes the causal loop from authorized production targets through physical output into policy-shaped supply, while preserving a separate market-clearing step.
