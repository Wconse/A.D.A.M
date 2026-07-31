# ADR 0131: Household-funded workforce training

## Status

Accepted.

## Context

Persistent qualification shortages currently raise wages but cannot expand qualified supply. Instant education changes would create human capital without time or resource cost, while training every unemployed cohort from a transient signal would overreact.

## Decision

After six consecutive months of qualification-specific vacancy pressure, an eligible unemployed local cohort may enter a three-month training program. The cohort advances exactly one education level, cannot be matched while training, and pays tuition equal to one month of its aggregate baseline income, with a minimum positive cost.

Enrollment requires sufficient liquid household wealth and unallocated workers. At most one cohort per region starts training in a monthly labor stage. Existing programs progress before new enrollment; completion changes education authoritatively and only affects matching from the following month. Enrollment and completion are typed events. Active programs are serialized and fingerprinted, and the chronicle reports starts, completions, and tuition.

## Invariants

- Training starts only after six skill-shortage months.
- Training consumes real household wealth and never creates cash.
- A cohort cannot train and accept employment simultaneously.
- Education advances by one adjacent level after three completed training months.
- Training state and outcomes replay identically.
- Unfunded or already-qualified cohorts do not enroll.

## Consequences

Human-capital supply now responds slowly and materially to demonstrated demand. Employer sponsorship, public education budgets, dropout risk, training capacity, and multi-cohort subdivision remain future work.
