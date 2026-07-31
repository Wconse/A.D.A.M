# ADR 0146: Unrestricted authorized regional service allocation

## Status

Accepted.

## Context

Regional interests and fiscal incidence existed, but recurring public-service spending was divided mechanically. A proposed universal fairness floor would have prevented both players and autonomous rulers from making extreme but feasible allocations, contradicting the project-wide simulation freedom principle in `AGENTS.md`.

## Decision

A current political-office holder may set exact regional shares of the country's discretionary public-service budget through the shared replayable command boundary. Shares must sum to exactly 10,000 basis points and may include zero shares. A single region may receive the full 10,000 while every other region receives nothing. Omitted domestic regions receive zero. Unknown, foreign, and unauthorized assignments are rejected atomically.

The engine validates only authority, region ownership, exact accounting, and available spending. It does not impose fairness, minimum regional shares, gradualism, or concentration caps. Player and autonomous actors use the same persistent allocation state and command legality.

When no explicit political allocation exists, the default autonomous planner follows a prudent behavior policy. It weights population, observed public-service shortfall, an existing public-service priority, and low regional satisfaction. These are AI decision weights rather than engine restrictions; an explicitly commanded allocation completely replaces them. Integer budget amounts use deterministic largest-remainder settlement and conserve the full available service budget exactly.

Regional delivery responds to the actual allocated budget relative to local output. A zero-funded region therefore degrades toward zero service funding, while a concentrated recipient remains limited by administrative delivery dynamics rather than an artificial allocation cap. Typed events distinguish autonomous and explicit allocations, and the chronicle exposes concentrated decisions.

## Consequences

A player with real political authority can favor a capital, punish a region, or make a disastrous allocation and experience the resulting service decline, fiscal incidence, regional satisfaction, confidence, and legitimacy effects. Default AI remains prudent without narrowing the action space. Later actor-personality and influence systems may generate favoritism or extremism through policy choice, not global legality.
