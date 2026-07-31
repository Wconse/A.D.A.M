# ADR 0136: Cohort lifecycle clock

## Status
Accepted.

## Decision
Each household cohort carries an explicit integer count of years spent in its current age band. Annual closure advances that clock and deterministically moves cohorts through child, youth, adult, mature, and senior bands. Entering adulthood makes previously dependent cohorts available but unemployed; entering senior age retires the cohort, converts its household type, and closes active employment agreements without moving money. Lifecycle transitions preserve population and household financial stocks.

Initial content starts at year zero of its declared band. This is an explicit Stage 0 approximation until content supports calibrated within-band age distributions.

## Consequences
Demography now changes economic roles rather than only scaling population. Retirement can remove labor supply and payroll obligations, while cohorts remain stable deterministic identities suitable for replay and migration.
