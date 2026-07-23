# ADR 0078: Realized-trade macroeconomic closure

## Status

Accepted — 2026-07-23

## Context

The producing Stage 0 world kept firms, employment, output, and household markets alive for fifty years, but regional annual output still followed an independent seeded growth equation. A region could therefore report growing output and support public borrowing even when no firm produced or sold anything. This left the executable simulation chain open between microeconomic activity and fiscal/political closure.

## Decision

Economic years now derive each region's annual output from the monetary value of settled local market fills across their twelve completed monthly cycles. Seller identity resolves the region, and fill spend is accumulated with checked integer arithmetic. The annual regional output event and the country fiscal plan use that realized value.

The old stochastic output-growth planner remains available only to the legacy `advance_one_year` harness. `advance_economic_year`, which powers the CLI and full simulation, uses material closure.

The reported output growth rate is the bounded change between the previous annual output and realized current output. Initial example-world output values are calibrated to the first realized-trade closure to avoid a fake political shock caused solely by changing accounting bases.

## Consequences

- No settled trade means zero regional output and zero fiscal revenue/spending base for the economic year.
- Production, payroll, household resources, demand, market settlement, regional output, fiscal balance, debt, and politics now form one executable causal chain.
- Unsold production is not counted as output in this first closure; inventory investment and intermediate consumption remain future accounting work.
- Payroll affects output indirectly through household resources and demand rather than being double-counted as separate output.
- Cross-region trade, taxes by type, government procurement, and proper value-added accounting remain planned.

## Verification

- An economic-year test proves regional output equals the sum of its twelve months of settled fill spending.
- A no-trade economic-year test proves phantom output and fiscal activity disappear.
- Seed-47 one- and fifty-year CLI audits retain active production/trade while regional output and fiscal capacity remain bounded by those markets.
