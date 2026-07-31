# ADR 0154: Persistent regional government-program memory

## Status

Accepted for Step 100.4a.

## Context

A delivered service level alone cannot explain the politics of a government promise. Regions remember what was promised, what was actually committed, what arrived, and whether they were deliberately omitted. A broken promise must also be politically worse than receiving no promise at all.

## Decision

Each government program stores cumulative regional memory. After every annual execution, all domestic regions receive a record containing cumulative promised, committed, and delivered funding, current fulfillment, outcome kind, consecutive excluded years, and bounded political memory.

Outcomes are `Beneficiary`, `Underfulfilled`, or `Excluded`. A charter share of zero is explicit exclusion. Non-zero shares compare delivery with the annual regional promise. Fulfillment rewards are weaker than shortfall penalties: full delivery adds 150 memory points while a fully broken non-zero promise subtracts 300. Explicit exclusion subtracts 100, so making and breaking a promise is worse than making none. Memory is bounded to `-5_000..=5_000`.

Typed events expose the complete annual evidence and the incremental political shift. State, replay, serialization, and fingerprints include the memory.

## Invariants

1. Every domestic region receives an annual outcome, including zero-share regions.
2. Cumulative promised, committed, and delivered money cannot be silently discarded.
3. A fully broken promise has a stronger negative shift than explicit exclusion.
4. Consecutive exclusion years reset when a region receives a non-zero share.
5. Political memory remains bounded and deterministic.
6. Recording memory does not itself mutate satisfaction or legitimacy; those delayed consequences belong to the next transition.

## Consequences

Programs now produce durable political winners, underfulfilled claimants, and excluded territories rather than transient delivery events. `SIMULATION_VERSION` advances 91 -> 92. The next slice applies this evidence to regional satisfaction, confidence, legitimacy, and elite cohesion without double-counting the same year.
