# ADR 0129: Employment tenure and switch cooldown

## Status

Accepted.

## Context

Materially better offers made voluntary mobility possible, but a newly hired worker could otherwise be poached in the next monthly market. That creates implausible churn and lets the same wage-pressure signal repeatedly move labor without a stable employment period.

## Decision

Every employment agreement retains bounded months at the current firm. New and reactivated agreements begin at zero; each completed labor-market month advances active tenure by one. A worker becomes eligible for voluntary switching only after three completed months at the source firm.

The destination agreement resets tenure and therefore supplies the cooldown for the next move. Inactive agreements do not accrue tenure. Tenure is serialized and included in the stable fingerprint. The existing one-transition-per-firm monthly boundary remains authoritative.

## Invariants

- A worker cannot switch before three completed months at the current firm.
- New and reactivated employment resets tenure.
- Inactive agreements do not gain tenure.
- Tenure cannot overflow.
- Replay-identical worlds advance tenure identically.

## Consequences

Employment now has short-term stability while preserving long-run wage mobility. Notice periods, counteroffers, occupation-specific experience, and non-wage preferences remain separate mechanisms.
