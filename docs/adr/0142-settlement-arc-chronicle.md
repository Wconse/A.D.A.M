# ADR 0142: Settlement arc chronicle narration

## Status

Accepted.

## Context

Migration, relocation fees, housing pressure, project authorization, and delayed completion were authoritative but absent from the Stage 0 chronicle. A material settlement system that changes only hidden state cannot produce readable history.

## Decision

The yearly chronicle aggregates typed settlement events without reading mutable current state. It reports households and people moved, named origin and destination regions, projected destination housing pressure, relocation fees, public construction commitments, planned dwellings, completed dwellings, and resulting capacity.

Settlement-only years receive importance 68: more important than ordinary production or household borrowing, but below emergency relief, firm distress, rationing, insolvency, conflict, sovereign crisis, and deaths. Region names are resolved from registration events so narration remains replayable and historical.

## Consequences

The adaptation chain is now readable: durable labor imbalance and services attract a solvent household; crowding prices or blocks settlement; real fees enter the treasury; public authorities commit funds; dwellings arrive after delay. This slice changes no rules or world state, so the simulation version and stable fingerprint remain unchanged.
