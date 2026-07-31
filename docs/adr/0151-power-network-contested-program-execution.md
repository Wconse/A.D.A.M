# ADR 0151: Power-network-contested program execution

## Status

Accepted for Step 100.3b.

## Context

Administrative capacity alone makes execution impersonal. Existing actors, home regions, political offices, and influence edges already represent concrete power. A program that favors or excludes those bases should receive support or friction without turning politics into a random veto.

## Decision

- At execution, inspect established influence edges into political offices of the program country.
- Compare each influencing actor's home-region program share with an equal domestic reference share.
- A favored base contributes bounded support; an underrepresented or excluded base contributes bounded opposition. The effect scales with the existing influence weight.
- Sum political effects into a program-wide execution modifier bounded to ±2,000 basis points.
- Compose the modifier with administrative absorption and clamp final execution capacity to 0-100%.
- Emit actor-, office-, region-, stance-, weight-, share-, and modifier-attributed evidence.
- The authorized execution command always proceeds when resources and timing are valid; opposition changes realized delivery and carryover rather than silently vetoing the command.

## Invariants

1. Influence creates neither money nor service capacity above 100%.
2. Political opposition cannot make delivered funding negative.
3. Only domestic actors influencing actual domestic political offices participate.
4. Equal or zero-effect relationships create no noise events.
5. Explicit program authority remains separate from execution support.
6. Canonical maps and bounded integer arithmetic preserve replay determinism.

## Consequences

Patronage can speed execution while exclusion produces administrative friction and longer carryover. The mechanism remains visible and attributable instead of becoming an artificial prohibition. `SIMULATION_VERSION` advances 88 -> 89. Physical goods, temporary labor, infrastructure, and logistics remain required before Step 100.3 is fully complete.
