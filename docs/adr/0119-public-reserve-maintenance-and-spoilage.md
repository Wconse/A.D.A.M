# ADR 0119: Public reserve maintenance and spoilage

## Status

Accepted for Stage 0.

## Context

Coverage targets made preparedness a material choice, but stored goods remained perfect and free forever. A government could therefore accumulate an arbitrarily deep buffer without consuming treasury capacity or losing physical stock. This removed the central tradeoff between resilience now, fiscal room later, and the institutional ability to preserve what was purchased.

## Decision

Government reserve policy now carries two independent monthly storage parameters:

- baseline physical spoilage in basis points of opening stock;
- carrying cost in basis points of opening stock valued at the current regional reference price.

Maintenance executes once at the beginning of each economic month, before payroll, commerce, new reserve procurement, and emergency release. Only stock carried from a prior month is charged and degraded; goods purchased later in the same month first incur maintenance next month.

The carrying-cost assessment is aggregated by country. When treasury cannot fund every stored good, available cash is allocated proportionally across all assessments using deterministic largest-remainder allocation, preventing canonical good order from silently protecting one reserve while abandoning another.

Baseline spoilage is always physical. If an assessment is underfunded, the unpaid share produces additional neglect spoilage, linear up to 2,500 basis points when nothing is paid. This loss is applied after baseline spoilage. Treasury payments are real sinks and destroyed stock leaves the authoritative reserve map.

Each maintained regional-good stock emits opening quantity, reference value, assessed and paid cost, baseline spoilage, neglect spoilage, and closing quantity. A completion event aggregates the monthly stage. The ordinary command surface can replay maintenance exactly, and the yearly chronicle explains both ordinary loss and destruction caused by unfunded upkeep.

## Invariants

- Maintenance never creates money or physical stock.
- Paid carrying cost cannot exceed assessment or available treasury.
- Scarce treasury is shared proportionally within a country; deterministic remainders use canonical reserve order.
- Baseline and neglect spoilage cannot exceed opening stock in total.
- A fully funded assessment creates no neglect spoilage.
- A zero-cost policy preserves prior behavior.
- Newly procured stock is not charged retroactively in its purchase month.
- Duplicate execution is rejected atomically.
- Maintenance date, policy rates, stock, treasury, events, replay, saves, and fingerprints remain deterministic.

## Consequences

A reserve worth 10 minor units under 10% monthly carrying cost and 10% baseline spoilage pays 1, loses 100 of 1,000 milli-units, and closes at 900 when treasury is available. With no treasury, the same stock additionally loses 225 milli-units through maximum neglect and closes at 675. When two equally assessed goods compete for half the required treasury, each receives half the maintenance funding and bears the same proportional neglect loss.

`SIMULATION_VERSION` advances from 57 to 58 because storage policy, monthly stage state, treasury transitions, and physical loss are authoritative. Focused coverage proves funded upkeep, maximum neglect, proportional multi-good funding, next-month charging, command replay, duplicate-stage atomicity, and chronicle narration. The full workspace passes 161 tests; two seed-1 50-year timelines produce identical chronicles and fingerprint `18007562743465462498` at 80.8 ms/year.

Deliberate limits: reference prices proxy storage complexity; there are no warehouses, reserve age lots, good-specific shelf lives, rotation sales, theft, or regional transfer. The next gate should make reserve policy adapt to observed losses and fiscal strain before introducing public logistics.
