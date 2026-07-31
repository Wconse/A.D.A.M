# ADR 0155: Annual government-program political consequences

## Status

Accepted for Step 100.4b.

## Context

Regional program memory must become causal without making command order political. Immediate mutations during each program execution would let ordering change national outcomes and could apply reactions more than once.

## Decision

Apply program consequences once during annual regional-interest closure, after ordinary country updates. Only programs executed in the closing simulation year participate.

Each region receives the sum of its current program outcome shifts, bounded to ±500 satisfaction points per year. Country legitimacy uses the population-weighted regional program shift. Elite cohesion uses both that national average and the spread between the best- and worst-treated regions: broad failure hurts cohesion, while unequal treatment adds a distinct polarization penalty.

Typed evidence records execution year, national average, polarization, indicator shifts, and resulting legitimacy/cohesion.

## Invariants

1. A program execution affects political indicators at most once through annual closure.
2. Closed-year consequences do not depend on program command order.
3. Regional satisfaction reacts locally; national legitimacy is population weighted.
4. Unequal treatment can reduce elite cohesion even when the national average is tolerable.
5. No program executed in the year means no program consequence event or indicator mutation.
6. All effects are bounded, replayable, and fingerprinted through authoritative state.

## Consequences

Broken promises and exclusion now outlive delivery bookkeeping and change later simulation conditions. `SIMULATION_VERSION` advances 92 -> 93. Step 100.4 now has persistent winners/losers and delayed national consequences; patron and bypassed-center attribution remains visible in existing power evidence and will be consolidated by the program chronicle.
