# ADR 0122: Evidence-driven regional reserve-priority adaptation

## Status

Accepted for Stage 0.

## Context

Regional-good reserve priorities made scarcity an explicit political choice, but those choices remained fixed forever. A repeatedly uncovered target could stay de-prioritized despite months of serviceable supply, while idle stocks could continue spoiling without the doctrine acknowledging that the assigned priority was too high.

## Decision

Every configured regional-good reserve priority now retains two bounded evidence streaks:

- **uncovered vulnerability** grows when a reviewed target remains uncovered even though local supply, rather than physical scarcity, is not the binding constraint;
- **idle spoilage** grows when carried stock suffers baseline spoilage while the target has no remaining gap and distributes nothing.

After three consecutive uncovered months, a priority below 10,000 rises by 500 basis points. After six consecutive idle-spoilage months, a non-zero priority falls by 500 basis points. Uncovered vulnerability takes precedence when signals conflict. A triggering streak resets; absent evidence resets its own streak.

Changes are executed through the ordinary authorized `SetGovernmentReservePriority` command using the current political-office holder. Reviews and resulting decisions are typed events. Priority pressure, reviews, decisions, serialization, replay, and fingerprints are authoritative. The chronicle identifies the leading regional revision.

## Invariants

- One abnormal month cannot change a regional priority.
- Supply-limited shortages cannot be solved cosmetically by raising priority.
- Every adjustment is exactly 500 basis points and remains inside 0..=10,000.
- Existing reserves are never destroyed by a priority decrease.
- Automatic revision uses the same authority and command boundary as an explicit decision.
- Direct execution, command replay, save/load, and fingerprints remain deterministic.

## Consequences

A previously de-prioritized but repeatedly uncovered medicine or staple target can recover protection gradually. Conversely, a buffer that repeatedly decays unused is slowly deprioritized, freeing future treasury authority for other needs. This creates path-dependent institutional learning without an optimizer or scripted rescue.

Deliberate limits: the first learning signal is direct reserve performance. Excess mortality is cohort-level rather than good-attributed, and household import dependence is not yet retained as monthly regional-good evidence. Those signals must be made explicit before they influence priority. The next gate should persist delivered household import dependence by regional good and feed repeated reliance into the same bounded review, without treating all imports as failure.
