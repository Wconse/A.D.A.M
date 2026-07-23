# ADR 0064: Advisory production adjustment

- Status: Accepted foundation
- Date: 2026-07-23

The production planner may now produce a non-binding `ProductionAdjustmentProposal` from bounded firm history. It reports average produced batches, sold and unsold quantities, remaining market demand, stockout and unsold observation counts, current physical feasibility, sales-supported batches, a market-demand ceiling, and expected operating cash margin after input and payroll obligations.

Advice expands toward the demand ceiling only when stockout observations outnumber unsold observations and expected operating cash margin is positive. It contracts toward sales-supported output when unsold observations dominate or expected margin is non-positive. Otherwise it holds near observed production. Every advisory result is capped by current labor, capacity, and available inputs.

The unmet market quantity is treated only as an upper bound because buyers may not have selected this firm. The proposal changes no inventories, employment, cash, or production schedule. A manager or AI actor must issue a separate future production decision command.
