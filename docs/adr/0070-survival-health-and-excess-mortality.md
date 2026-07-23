# ADR 0070: Survival fulfillment causes health loss and excess mortality

- Status: Accepted foundation
- Date: 2026-07-23

Settled survival consumption is now a material input rather than an observational dead end. Every cohort retains monthly survival fulfillment, functional capacity, and a fixed-point mortality remainder. Fulfillment below 80% consumes functional capacity; adequate fulfillment restores it gradually. Lost capacity produces age-sensitive excess mortality and reduces fallback labor income and effective contracted production labor.

Deaths are applied atomically to cohort and regional population ledgers. Household counts and aggregate annual income are rescaled to survivors, aggregate wealth and debt remain with the surviving cohort, and active employment agreements lose workers proportionally. The retained mortality remainder prevents small cohorts from escaping persistent mortality through integer truncation. Health state, stage guards, and the remainder participate in save/replay and stable fingerprints.

The current response coefficients are explicit Stage 0 approximations: a cohort needs 80% survival fulfillment to avoid deterioration, health loss and recovery use fixed monthly divisors, and excess mortality is proportional to lost capacity with age-band vulnerability. These values are not epidemiological claims and must later be replaced by good-specific physiological requirements, disease, climate, housing, healthcare, and calibrated age schedules.

A full monthly integration test now proves the chain `market shortage -> low realized survival consumption -> functional loss -> excess deaths -> exact regional population accounting`. This closes the first material consequence loop. It does not yet model substitution, informal supply, migration, organized relief, blame attribution, protest, or government response; those are the next feedback gates rather than reasons to leave persistent deprivation consequence-free.
