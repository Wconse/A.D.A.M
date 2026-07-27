# ADR 0098: Carrier-Funded Route-Capacity Expansion

## Status

Accepted

## Context

Immediate household and firm-procurement imports now share an authoritative monthly route-capacity pool and pay real freight revenue to registered carriers. Capacity shortages were therefore observable and carriers had liquid operating income, but route capacity was still static. An unconditional infrastructure rule would have expanded routes without proving demand, and the existing generic `InvestmentProject` cannot target a logistics route or mutate its authoritative capacity.

## Decision

A route records one month of pressure only when market matching proves that the remaining route limit is lower than otherwise feasible demand: remaining demand, seller stock, and buyer affordability. B2B and household matching expose route-specific transient evidence, which the commercial cycle unions after both have consumed the shared capacity ledger.

Pressure persists in `World::route_capacity_pressure` and is part of the stable fingerprint. It advances only when the same route also produced positive typed `MarketFreightPaid` or `FirmProcurementFreightPaid` revenue on the current simulation date. An unconstrained or revenue-free month clears the route's streak. The streak is capped at three while financing is unavailable.

After three consecutive qualifying months, a registered route carrier may fund one bounded expansion from cash:

- added capacity is the ceiling of 10% of current monthly capacity, with a one-milli-unit minimum;
- construction cost is twelve months of route tariff revenue on the added capacity, rounded upward, with a one-minor-unit monthly minimum;
- the carrier must hold the full cost in cash; no bank debt or invented lender is introduced;
- successful construction debits the exact cost, increases the route's authoritative capacity, clears pressure, and appends `RouteCapacityExpanded`;
- insufficient cash leaves capacity and cash unchanged and retains pressure at the threshold for reconsideration next month.

The response runs after B2B and household settlement, when current freight payments and carrier cash exist, but before monthly firm observations and account reset. Existing replayable commercial and economic cycle commands therefore cover the response without a separate command.

Construction spending is modeled as liquid cash converted into installed route infrastructure. Stage 0 does not yet represent construction suppliers or a separate infrastructure asset ledger, so this cost intentionally leaves the liquid-money stock. The expansion event reports the exact amount, and tests prove that the carrier cash decrease equals installed-capacity cost rather than an accidental settlement imbalance.

## Consequences

- Route capacity can now grow from repeated, route-specific, economically feasible demand rather than simple utilization.
- Carriers must demonstrate positive actual freight revenue and self-finance the response.
- Every route consumer observes the expansion because the route itself remains the single authoritative capacity location.
- Failed financing and arithmetic errors are atomic inside the cloned commercial-cycle transition.
- Typed expansion evidence enters replay, persistence, stable fingerprints, and the yearly chronicle.
- Tests prove pressure accumulation and reset, insufficient-cash behavior, exact 10% capacity growth, exact cash debit, event evidence, direct/replayed equality, and stable fingerprint equality.
- `SIMULATION_VERSION` advances from 36 to 37. Full workspace validation passes 128 tests. The seed-1 50-year fingerprint is `8785785093042010742`; release runtime is 27.4 ms/year.

## Follow-up

Later stages may introduce construction industries, explicit infrastructure assets, depreciation, debt finance, competing carriers, and investment appraisal from margins rather than tariff revenue alone. Those additions should preserve this route-specific demand evidence and authoritative-capacity mutation seam.
