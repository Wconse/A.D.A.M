# 0177. A new employer outbids an old one

## Status

Accepted.

## Context

ADR 0176 gave a region a way to notice that it kept failing to buy something
and to treat that failure as a standing invitation to produce the good locally.
The invitation could not be accepted. Founding a firm required somebody
`Unemployed` in the region, so in a fully employed region a shortage could
shout its price forever and no producer could ever answer it. That was an
unowned limit in the sense of `AGENTS.md`: nobody paid for the rule that a new
employer must wait for a queue of idle people to form, and the economy was
quietly prevented from responding to its own prices.

In life a new employer does not wait for unemployment. It takes staff from an
existing employer by offering more than the work is currently paid, and the old
employer either answers the offer or loses the worker.

## Decision

A founder may staff a new firm by outbidding an incumbent employer.

- Idle labour is still hired first, at its own reservation wage. Nobody outbids
  an employer for a worker who is standing in the street.
- Failing that, the founder scans standing employment agreements in the region
  for a worker who is qualified for the recipe, is not in training, and has
  been at the current employer for at least three months, and whose employer is
  solvent.
- The offer is the incumbent wage improved by the same ten percent margin the
  labour market already uses elsewhere (`MATERIAL_OFFER_IMPROVEMENT_PERCENT`),
  rounded up, and it must be strictly above the standing wage.
- The offer may not exceed what the work is worth at local prices. The value
  ceiling of ADR 0173 was extracted into `recipe_labor_value_ceiling` so that a
  founder and an incumbent are judged by identical arithmetic.
- The founder takes the worker whose present employer values them least,
  because that is the cheapest hire it can defend paying.
- `FirmFoundingPlan` records `poached_from`, and founding releases the worker
  from the previous agreement before registering the new one.

Both sides are owned. The founder pays a strictly higher wage and must carry
three months of it as working capital before a single batch is sold. The
incumbent pays by losing a worker it had, and answers - or declines to answer -
through the ordinary retention machinery in the following month.

## Consequences

- Entry no longer depends on unemployment, so the ADR 0176 signal has a channel
  into production in a fully employed region.
- No new event was added. A poached hire is an ordinary employment change.
- The incumbent's reply is approximated by the existing monthly retention pass
  rather than by a same-month counter-offer.
- Simulation version is 104.

## Known gaps

- Capacity investment (ADR 0172) still hires only idle workers, so an existing
  firm cannot outbid anybody while a new one can.
- Retention answers a departure a month late; a real bidding war inside one
  month is not modelled.
- Entry is still limited to survival goods.
- Honest measurement: in the demo scenario this slice founded no firms. The
  reasons were diagnosed separately and are recorded in ADR 0178.
