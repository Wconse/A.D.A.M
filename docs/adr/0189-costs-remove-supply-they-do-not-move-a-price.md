# 0189 - Costs remove supply, they do not move a price

## Status

Accepted.

## Context

ADR 0188 removed four facts from the price that were not facts about goods, and
what it uncovered was a fifth: a floor. `revised_price` compared the reference
price of a good against the variable cost of the cheapest solvent local producer
and refused to let the reading go under it. Below cost, the price jumped to cost
in a single month; at cost, no glut could cut it.

The rule in `AGENTS.md` asks one question of every limit: who is holding this
line, and what does it cost them? This floor had no answer. It lived inside a
measurement. No firm chose it, no firm funded it, and nobody could stop paying
for it and let the line break. Meanwhile the seller, who is the only party with
standing to refuse a price, had no say at all: `plan_firm_market_offers` set
`unit_price` to the reference price times the markup and offered the goods
whatever the cost of making them.

So the model had it exactly inverted. A cost was moving a price, which nobody
can do, and a cost was not removing supply, which is the only thing a cost
actually does.

The consequences were visible in the fifty-year chronicle. Grain in `demo.toml`
went from 5 to its wage-driven cost of 50 within one month, an input became
dearer than the bread it becomes, entry was refused twelve times because the
raise exceeded the value of the work, and output died after the boom.

## Decision

Two changes, in this order, so that the world was never left without either
mechanism.

**A seller does not sell below its own variable cost.** A new
`firm_variable_unit_cost` prices one whole unit of a firm's own output: the
inputs it must replace at the local prices of its own region, plus the wage it
has actually agreed to pay, divided by the batch it gets out. It returns `None`
when an input has no local price, because a cost nobody can price is a cost
nobody can defend a refusal to sell with. `plan_firm_market_offers` now raises
its asking price to that cost when the going rate is lower.

This limit has an owner and a bill. The firm holds the line, and it pays with
the stock that stays on its shelf, the revenue it does not take, and the payroll
it still owes out of the same till. It can hold out only while its cash lasts,
and when the cash runs out it answers with arrears, shedding, or insolvency -
all mechanisms that already exist and already have owners.

**Costs leave the price formation entirely.** `revised_price` no longer takes a
cost floor. Neither the jump-to-cost nor the never-cut-below-cost clamp remains,
and `cost_floor` and `floor_binding` are gone from `RegionalPriceAdjustment` and
from `DomainEvent::RegionalPriceAdjusted`. `local_variable_cost_floor` was left
with no callers and was deleted rather than kept as decoration. A glut now cuts
a price straight past what the good costs to make, and what does not happen is a
sale at that price: the producer withdraws instead, and the withdrawal reaches
the next month's reading as supply that never arrived.

## The fixtures were the evidence

Six tests failed the moment sellers stopped selling at a loss, and every one of
them failed for the same reason: the fixture had been built on a firm that lost
money on every unit and never noticed.

In `cross_region_procurement`, a farm paid a worker 50 a month to grow one unit
of grain priced at 5. That wage was ten times the value of the work by the
model's own `labor_value_ceiling`, so the chain traded only because somebody was
selling at a tenth of cost. The wage is now 5, which is exactly what the work
earns.

In `intermediate_procurement`, the same wage of 50 sat against grain at 5 and
bread at 10. Repricing the goods to cover the wages was not enough on its own:
at bread 60 the two wages of 50 in that world could not buy back the bread they
baked, and the chain jammed in the second month. Bread is now priced at 50,
where two loaves cover the grain and the baker's month exactly, and the two
wages buy back exactly what is baked. The circuit closes at zero profit, which
is the honest steady state for a world whose only income is wages.

Two tests had premises that were themselves the magic.
`a_loss_making_firm_refuses_to_buy_capacity` had both firms losing money purely
because wages exceeded revenue; the loss is now earned, from bread too cheap to
recover the grain, and the test watches the firm that actually loses.
`inventory_investment_is_valued_at_observed_prices_not_reference_prices` had
assumed twelve marked-up grain trades a year; it turns out a 20% markup cannot
be realised at all in a world where wages are the only income, so the baker buys
once and then walks away from the counter. That is recorded as a finding, not
patched over.

## Consequences

Measured over fifty years (version 116, fingerprint 2108872914777405248, 234
core tests):

Money nearly stops being manufactured. The stock grows by 37140 (3460015 to
3497155) against 118227 in version 115 and 410363 in version 114. The floor had
been a nominal-value press: it lifted reference prices on nobody's authority,
and every good in the world was revalued upward from it.

The wage obstacle to entry disappears completely. `RaiseExceedsWorkValue`, which
blocked twelve openings in version 115, now blocks none, because prices are no
longer shoved above the worth of the work by a floor. `NoSolventFounder` is also
zero. What blocks entry now is that there is no worker to be had - 48 openings -
which is a real constraint with a real owner.

Deaths fall slightly, 3068 against 3126.

And the collapse gets worse, not better. Output dies by 2034 rather than 2043
and stays at zero for forty years. This is the honest reading, and it names the
next slice precisely. With the floor gone, a reference price learns only from
pressure - who sold out, who was left holding stock - and never from the prices
at which trades actually happened or were asked. So the reference drifts toward
1 while sellers ask their real costs, nobody's offer is accepted, supply
withdraws, and the drift accelerates. Every consumer of the reference price -
`input_value_ceiling`, the valuation of entry, procurement - is then reading a
number that no trade in the world supports.

That is not an argument for restoring the floor. The floor was a wrong answer to
a real question, and the question is still open: a price should learn from what
was actually paid and actually asked. That is the next slice.
