# ADR 0004: Household cohort population ledger

- Status: Accepted
- Date: 2026-07-21

## Context

Population must drive labor, production, consumption, education, migration, politics, and mortality. A single regional population number cannot produce those behaviors, while one entity per citizen is unnecessary for Stage 0.

## Decision

Population is represented by data-driven household cohorts. Each cohort currently records:

- region;
- people and household counts;
- age band;
- household type;
- education;
- employment status;
- total annual income;
- liquid wealth;
- debt.

The sum of cohort populations is required to equal the authoritative regional population. Content loading fails when the ledger is unbalanced.

Cohort IDs are stable typed 32-bit identities. Schema version 3 reserves zero and validates household counts and non-negative financial stocks.

Until births, deaths, aging, education, and migration transitions are implemented, the aggregate annual harness rescales cohorts proportionally with a deterministic largest-remainder algorithm. This preserves exact population accounting and cohort shares but is explicitly temporary, not an accepted demographic model.

## Consequences

- Household demand can originate from actual cohort budgets in the next slice.
- Demographic transition systems must update cohorts and regional totals together.
- Cohort split/merge rules must preserve people, households, financial stocks, and event traceability.
- Proportional rescaling must be removed when real demographic flows replace the aggregate growth harness.
