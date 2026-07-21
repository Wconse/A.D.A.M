# ADR 0005: Budget-constrained household demand intentions

- Status: Accepted
- Date: 2026-07-21

## Context

Economic demand must originate from population and household resources rather than scripted national demand. Before firms and market clearing exist, the simulation needs a deterministic representation of what cohorts want and what they can currently afford.

## Decision

World content schema 4 defines:

- goods with stable typed IDs;
- consumption profiles linked to household cohorts;
- hierarchical targets: survival, participation, development, discretionary;
- targets measured per person or per household;
- regional observed prices;
- fixed-point quantities in thousandths of a unit.

Monthly disposable resources are currently approximated as annual cohort income divided by twelve minus debt divided by 120. Targets are processed by need tier. When a tier is not fully affordable, available money is distributed proportionally across that tier using deterministic largest remainders. Lower tiers receive no budget until higher tiers have been processed.

A demand intention records desired quantity, budgeted quantity, and reserved spending. Supply shortages are not yet applied; that belongs to market clearing. Budget shortfall and supply shortfall must remain separate causes.

## Consequences

- Population, household count, income, debt, profile, and regional prices now directly generate demand.
- Media, policy, habit, quality, substitution, savings behavior, taxes, transfers, and wealth drawdown are not yet modeled.
- The current income field mixes labor/property/transfers at cohort level and will be decomposed when monthly household accounts are introduced.
- Cohort age bands are still an aggregation axis, not a complete household-composition matrix. This approximation is tracked in the roadmap and must be resolved before fertility and intra-household allocation are accepted.
