# ADR 0075: Physical survival scarcity can be proportionally rationed

- Status: Accepted foundation
- Date: 2026-07-23

Government emergency policy now has a separate physical-shortage strategy. The Stage 0 choices are canonical market allocation and proportional survival rationing. Rationing runs after household demand and seller offers are known but before market clearing. It activates only when local offered quantity is below total requested survival quantity for a region and good.

Under proportional rationing, each cohort receives the same fraction of its requested survival quantity. Integer milli-quantity remainders are distributed by largest remainder and stable cohort ID, preserving exact aggregate supply and deterministic replay. Quotas constrain market orders but do not provide purchasing power: cohorts must still pay ordinary offer prices, and unaffordable quota can remain unsold for the existing relief system to observe.

Reducing an order to its quota must not erase the underlying need. The commercial cycle therefore restores rationed-away quantities to cohort unmet-demand ledgers after clearing and adds the same withheld demand to seller market observations. Health, firm expectations, policy analysis, and the chronicle continue to see the original physical shortage rather than interpreting a lower administrative quota as fulfilled need.

A replayed integration test gives two equally funded cohorts one unit of food against two units of survival need. Canonical priority would feed the first cohort fully and the second not at all. Proportional rationing creates two 0.5-unit fills, records 0.5 unmet for each cohort, and produces 50% survival fulfillment for both. No goods or money are created.

This is allocation under scarcity, not a supply response. Procurement, imports, reserve release, capacity investment, enforcement costs, evasion, corruption, black markets, and politically favored ration weights remain future slices.
