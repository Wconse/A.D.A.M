# 0180 — Substitution is a forced measure, not a free trick

## Status

Accepted. Simulation version 106. Content schema version 9.

## Context

The demand side had two faults that hid each other.

**A household budgeted for food that nobody was selling.** `price_targets`
asked the offer book how much of a survival target could actually be bought,
and then, for the part the book could not cover, quoted the *regional reference
price* and added it to the bill anyway. A reference price is a statistic about
last month's trades; it is not a seller. So a cohort in a region with an empty
market still reserved money against a full ration, reported a plausible spend,
and only discovered the emptiness at clearing. Money was frozen against goods
that did not exist, and the market's own emptiness never reached the buyer's
plan.

**A need was measured in kilograms, so substitution would have been a bug.**
`ConsumptionTarget` already carried `need_group` and `preference` — the fields
existed, were validated, and were hashed into the fingerprint — but nothing
read them. `price_targets` treated every target in a tier as an independent
line item, so two ways of meeting one need would have been billed as two
separate needs and a household would have been required to buy both. Worse,
`survival_fulfillment_bps` divided consumed kilograms by demanded kilograms.
Under that arithmetic a food a person needs *more* of scores as the more
nourishing one: 1.8 kg of raw grain, which is a worse meal than 1.0 kg of
bread, would have counted as 1.8 times the nutrition. Substitution would have
been a free trick — swap to the bulkier food and the same market feeds more
people.

Neither fault could be fixed alone. Letting a household reach for a substitute
while fulfilment is weighed in kilograms rewards eating badly; measuring the
need correctly while the reference price still funds absent goods means the
household never has to reach for anything.

## Decision

### A need is a share of itself, not a weight of goods

Targets that share a `need_group` within a tier are ways of meeting one and the
same need. Each target's `monthly_quantity` is the amount of *that good* which
closes the whole need on its own. A worse food therefore asks for more of
itself for the same share of the need, and this is stated in content rather
than computed by the engine.

`price_targets` now walks each need through its substitutes in `preference`
order, carrying the share of the need that is still open in fixed point:

1. the preferred good is asked for the open share of its full ration;
2. the offer book answers with what can actually be delivered — local offers
   first, then peaceful direct imports within the remaining monthly route
   capacity;
3. the share that delivery closed is subtracted, and whatever is still open
   passes to the next preference, which quotes **its own, larger** ration at
   **its own** price;
4. a closed need asks nobody else, so a household that can buy bread never
   reaches for raw grain.

### An unsold quantity is not funded by a price with no seller behind it

`household_target_cost` became `household_target_purchase` and returns both
what can be had and what it costs. The part nobody offers costs nothing and is
bought in no amount. `DemandIntent` now separates what the household *set out*
to buy (`desired`, which still drives the market order and the affordability
gap that relief reads) from what it can actually pay for (`budgeted`). Hunger
stays fully visible as unmet demand and as a price signal; the money simply
stops being reserved against an absence.

The one case that keeps the reference price is planning with no offer book at
all — `monthly_survival_cost` and non-survival tiers. There is no evidence of
scarcity there, so nothing constrains the ration. Absence of evidence is not
evidence of plenty, but it is also not a limit, and inventing one would be the
very thing this project forbids.

### Fulfilment counts closed needs

`survival_fulfillment_bps` reads the cohort's own profile instead of the trades
it happened to make. For every survival need it sums, across that need's
substitutes, the fraction of a full ration each good actually delivered, caps
the result at one — eating two rations does not feed a household twice, which
is the domain of a ratio and not a policy — and averages across needs.

Two consequences fall out of this and are pinned by tests. Half a bread ration
plus half a grain ration feed a cohort exactly once. And the weight of a full
bread ration in raw grain feeds only 55.55% of the month, because raw grain is
the worse food. Substitution is now what it is in life: something you do
because the good you wanted is gone, and you are worse off for it.

A third consequence is a correction rather than a design choice: a cohort that
bought nothing is now hungry whether or not it ever placed an order. The old
ratio divided met by met-plus-unmet, so a household with no recorded demand
scored full fulfilment on an empty stomach.

### Content schema v9 states the substitution

`TargetSpec` gained `need_group` and `preference`, both defaulted, so v8
scenarios describing goods that cannot replace one another parse unchanged.
The demo now states one need — being fed — with two ways to meet it: bread at
1.0 per person, and raw grain at 1.8. The 1.8 is not a penalty coefficient; it
is the statement that unbaked grain feeds a person worse. Since the demo baker
turns 1.0 of grain into 2.0 of bread, a region that eats its harvest raw burns
3.6 times the grain that eating bread would have taken.

## Consequences

Measured on the 50-year demo run, seed 1:

| | before (ADR 0179, v105) | after (v106) |
|---|---|---|
| fingerprint | 15330073222654020652 | 15507271165589345173 |
| speed | 140.3 ms/year | 148.5 ms/year |
| grain demanded by households | none | competes with the bakeries |
| terminal state | regions failing, output positive | output reaches zero by 2045 |

The run got worse, and it got worse for a reason worth keeping.

Households that cannot buy bread now buy grain, and grain is the bakery's
input. From 2040 the chronicle names it directly: *Northreach Bakery could not
procure 2 762 000 milli-units of Grain*, then Southvale Bakery the year after.
A bakery starved of grain bakes less bread, households fall back to grain even
harder, and the loop tightens until the last mill stops. The demo economy is
eating its own supply chain.

This is a real mechanism, not an artefact: a starving region eating the grain
that should have been baked, or sown, is one of the oldest ways a famine makes
itself permanent. What is *missing* is the counterweight that exists in life. A
baker who turns one sack into two loaves can outbid a hungry family for that
sack, because the sack is worth more to the baker than to the eater. In the
current cycle the bakery buys its input on cash it earned last month, so when
households take its customers the bakery loses exactly the revenue it needed to
keep buying, and it cannot bid what the grain is worth to it. Nothing in the
model lets a converter defend its input against final demand.

Before this slice the same defect existed and was invisible: households
reserved money against bread that was not there, the shortfall showed up only
as unmet demand at clearing, and the grain never moved. The collapse is not new
behaviour caused by measuring honestly; it is the old behaviour with the
padding removed.

## Known gaps

- **A converter cannot defend its input.** Firms procure before households
  shop, but on last month's cash. The next slice should let a firm bid what an
  input is worth in its own recipe, judged by the same value-ceiling arithmetic
  ADR 0173 already applies to labour, and let it lose the bid honestly when a
  household is willing to pay more.
- **No household produces anything.** A cohort that cannot buy food cannot grow
  any either. Until subsistence exists, a market failure is a death sentence.
- **Grain has no seed cost.** Eating the harvest costs this year's meals only;
  nothing carries forward into next year's planting, which is precisely how the
  real version of this trap bites hardest.
- Substitution is stated per need in content, with no notion of a partial or
  imperfect substitute beyond "more of it is required".
