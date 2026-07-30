# ADR 0123: Observed household import dependence in reserve priorities

## Status

Accepted for Stage 0.

## Context

Regional reserve priorities could react to uncovered gaps and idle spoilage, but the government did not retain whether survival consumption was supplied locally or delivered over a route. A region could therefore depend almost entirely on imported food or medicine for years without that exposure entering preparedness doctrine. Treating every import as a failure would be equally wrong: trade is often efficient and resilient.

## Decision

Every settled survival-market fill is now classified by the buyer region and good as local or delivered import. Monthly state retains both physical quantities and derives an exact imported share. A typed event records country, region, good, local quantity, imported quantity, and imported share after atomic market settlement.

Configured reserve-priority reviews gain a third persistence streak. Import reliance is active only when delivered imports provide at least 6,000 basis points of fulfilled survival consumption for that regional good. Six consecutive high-reliance months raise a priority below 10,000 by 500 basis points through the ordinary authorized priority command. Occasional or minority imports reset the streak and cause no revision.

Uncovered serviceable gaps still react after three months and take first precedence. Sustained high import reliance takes second precedence. Idle-spoilage retrenchment remains third, so a heavily import-dependent region is not automatically deprioritized merely because its precautionary stock was unused during peaceful trade.

Monthly import-dependence state, persistence streaks, review evidence, revision reason, commands, events, serialization, replay, and fingerprints are authoritative. The chronicle distinguishes revisions caused by sustained household import reliance.

## Invariants

- Only physically settled survival imports count; quotes and intentions do not.
- Local and imported quantities are conserved from the same settled fills.
- Imports below 60% of fulfilled survival consumption are not classified as strategic dependence.
- One import-heavy month cannot change doctrine.
- Import dependence changes priority, not treasury, inventory, route capacity, or consumption.
- Revisions still require the existing political authority and shared command boundary.
- Direct execution, command replay, save/load, and fingerprints remain deterministic.

## Consequences

A trade-dependent region can gradually build preparedness without the model declaring trade itself harmful. The distinction creates future space for supplier concentration, route reliability, sanctions, and diplomatic risk while keeping the first rule observable and bounded.

Deliberate limits: dependence is currently based on import share, not supplier concentration, route redundancy, travel time, hostility risk, or domestic surge capacity. The next gate should distinguish diversified imports from single-route or single-country dependence before public interregional reserve logistics is introduced.
