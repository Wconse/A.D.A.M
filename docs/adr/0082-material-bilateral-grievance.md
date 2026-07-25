# ADR 0082: Material bilateral grievance

## Status

Accepted

## Context

Bilateral hostility existed only as an exogenous journaled command: an operator flipped the relation and the procurement layer obeyed. Nothing inside the executable economy could generate hostility, so conflict had no material cause and the political layer stayed decorative. At the same time the procurement layer already produced the raw evidence a causal model needs: per-firm unmet input demand, the pristine cross-border offer book, and the route graph.

Adding free-form "tension" as an unexplained scalar would violate the project rule that authoritative state must be causally derived, bounded, deterministic, and replayable.

## Decision

The monthly procurement cycle now maintains a directed, bounded grievance level per (aggrieved, target) country pair, derived only from observed material facts of the same month:

- a country accrues grievance toward a foreign country when one of its firms ends procurement with unmet input demand while that country offered the good (pristine pre-settlement offer book, positive quantity, foreign seller) and a direct logistics route could deliver it in peace;
- route existence is evaluated ignoring hostility: an embargo that blocks a deliverable good entrenches grievance rather than erasing the evidence;
- pairs without fresh evidence decay toward zero and are removed at zero, so reconciliation is reachable and state stays minimal;
- levels are basis points bounded to [0, 10000]; accrual is +500 per month of evidence, decay is -250 per month without it;
- every level change is journaled as `BilateralGrievanceChanged`;
- crossing 7500 activates the ordinary journaled `set_country_hostility` transition, so emergent hostility uses exactly the same authoritative relation, event, and downstream behavior as commanded hostility.

Grievance is authoritative state: it is serialized, compared, fingerprinted, and covered by replay tests. `SIMULATION_VERSION` is bumped to 33.

## Consequences

Conflict now has an executable material cause: cross-border dependence plus shortage plus visible foreign supply produces grievance, sustained shortage produces hostility, and hostility that blocks deliverable goods entrenches grievance. Restored trade de-escalates deterministically.

Approximations retained: grievance observes only firm procurement (not household shortage), only direct routes (no multi-leg reachability), fixed accrual/decay/threshold constants, and no de-escalation of hostility itself once active. Hostility deactivation, household-driven grievance, and political mediation of escalation remain later vertical slices.
