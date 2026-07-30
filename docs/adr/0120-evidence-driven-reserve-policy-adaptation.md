# ADR 0120: Evidence-driven public reserve policy adaptation

## Status

Accepted for Stage 0.

## Context

Reserve coverage and procurement authority were fixed forever after configuration. The simulation could observe repeated shortages, budget ceilings, spoilage, and failed upkeep, but governments never learned from those outcomes. Immediate monthly reactions would be equally implausible: one unusual shortage or a single damaged lot should not reverse national doctrine.

## Decision

Every country using `ReserveRelease` now performs an observed policy review after procurement and distribution. The review aggregates current-month typed evidence and updates four bounded persistence streaks:

- preparedness pressure: survival shortage exceeded opening reserve stock, while neither supply nor budget was the binding constraint;
- budget pressure: procurement evidence explicitly identified the monthly treasury ceiling as binding;
- upkeep stress: underfunded maintenance caused neglect spoilage;
- waste pressure: baseline spoilage occurred in a month with neither observed shortage nor reserve distribution.

Policy changes are deliberately slow and ordered. Two consecutive upkeep-stress months reduce target coverage by one month; if coverage is already one month, procurement authority falls by 500 basis points. Three quiet waste months reduce coverage by one. Two budget-constrained months raise procurement authority by 500 basis points. Three recurring unbuffered shortage months raise coverage by one. Coverage remains in 1..=12 months and authority in 0..=10,000 basis points. Fiscal retrenchment takes priority over expansion when evidence conflicts. The streak that triggers a decision resets; unrelated evidence continues accumulating or resets when absent.

The review is a once-per-month atomic stage inside the commercial cycle, after reserve procurement and release. It has a replayable command boundary. Per-country events preserve aggregated physical evidence, all streaks, and previous/new policy values; a completion event records reviewed and changed country counts. Pressure state, review date, policy changes, events, save data, and fingerprints are authoritative. The yearly chronicle explains the direction and leading named-country revision.

## Invariants

- One month of noise cannot change doctrine.
- Supply-limited shortages do not falsely increase coverage or procurement authority.
- A budget increase requires repeated explicit budget binding.
- Neglect-induced losses cause retrenchment before expansion.
- All adjustments are one bounded step and cannot exceed existing policy limits.
- Countries outside `ReserveRelease` are not reviewed.
- Duplicate review is rejected atomically.
- Direct execution, command replay, serialization, and fingerprints remain deterministic.

## Consequences

Three recurring serviceable shortages can move coverage from one to two months. Two budget-limited months can move procurement authority from 4,000 to 4,500 basis points. Two months of neglect spoilage can cut a three-month target to two months. This creates institutional memory and path dependence without a volatile optimizer.

Deliberate limits: the rule is bounded institutional learning, not utility maximization. It does not forecast seasonality, distinguish goods in national doctrine, model elections or legislative vetoes, or invest in storage quality. The next gate should expose differentiated reserve priorities by good or region before public interregional logistics.
