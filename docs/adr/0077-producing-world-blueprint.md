# ADR 0077: Producing world blueprint and proportional fallback income

## Status

Accepted — 2026-07-23

## Context

The full monthly CLI path and event-backed chronicle exposed a fatal content gap: the example world defined demand, prices, health, and political response, but no recipes, firms, employment, ownership, management policy, or production targets. A 50-year run therefore produced permanent physical starvation and reduced the three-region population to 100 while the legacy macro output series continued to grow.

A second integration defect appeared after producers were added. Any active employment agreement suppressed the entire aggregate annual income of its cohort even when only a small fraction of the cohort was contracted. Four 20,000-worker firms therefore replaced the income of a multi-million-person cohort with negligible payroll, creating artificial debt and delayed collapse.

## Decision

World content schema version 5 includes production recipes, firms and inventories, employment agreements, ownership stakes, corporate appointments, firm policies, and initial production targets. `WorldBlueprint::build_world` registers them in dependency order and applies policy/target commands only after authority exists.

The Stage 0 example world contains four survival producers in each of three regions: food, housing, energy, and healthcare. Each firm has local labor, a controlling local actor, an operations appointment, an offer policy, cash, capacity, and an initial target. Dependent cohorts receive an explicit household resource share because the current cohort model does not yet represent intra-household transfers.

Monthly fallback income is now reduced proportionally by the number of contracted workers rather than disabled by the mere existence of one contract. Payroll still adds concrete contracted income; the remaining cohort population retains its configured non-contract income. Fully contracted cohorts receive no fallback income.

## Consequences

- Configured content now closes the path from labor and authorized management through physical production, seller offers, household purchases, observations, and next-month targets.
- Partial employment can no longer erase the income of millions of uncontracted people.
- The deterministic seed-47 50-year audit retains roughly 36.8 million people, continues producing and trading through 2074, and ends near 98.4% minimum survival fulfillment instead of collapsing to 100 people.
- Aggregate regional output remains a legacy annual harness and is not yet derived from micro production or trade.
- Prices, non-contract income, and intra-household resource shares remain configured Stage 0 approximations.

## Verification

- Content integration test builds an authorized production chain from TOML.
- Core unit test proves partial contracts preserve only the uncontracted income share.
- Full repository quality script passes.
- Real one-, ten-, and fifty-year CLI runs confirm nonzero production, trade, adaptive targets, and persistent population.
