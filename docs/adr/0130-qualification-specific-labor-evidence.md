# ADR 0130: Qualification-specific labor-market evidence

## Status

Accepted.

## Context

A single regional labor-pressure streak cannot distinguish general unemployment from a shortage of workers qualified for a particular production technology. Applying one signal to every recipe can raise low-skill offers because an unrelated specialist occupation is scarce.

## Decision

Each configured region now retains a separate observation for every minimum-education level demanded by active firms. The observation records target-derived vacancies, available unemployed workers meeting or exceeding the threshold, and bounded unemployment and vacancy-pressure streaks.

Competitive wage adaptation reads the exact qualification bucket for the firm's recipe. Higher-educated workers count as qualified supply for lower thresholds, but vacancies remain assigned to the exact recipe threshold. Skill observations are serialized, fingerprinted, emitted as typed events, and summarized in the yearly chronicle.

## Invariants

- Legacy recipes without an explicit labor profile create no skill bucket.
- Each vacancy belongs to exactly one minimum-education bucket.
- Supply is measured only from unallocated, unemployed, non-training cohorts meeting the threshold.
- Wage pressure comes from the target firm's qualification bucket.
- Equal worlds produce identical buckets and streaks.

## Consequences

A region can now exhibit broad unemployment and specialist scarcity simultaneously. Occupations within one education level, credentials, experience, and geographic mobility remain future refinements.
