# 0089. Blended local-first survival budgeting

## Status

Accepted (step 040).

## Context

Step 039 quoted either local supply or an import. When local supply covered only part of a survival target, a household could reserve the local price for the entire need even though the market would consume the local supply first and require a more expensive import for the remainder.

## Decision

Survival planning now quotes the target as a deterministic local-first basket: all local offers by price and seller, then peaceful direct imports by delivered price, source region, and seller. Each source segment is rounded up when converting quantity and unit price into the maximum spend required by market affordability; actual market settlement remains rounded down. Any unquoted remainder uses the regional reference price, preserving prior no-supply behavior.

`PricedTarget` carries the quoted basket cost directly, and budgeted quantity is derived proportionally from that cost. This preserves normal single-price behavior while allowing a true multi-source reserve.

## Consequences

A household can reserve 12 minor units for 500 milli-units local at 10 plus 500 milli-units imported at 13: 5 + ceil(6.5) gives a sufficient order budget, the market buys both portions, and settles 11 actual minor units. The full gate passes with 115 tests; version 34 and release fingerprint `12100901864703017553` remain unchanged.

Known limits: source capacity is quoted independently per household before canonical market allocation, so simultaneous cohorts can still overquote shared supply; relief costs remain reference-priced; and routes remain immediate accounting links rather than capacity-constrained shipments.
