# ADR 0181: A queue at the gate is evidence for the supplier too

- Status: accepted
- Date: step B10
- Supersedes a wrong claim recorded in step B9 of `docs/roadmap.md`

## Context

Step B9 closed the substitution loophole and left the roadmap with a diagnosis
that turned out to be false. The claim was that "nothing in the world expands an
existing firm's `capacity_batches` at all". That is not true. `capacity_investment.rs`
has always let a sold-out firm buy more capacity out of its own cash: it plans an
expansion from the firm's own monthly evidence, commits the money, launches an
investment project that takes `CONSTRUCTION_MONTHS` to build, and pays the budget
out to the households that build the works.

The mechanism was never the problem. The evidence was.

`plan_capacity_expansion` reads a firm's own trading month through
`ProductionAdjustmentProposal`, and the two fields that decide whether a firm has
a reason to grow are `stockout_observations` and `average_unmet_market_demand`.
Both are derived from `MarketOfferOutcome`, and in particular from
`sold_out_while_demand_remained`: the seller emptied its shelf **and** somebody
still wanted the good.

Household clearing in `market.rs` fills that in honestly. It counts the buyer
demand that left the market unserved and hands it to the sellers in that market.
Firm-to-firm procurement did not. In `record_procurement_outcomes` the seller's
row was written as:

```rust
offered: quantity,
sold: quantity,
unsold: QuantityMilli::default(),
unmet_market_demand: QuantityMilli::default(),
```

The seller was told what it sold and nothing else. A grain farm with a queue of
bakeries at the gate read its month as "sold everything, nobody left wanting",
scored `stockout_observations = 0`, and was filtered out of expansion on the very
first test, `stockout_observations <= unsold_observations`.

That is a magic restriction, and an asymmetric one. A baker who sold out to
hungry households got a reason to build another oven. A farmer who sold out to
hungry bakers did not. Which industries in this world are allowed to grow
depended on **who their customer happened to be**, not on any physical or
financial fact. Intermediate goods - which is to say the whole upstream of the
economy - were structurally forbidden from answering their own shortage.

This is exactly the defect that made the demo world starve. Bakeries could not
procure grain for years on end, the chronicle repeated
"could not procure 2762000 milli-units of Grain", and the farms that were selling
out every single month never once saw a reason to plant more.

## Decision

A buyer that leaves a market empty-handed is the same evidence no matter whether
that buyer is a household or a firm. Procurement now records it, under two rules
that keep the signal honest.

1. **The unserved order belongs to the market that wanted the good.** Settlement
   accumulates every unfilled remainder into `unmet_by_market`, keyed by
   `(region, good)` - the same key the household clearing uses, and the buyer's
   own region, so an import that failed is counted where the demand lived.

2. **Only a seller that actually ran out may claim it.** Settlement records
   `exhausted_sellers`: the `(firm, good)` pairs whose entire offer was taken
   during the month. A seller still holding stock on the shelf did not turn
   anybody away, and reads `unmet_market_demand = 0`.

Rule 2 is what stops this from becoming a free growth coupon. Without it every
seller in a tight market would inherit the whole queue and expand on somebody
else's shortage.

No constant was introduced, no threshold was tuned, no number was scaled. The
signal that households have always handed their suppliers is now also handed to
suppliers of suppliers.

## Consequences

- A farm that sells out to mills can now reach `sold_out_while_demand_remained`,
  earn `stockout_observations`, carry a positive `average_unmet_market_demand`,
  and therefore be considered by `plan_capacity_expansion` on the same terms as
  a bakery selling to households.
- The whole upstream of any production chain becomes capable of answering its own
  shortage. Whether it *does* answer is now decided by solvency and margin, which
  is where that decision belongs.
- `unmet_market_demand` enters the world fingerprint through `world.rs`, so this
  changes replay. `SIMULATION_VERSION` moves 106 -> 107.
- Seller rows for procurement still carry `unsold = 0`. The same physical
  inventory is offered to firms in procurement and to households in the clearing
  later in the month; writing a procurement "unsold" would double-count the same
  stock against the seller and suppress expansion for the opposite wrong reason.

## Tests

Both directions are locked in `crates/adam-core/tests/intermediate_procurement.rs`:

- `a_farm_that_sold_out_with_a_mill_still_waiting_reads_the_queue`: a second
  bakery wants more grain than the farm can grow; the farm's observation carries
  a positive `unmet_market_demand` and reports `sold_out_while_demand_remained`.
- `a_farm_with_grain_left_on_the_shelf_does_not_claim_the_queue`: two farms sell
  into one market while a cashless bakery leaves empty. The farm that emptied its
  shelf reads the queue; the farm still holding grain reads zero.

## Measurement

Fifty years, seed 1, `SIMULATION_VERSION = 107`, fingerprint
`11683835997810038817`, 146.6 ms per year.

The chronicle now reports supply growth per year, and the honest result is that
the economy still dies. Across fifty years the world launched **two** capacity
projects - one in 2027, one in 2033 - and founded **zero** new firms. Output still
falls to nothing and the regions still empty out.

This ADR removed a restriction; it did not rescue the run, and it was not
supposed to. The queue is now visible to the farms. What they do with it is
decided further down `plan_capacity_expansion`, and that is where the next slice
looks.

## Known gaps

- **A firm may only grow out of its own till.** `plan_capacity_expansion` caps
  the purchase at `affordable = firm.cash() / batch_cost`, where
  `batch_cost = quantity_value(price, output_per_batch) * CAPACITY_VALUE_MULTIPLIER`.
  A farm that is selling out every month is precisely a firm whose cash is thin
  and whose prospects are good, and it cannot borrow a single unit against them,
  even though the world has financiers and a credit market. Investment finance is
  the strongest remaining candidate for a magic restriction on the supply side.
- Founding still discharges regional pressure to zero and rebuilds it over
  `ENTRY_SIGNAL_BPS` months, so entry answers a famine one one-worker firm at a
  time (recorded in step B9).
- A converter still cannot defend its input by bidding what that input is worth
  inside its own recipe (from ADR 0180).
- Grain still has no seed cost, and households still produce nothing.
