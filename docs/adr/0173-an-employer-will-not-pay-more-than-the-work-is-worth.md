# 0173. An employer will not pay more than the work is worth

## Status

Accepted.

## Context

The fifty-year chronicle showed an economy that starves from the first year and
stops producing entirely by 2047. Two earlier slices gave firms the power to
found themselves and to buy capacity when scarcity paid for it, and neither ever
fired in the demo world. The reason was the same in both cases: no firm in that
world earns a positive operating margin, so every profitability test correctly
refused. Cheap capacity was never the binding constraint.

Tracing the loss back reached the wage. A hire was priced by
`competitive_labor_bid`, which takes what the cohort used to earn per person per
month, marks it up by how urgently the firm wants staff, and adjusts it by
vacancy or unemployment pressure in the region. Every term in that expression
describes the worker's side of the bargain. Nothing in it describes the
employer's side.

A labor market with only one side is not a market. The only check standing
between a firm and a ruinous hire was whether it had the cash to pay this
month's wage, which any firm has right up until it does not. So firms hired at
whatever the going bid happened to be, produced goods worth a fraction of their
payroll, bled cash, cut production, and took the food supply down with them.
The divergence from life was not a wrong rule; it was a missing half of a loop.

## Decision

A firm now refuses a hire whose wage exceeds the value of the work bought.

The ceiling is computed from two quantities the world already observes, not from
a tuned parameter. A batch is worth its output at the local observed price,
less the local observed price of every input the recipe consumes. The recipe
states how much labor a batch takes in milli worker-months, so the value of one
worker-month follows directly from that margin. When the output good or any
input has no observed local price, there is no evidence to judge the hire by and
the ceiling does not apply.

When the bid exceeds the ceiling, the vacancy stays open. It is not filled at a
lower wage, because the wage is set by the worker's side of the market and the
firm has no power to dictate it. The unfilled vacancy continues to register as
vacancy pressure, which is exactly what it is.

This bound is owned. The party paying for it is the firm: hiring above the
ceiling loses it money on every wage payment, and that loss is what makes the
refusal rational. The party bearing the consequence is the worker, who stays
unemployed because the work available is not worth what they are asking. Both
costs are real and both are visible in the world state.

## Consequences

Work whose product does not cover its wage does not happen. Regions can now
carry unemployment that is genuinely structural rather than frictional: the jobs
are not there because at the going wage they would destroy value.

The two dormant slices before this one gain a path to firing. Once firms stop
hiring into loss, positive operating margins become reachable, and the capacity
investment and firm entry stages have something to act on.

Profitability now depends on the price of output relative to the price of
inputs and the productivity written into the recipe. That is the correct set of
dependencies, and it makes recipe productivity a live economic quantity rather
than a bookkeeping detail.

## Known gaps carried forward

- Existing employment agreements are untouched. A firm already saddled with a
  wage it cannot cover keeps paying it; there is no layoff, no wage
  renegotiation, and no orderly wind-down driven by the same arithmetic. This is
  the next opening in the loop and the demo world is full of exactly these
  legacy agreements, which is why its fingerprint is unchanged by this slice.
- Poaching, retention, and job switching are validated against the competitive
  bid but not against the value ceiling.
- The demo content never enables the labor market for its recipes, so its
  staffing is still the static roster written into the scenario.
- The ceiling values a worker at the average product of labor in the recipe
  rather than the marginal product, which is the same number only because
  recipes are linear today.
