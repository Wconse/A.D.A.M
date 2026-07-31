# ADR 0140: Housing-constrained paid internal migration

## Status

Accepted.

## Context

Step 091 allowed a solvent household partition to move toward durable vacancies and public services, but destination settlement had no physical housing limit and moving consumed no service. This allowed attractive regions to absorb households indefinitely and made relocation financially frictionless.

## Decision

Every registered region owns a persistent housing-market record containing dwelling capacity and a positive baseline monthly housing cost. Legacy content derives one dwelling per two initial residents and a bounded cost proxy from initial output per resident; scenarios and later systems may replace those assumptions explicitly. Housing state is serialized and fingerprinted.

A migration candidate is rejected when one additional household would exceed destination dwelling capacity. Projected occupancy becomes a 0-10,000 basis-point housing-pressure signal. Pressure lowers destination attractiveness and raises a mandatory relocation and registration fee from 50% to 150% of the destination baseline monthly cost.

The migrant must retain the existing one-month income reserve and separately afford the fee. The fee is debited from the split migrant cohort and credited to the destination country's treasury after annual fiscal closure, conserving money exactly and representing real administrative, registration, and settlement services. Typed migration evidence retains the fee and projected housing pressure.

## Consequences

Employment opportunity, public services, household liquidity, housing scarcity, and state capacity now meet in one decision. High-pressure regions remain attractive when opportunity is strong but become more expensive, and physically full regions refuse additional migration. Public revenue rises only when a move actually occurs.

The model still aggregates dwellings and treats the state as the relocation-service provider. It omits landlords, construction, vacancy quality, homelessness, rent paid every month, route travel, informal housing, and regional zoning. The next useful gate is endogenous housing construction financed by observed fees, pressure, and firm investment rather than automatic capacity growth.
