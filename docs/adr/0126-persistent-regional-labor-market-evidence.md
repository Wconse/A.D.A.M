# ADR 0126: Persistent regional labor-market evidence

## Status

Accepted.

## Context

Competitive matching creates real hires, but one-month match results are insufficient for later wage, training, or migration decisions. The simulation needs to distinguish durable labor surplus from durable skill or worker scarcity without inferring either from missing events.

## Decision

Each region containing at least one recipe explicitly opted into competitive labor matching receives one authoritative observation after monthly matching.

The observation records remaining unallocated unemployed workers, remaining target-derived vacancies, funded offers made before allocation, accepted hires, the mean offered wage, and consecutive unemployment or vacancy pressure. Unemployment pressure advances only when remaining available workers exceed remaining vacancies. Vacancy pressure advances only when remaining vacancies exceed available workers. Balance or reversal resets the corresponding streak.

Observations are stored by region, serialized, fingerprinted, and emitted as typed domain events. Regions containing only legacy recipes produce no labor-market state or evidence. The yearly chronicle aggregates offers, hires, residual workers and vacancies and identifies the regions with the longest persistent pressure.

Only cohorts explicitly marked unemployed participate in automatic matching and regional availability evidence. This prevents the first matching model from silently treating partially unallocated members of an employed cohort as active job seekers.

## Invariants

- Evidence is captured after matching and therefore describes residual pressure.
- One region has at most one authoritative labor observation per monthly stage.
- Pressure streaks are bounded `u8` values and saturate rather than overflow.
- Legacy-only regions remain absent from the labor evidence map.
- State, event replay, stable fingerprints, and chronicle attribution use canonical region ordering.

## Acceptance criteria

- Competition for one worker leaves one residual vacancy and one month of vacancy pressure.
- A region with labor but zero target-derived vacancies accumulates unemployment pressure across months.
- Typed evidence includes offers, hires, residual quantities, wage, and both streaks.
- The annual chronicle identifies persistent regional vacancy or unemployment pressure.
- Formatting, Clippy, workspace tests, documentation, and deterministic 50-year comparison pass.

## Consequences

The world now retains the evidence required for bounded wage adaptation, training, and migration instead of reacting to a single failed match. Counts are cohort-aggregate worker units rather than individual biographies. Qualification-specific vacancy buckets, employed-worker search, commuting, bargaining institutions, and vacancies by occupation remain future slices.
