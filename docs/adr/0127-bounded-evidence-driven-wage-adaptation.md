# ADR 0127: Bounded evidence-driven wage adaptation

## Status

Accepted.

## Context

A competitive labor market that repeats the same bid forever cannot respond to durable worker scarcity or surplus. At the same time, wages must not jump from one failed match, exceed firm liquidity without consequence, or become an unauditable hidden modifier.

## Decision

Competitive wage bids retain their local income anchor and production-urgency premium, then apply a bounded regional adjustment from the previous authoritative labor-market observation.

After each three consecutive months of residual vacancy pressure, bids gain 500 basis points, capped at four steps (+2,000 basis points). After each three consecutive months of residual unemployment pressure, bids lose 250 basis points, capped at four steps (-1,000 basis points). Pressure below three months has no wage effect. Because the two pressure streaks reset on balance or reversal, positive and negative adjustments cannot apply simultaneously.

The adjustment changes only the offered wage. It does not mint cash, reserve payroll, override firm cash coverage, alter qualification requirements, or guarantee a hire. Every replayable employment match carries the signed adjustment and is rejected if its wage or adjustment does not equal the current deterministic bid.

Accepted-match events retain the signed basis-point adjustment. The yearly chronicle reports how many accepted bids were raised or restrained and the net adjustment observed across hires.

## Invariants

- Wage adaptation uses only previously persisted regional evidence.
- One or two pressure months do not change bids.
- Scarcity adds at most 2,000 basis points; surplus subtracts at most 1,000.
- Firm cash must still cover the full adapted monthly wage.
- A direct match command cannot substitute an arbitrary wage or pressure adjustment.
- Equal worlds calculate and replay identical bids and events.

## Acceptance criteria

- Three vacancy-pressure months raise the representative 120-unit bid to 125 with an explicit +500 basis-point cause.
- Three unemployment-pressure months restrain the same bid to 118 with an explicit -250 basis-point cause.
- Existing no-pressure competition retains its accepted wages.
- Chronicle evidence distinguishes raised and restrained accepted bids.
- Formatting, Clippy, workspace tests, documentation, and deterministic 50-year comparison pass.

## Consequences

Firms now react gradually to demonstrated labor scarcity while labor surplus tempers wage escalation. The mechanism remains a regional signal rather than occupation-specific bargaining. Contract renegotiation for existing workers, worker switching, union power, minimum wages, inflation expectations, training, and migration remain future work.
