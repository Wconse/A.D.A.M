# 0188 - A buyer who walks away blames nobody

## Status

Accepted.

## Context

ADR 0187 changed the shape of the failure rather than curing it: the world now
responds hugely to hunger and then falls over. The next question was why the
boom cannot hold, and the honest answer required looking at what the price
signal was actually made of.

Four things were wrong, and all four were the same kind of wrong. Each let a
fact enter the price that was not a fact about goods.

**Intermediate demand was invisible to the market.** A mill queueing for grain
was not counted as pressure on grain at all. Only households were seen, so the
price of an input never learned that industry wanted it.

**A refusal to pay was read as a shortage.** A buyer who looked at the price,
decided the input cost more than the output it would become, and walked away
left behind an `unmet` record indistinguishable from empty shelves. The price
then rose because someone had declined to pay it. That is a ratchet: the less
affordable an input became, the faster it climbed.

**An empty till was read as a shortage too.** A buyer with no money queued, was
counted, and lifted the price it could already not afford.

**And a neighbour was blamed for both.** `update_bilateral_grievances` accrued
resentment against a foreign supplier whenever demand went unmet, including when
the buyer had refused the price or had no cash. Two countries could drift into
hostility because a domestic bakery was broke.

The fixture that hid this was the plainest evidence of all. Both grievance tests
were built on a cashless bakery - "The cashless bakery could not buy the grain
the foreign farm offered" - so the suite had been proving, in green, that a
buyer who cannot pay has been wronged by someone abroad.

## Decision

A shortage moves a price only when it is a shortage of goods.

- **Intermediate demand is shown to the market.** `observed_market_pressure` now
  reads three sources: seller order books, `monthly_firm_procurement_purchases`,
  and firm input shortfalls. Industry queueing for an input is pressure on that
  input.
- **A buyer will not pay more than the input is worth to it.**
  `input_value_ceiling` prices one batch of the output, subtracts the other
  materials, and divides by the input consumed. Above that price the order is
  refused and flagged `refused_on_price`. Wages are deliberately not subtracted -
  see the gaps below.
- **Only effective shortage is priced.** The settlement distinguishes `unmet`
  from `effective_unmet_by_market`. A shortfall counts toward price only when the
  route was not exhausted, the buyer did not refuse on price, and the buyer was
  not out of cash. The world carries this in
  `monthly_firm_effective_input_shortfalls`, and pricing reads that field, not
  the raw one.
- **Grievance rests on goods that were not there.** Three skips stand in a row
  in `update_bilateral_grievances`: a route at capacity, a refusal on price, and
  an empty till. None of them is anybody's fault abroad.

The grievance fixture was rebuilt to match its own claim: a funded bakery with
capacity for two batches against a foreign farm that grows one. The shortage is
now physical, and the tests measure what they say they measure.

## Consequences

Over fifty years (version 115, fingerprint 9829297342219149187, 233 tests) the
price signal is no longer self-feeding, and the money circuit leaks far less.
Total money grows by 118227 over the run (3460015 to 3578242) where version 114
grew by 410363 - the ratchet had been manufacturing nominal value out of
unaffordability. Treasuries end holding 81023 rather than 8462 and owners 106850
rather than 34962, while households end with 1006238 against 1238463: with
prices no longer climbing away from buyers, less nominal cash accumulates in
household hands. Deaths are flat at 3126 against 3132.

The boom is smaller and honest. Output peaks at 1230331 rather than 1769829,
because part of the old peak was priced on demand nobody could pay for.
Seventeen supply-growth reports replace twenty-one.

And the collapse is still there. Output falls to zero after the peak, twitches
once (6242, then 3131) and stays dead for the last thirty years. Nine "could not
procure" reports remain. Entry is still refused at twelve `RaiseExceedsWorkValue`
and twelve `NoSolventFounder` per review. Removing the false price signal did not
close the circuit; it uncovered what was underneath it.

What is underneath is now visible and is the next slice. The variable cost floor
derived from wages lifts an input's price to the floor within a single month -
grain goes from 5 to 50 with a wage of 50 - which takes it above what the finished
good is worth to its buyer, and the chain stops. Meanwhile the worker paid that
same wage cannot buy the output that wage priced. Both fixtures had to be brought
into agreement with this before they would pass honestly: bread at 10 became 60
in `b2b_procurement_capacity`, and 10000 in `bilateral_grievance`. A fixture that
must be made rich to keep a chain alive is telling us the chain is not viable at
ordinary prices.

### Known gaps

- **Wages are not subtracted from the input value ceiling.** A buyer will pay up
  to the full revenue of the output minus other materials, ignoring the labour
  it must also fund. The ceiling is therefore too generous, deliberately, until
  the wage question below is settled.
- **The wage-derived variable cost floor closes the chain in one month.** 25 per
  unit at 2000 milli output, 50 at 1000. Price reaches the floor immediately
  rather than approaching it, and the floor can exceed the downstream good's
  worth. This is the main unsolved mechanic.
- **Price sensitivity is asymmetric.** `SHORTAGE_SENSITIVITY_BPS` is 2500 against
  `SURPLUS_SENSITIVITY_BPS` 700: a price climbs more than three times faster than
  it slides back. This is why a settled grievance still takes forty-two months to
  decay to nothing.
- **Firm entry still reads the raw shortfall.** `record_procurement_outcomes`
  writes `monthly_firm_input_shortfalls` "whatever its cause", and entry
  evidence reads that field rather than the effective one.
- **The price signal appears in the buyer's region, not the producer's.**
  Bakerton moved 38 to 3 while Farmland held 5 to 1.
- Total money still grows, most likely through government borrowing creating
  deposits. Carried from 0184.
- Capital spending is still not counted as a cost when earnings are computed.
  Carried from 0186.
- Employment agreements stop being active after annual closure. Carried from 0185.
- Entry still picks the first recipe producing a good. Carried from 0183.
