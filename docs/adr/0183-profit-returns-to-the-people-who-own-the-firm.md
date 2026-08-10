# 0183 - Profit returns to the people who own the firm

## Context

ADR 0182 left a boom followed by a bust and one stubborn fact beside it: in
fifty years not a single new firm was founded. The entry observer added in this
slice named the reason year by year. Openings multiplied - 21 in the first
year, 48, 66, 72 - and none were ever feasible. Two obstacles shared the blame:
`RaiseExceedsWorkValue`, an employer refusing to pay more than the work is
worth (ADR 0173, deliberately untouched), and `NoSolventFounder`, rising from
10 to 48 refusals a year.

`NoSolventFounder` was the interesting one. A worker had been found and priced;
what was missing was anybody with money to sign for the venture. Founders are
sought among actors, and an actor's purse is `actor_cash`. Nothing in the world
ever filled it, because `policy.dividend()` was read in exactly one place in
the whole crate - the fingerprint hash. Firms accumulated profit for fifty
years and never sent a coin of it home. The owning side of the economy existed
on paper, drew income never, and therefore could never fund anything new.

A second, quieter defect sat underneath. `OwnershipStake::new` takes economic
rights before voting rights; the scenario loader passed them the other way
around. Every demo firm's economic holding was silently the voting number.
Since `execute_dividend` refuses to pay unless economic rights total 100%, the
first working distribution mechanism still paid nothing at all, and the run
looked identical to before. The swap was invisible precisely because no code
path had ever depended on economic rights.

## Decision

1. **A year of profit reaches its owners.** `execute_observed_owner_distributions`
   runs once per economic year, after the twelve monthly cycles and before the
   annual plans. For each firm it skips insolvency, a zero dividend share, a
   firm with no operating record, and ownership that does not add up to a
   whole. What remains is free cash: the till minus payroll, wage arrears,
   input commitments and investment commitments already promised. The firm's
   own dividend policy share of that surplus is paid through the existing
   `execute_dividend`, which splits it by economic rights and credits
   `actor_cash`. No new constant and no new threshold.

2. **Obligations are one idea, not a list of arguments.** The funding-gap code
   in `credit_market.rs` was split into `FirmObligationTerms` - payroll, wage
   arrears, inputs, investment, monthly sales, cash - with `obligation()`,
   `observed_surplus()` and `free_cash()`. The lender's gap is now a two-line
   consequence of those terms, and the boardroom reads free cash from the same
   place the lender reads it. One definition of what a firm owes, two readers.

3. **A scenario's economic share arrives as the economic share.** The loader's
   swapped arguments are corrected, and a content test pins both numbers apart
   so the two can never be traded again.

## Consequences

The circuit that was missing is now closed on the owning side: firms earn,
owners are paid, and paid owners can found firms. `NoSolventFounder` disappears
from the run entirely.

Over fifty years, 32 years carry distributions, 2840491 minor units reach
owners, and **152 firms are founded** where previously there were none - 1 in
2027, 2, 3, 5, 11, 13 as the money spreads. Excess deaths fall slightly, from
3153 to 3085.

The bust, however, arrived earlier rather than later: output holds near 264000
through 2031, recovers to 334733 in 2035, then falls to 78257 in 2037 and to
nothing from 2038, with every region's population reaching zero. Faster entry
spends the same finite labour force sooner. This is an honest result and not a
regression to be papered over: the economy now dies of something it does, not
of something it is forbidden to do.

## Tests

- `a_year_of_profit_reaches_the_owners_who_carry_the_firm`: a profitable firm
  with a dividend policy transfers the policy's share of free cash to its
  owner's purse.
- `a_firm_that_owes_more_than_it_holds_pays_its_owners_nothing`: the same firm,
  with obligations above its till, pays nothing - a dividend is what is left
  over, never what is borrowed.
- `an_owner_holds_the_economic_share_the_scenario_declares`: the scenario's
  economic and voting shares arrive as themselves, in both directions.
- Workspace green: 226 library tests in adam-core, 9 in adam-content, all
  integration targets, 0 failures. fmt, clippy `-D warnings` and rustdoc
  `-D warnings` clean.

## Measurement

Simulation version 110. Fifty years, seed 1: final fingerprint
`2341776737485900998`, 35.589 s (711.8 ms per year - the cost of an economy
that actually grows to 152 firms rather than staying at 7). The version number
enters the fingerprint hash, so the same world measured at version 109 read
`15844375229004388541`.

## Known gaps

- `RaiseExceedsWorkValue` remains the only entry obstacle of substance: with no
  unemployment, a new firm can hire only by poaching at 110% of another
  employer's wage, and by the last years `no worker` blocks all 72 openings.
  Entry ought to have paths that do not require poaching - training, hiring
  from a growing population, hiring the unemployed.
- The bust is now earlier (2038) and total, with population reaching zero in
  every region. Explaining what consumes the labour force is the next slice.
- Cohorts still report zero annual income even though payroll credits their
  wealth every month, and `livelihood stress` sits near 7500 for decades.
- 4000 basis points of every demo firm were ownerless before this slice; the
  scenario now grants the founder the whole economic side. Multi-owner firms
  and share transfer remain unmodelled.
