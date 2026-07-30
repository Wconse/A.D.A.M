# ADR 0121: Differentiated public-reserve priorities

## Status

Accepted for Stage 0.

## Context

A single national coverage target treated every survival good in every region as equally strategic. Under a binding treasury ceiling, canonical identifier order then decided which shortage was funded first. That was deterministic but not a political choice and could make low-consequence stocks crowd out medicine or an isolated region's only staple.

## Decision

A political-office holder may assign a bounded priority share to a `(country, region, good)` reserve target through the shared replayable command boundary. Priorities use basis points:

- `10,000` applies the full national coverage target;
- intermediate values proportionally scale that regional-good target with deterministic upward rounding;
- `0` excludes the target from automatic stockpiling without deleting existing physical reserves.

Unconfigured targets default to `10,000`, preserving previous behavior. During observed procurement, requirements are reviewed from highest priority to lowest. Country, region, and good identifiers remain stable tie-breakers. The existing country-wide monthly procurement ceiling is still authoritative, so priority changes who gets scarce funds rather than creating money or inventory.

Every priority decision is validated against real country, region, good, and office-holder authority; it is evented, serialized, fingerprinted, and replayable. Requirement-review evidence records the applied priority, and the chronicle states when differentiated targets shaped allocation.

## Invariants

- A priority cannot create goods, treasury cash, or purchasing authority.
- A region can only be prioritized by the country that contains it.
- Zero priority stops new automatic stockpiling but never destroys already-held reserves.
- Scarce budgets fund higher priority targets before lower priority targets.
- Equal priorities preserve canonical deterministic ordering.
- Direct execution, command replay, save/load, and fingerprints remain identical.

## Consequences

Reserve doctrine now exposes a concrete political tradeoff: a government may protect medicine, staples, or a vulnerable region at the cost of leaving other needs less buffered. This creates a useful future seam for elections, lobbying, regional inequality, and information failures without modeling those institutions prematurely.

Deliberate limits: priorities are explicit decisions, not yet learned from regional deaths, seasonality, import dependence, or lobbying. They do not allocate maintenance spending differently and cannot move reserves between regions. The next gate should use observed vulnerability to propose bounded priority revisions, then introduce public interregional logistics only after priorities can explain why a transfer is attempted.
