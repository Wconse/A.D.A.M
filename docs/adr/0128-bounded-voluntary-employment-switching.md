# ADR 0128: Bounded voluntary employment switching

## Status

Accepted.

## Context

Competitive hiring and adaptive wages affect only unallocated workers. A fully employed cohort therefore cannot respond to materially better offers, incumbent firms never face retention pressure, and wage competition stops once the local labor pool is allocated.

## Decision

After ordinary unemployment matching, firms with a remaining configured vacancy may recruit one worker from an active local agreement. A switch is allowed only when the new funded monthly wage is at least 10% above the current wage.

Switch candidates must satisfy the target recipe's education floor, share the target region, and come from a distinct firm. Insolvent targets, unfunded offers, stale agreements, fabricated wages, and sub-threshold improvements are rejected atomically. The competitive bid is re-derived during command execution, including the persisted regional-pressure adjustment.

Each firm may participate in at most one labor transition per monthly market, whether as an ordinary hiring destination, switching destination, or source losing a worker. Ordinary unemployment hires run first; switching fills only residual vacancies. This gives job seekers priority and prevents same-month worker chains or oscillation.

The switch decrements the source agreement by exactly one worker and creates or expands the destination agreement. Source wage arrears remain on the source agreement as an enforceable historical claim. No wage is prepaid and normal payroll remains authoritative.

Switch offers participate in regional offer and placement evidence. Typed events record source, destination, cohort, old and new wage, qualification floor, and labor-pressure adjustment. The yearly chronicle reports switches and aggregate monthly wage gain.

## Invariants

- No worker is duplicated or destroyed during a switch.
- Source and destination are distinct local firms.
- The destination has a real target-derived vacancy and enough cash for the offered wage.
- The offered wage is current, deterministic, and at least 10% above the source wage.
- A firm participates in at most one labor transition per month.
- Existing source arrears survive worker departure.
- Direct execution and replay produce identical state, evidence, and fingerprints.

## Acceptance criteria

- A worker earning 100 moves to a funded 113 offer.
- Source staffing falls by one exactly as destination staffing rises by one.
- The ordinary matching result remains empty when the only transition is a switch.
- The switch and monthly completion events identify the move.
- Direct and replayed monthly markets are equal and fingerprint-identical.
- Formatting, Clippy, workspace tests, documentation, and deterministic 50-year comparison pass.

## Consequences

Incumbents now face a concrete retention cost and productive entrants can attract experienced workers without creating labor. The model deliberately omits notice periods, commuting, moving regions, counteroffers, worker preferences beyond wage, unions, non-wage conditions, and occupation-specific experience. These can be added once switch frequency and vacancy dynamics are observed.
