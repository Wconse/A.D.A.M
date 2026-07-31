# ADR 0149: Government-program appropriation

## Status

Accepted for Step 100.2.

## Context

A program charter is a political promise, not money. The simulation needs a separate authoritative decision that can fully fund, underfund, explicitly zero-fund, debt-fund, or later cancel a program without silently rewriting the original promise.

## Decision

- Add replayable annual program appropriation and cancellation commands.
- An appropriation records its real source: existing treasury cash or newly issued bounded public debt.
- Treasury funding immediately leaves the general treasury and becomes program carryover. Debt funding increases public debt and commits the proceeds directly, without briefly inflating uncommitted treasury cash.
- Zero is a legal appropriation and remains typed historical evidence. Negative values are invalid.
- Each program receives at most one appropriation decision per simulation year inside its declared schedule.
- Cancellation stops future appropriations but does not erase or refund already committed carryover.
- Every decision requires the same real political-office authority as declaration.

## Invariants

1. Appropriation creates no unbacked money: treasury funding reduces treasury exactly; debt funding raises debt exactly.
2. Debt funding cannot exceed the existing sovereign headroom derived from real output and outstanding debt.
3. Rejected decisions are atomic.
4. Promise, appropriation, shortfall, source, and carryover remain distinct.
5. Zero funding, full funding, concentration across programs, and cancellation are engine-legal political choices.
6. Direct and replayed decisions produce identical state and fingerprints.

## Consequences

Multiple programs can now compete politically for finite fiscal capacity without fairness caps. Autonomous competition policy is deferred; this gate establishes the common command and accounting boundary first. `SIMULATION_VERSION` advances 86 -> 87.
