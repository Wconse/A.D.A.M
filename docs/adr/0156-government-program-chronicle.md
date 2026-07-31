# ADR 0156: Government-program chronicle

## Status

Accepted for Step 100.5.

## Context

The graphical client and CLI need a human-readable political history, but narration must not become a second simulation. Program history should be derived solely from authoritative typed events.

## Decision

Extend the deterministic annual chronicle with government-program evidence. The chronicle tracks declarations, appropriations, cancellations, delivery, carryover, delay, physical material use, temporary employment and wages, regional beneficiaries/underfulfillment/exclusion, the clearest regional loser, and national legitimacy/cohesion consequences.

Program names are learned from declaration events and reused in later annual entries. Program activity can create a chronicle entry even before a normal annual economic closure, allowing political decisions to remain visible immediately.

## Invariants

1. Chronicle generation remains read-only and event-derived.
2. Program names and outcomes are deterministic under replay.
3. Narration does not infer resources that were not represented by typed events.
4. Delivery, carryover, delay, materials, labor, losers, and political consequences remain separately visible.
5. Program-only years are narratable without inventing a completed economic year.

## Consequences

Government programs now produce a compact textual history suitable for CLI use and later graphical timeline cards. `SIMULATION_VERSION` advances 93 -> 94. The simulation-side half of the anniversary milestone is complete; Step 100.6 begins the independent Bevy presentation layer.
