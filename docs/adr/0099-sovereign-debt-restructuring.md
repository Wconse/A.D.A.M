# ADR 0099: Sovereign debt restructuring

## Status

Accepted.

## Context

The annual fiscal loop already charges interest and restrains discretionary spending as public debt rises. A long-run audit showed that this can still leave a country servicing an ever-growing stock after the spending floor binds. Debt then becomes an arithmetic spiral with no terminal institutional event, while creditors and a full bond market do not yet exist.

## Decision

Annual closure tests debt sustainability after taxes, spending, treasury use, and new borrowing are settled. A country restructures only when both conditions hold:

- closing public debt exceeds twice current measured annual output; and
- annual interest is at least one third of realized tax revenue.

A restructuring writes off 40% of the closing principal. It is not free relief: legitimacy falls by 800 basis points and elite cohesion by 500 basis points. The transition emits `PublicDebtRestructured` with debt before, debt after, and principal written off. The ordinary fiscal closure then records the reduced authoritative debt stock.

The rule is deterministic, symmetric for every country, and uses only auditable monetary state. It deliberately represents a coarse Stage 0 settlement between the state and aggregated creditors; it does not invent creditor cash, foreign ownership, maturity ladders, inflation, or exchange rates before those systems exist.

## Consequences

- Debt crises have a finite but costly resolution instead of compounding forever.
- A default can restore fiscal room while causing a sharp political shock, creating a recovery-versus-instability tradeoff.
- Repeated restructurings remain possible if the material tax base is not repaired.
- Future sovereign finance work should replace the threshold rule with explicit bonds, holders, maturities, risk premia, negotiation, and creditor losses.