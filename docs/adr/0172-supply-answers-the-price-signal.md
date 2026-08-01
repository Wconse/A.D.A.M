# 0172. Supply answers the price signal

- Status: accepted
- Date: step 099

## Context

ADR 0171 removed the ceilings that used to hold prices down. Scarcity was then
free to reprice a good upward without limit, and in a permanently short market
it did exactly that, month after month, forever.

That was not a pricing bug. It was an unfinished loop. In life a price that
stays high is an invitation: somebody builds more of the thing, supply grows,
and the price comes back down. Our world could not answer the invitation.

The diagnosis was that all the parts existed but nothing called them:

- `plan_observed_production_adjustments` already wanted to raise a sold-out
  firm's target, but its ceiling was `min(demand, physically feasible)`, and
  physical feasibility is bounded by installed capacity, which never grew.
- `investment.rs` could already build capacity, but `launch_investment_project`
  and `advance_investment_project` were reachable only through an external
  command. No monthly stage ever called them.
- `board.rs` could commit firm cash to investment, but only through a board
  resolution that autonomous firms never passed.

So the supply side of the price signal was disconnected. Prices rose and
nothing in the world responded.

## Decision

A new monthly stage, `execute_observed_capacity_investment`, closes the loop.

Each month it first advances every open construction project, then reviews the
same production evidence the management stage uses. A firm orders new capacity
only when all of the following are true of its own observed experience:

1. It is solvent.
2. It ran out of stock more often than it was left holding unsold goods.
3. It expects a positive operating cash margin on the extra output.
4. Demand already absorbs everything its current capacity can make.
5. It has no construction already under way.

The size of the expansion is not a tuned constant. It is the smaller of two
real quantities: how much demand the firm actually watched walk away, and how
much capacity the firm can pay builders for out of its own cash. If it can
afford nothing, it builds nothing, and the shortage persists — honestly.

Committing the money is a scoped corporate power, `commit_firm_investment`,
guarded by `ProposeMajorInvestment` authority, and every decision crosses the
replayable command boundary through `WorldCommand::CommitFirmInvestment`.

The budget does not vanish into the works. It is paid immediately to the
households of the firm's region as building income, reusing the mechanism
ADR 0171's slice introduced for founding capital. Investment moves money; it
never destroys it.

## Consequences

- A durable shortage now has an ending. High prices attract capacity, capacity
  raises output, and output relieves the price. The correction takes three
  months of construction, so overshoot and lag are real and visible.
- Capacity growth is financed, not granted. A poor firm in a hungry market
  stays small, which is the honest outcome and a legible source of drama.
- Building a works is a payday for somebody. Investment booms now show up as
  household income, and a collapse in investment shows up as its loss.
- Both directions are gated by tests: a profitable sold-out bakery must expand
  and must pay its builders, and a loss-making chain must refuse to expand no
  matter how hungry its market is.

## Known gaps

- Construction takes a fixed three months. Builders are not yet an industry
  with their own order book, so the lag is a physical constant rather than an
  outcome of somebody's schedule.
- Expansion is cash-financed only. A firm cannot yet borrow to build, so the
  credit market plays no part in the investment cycle.
- Firm entry still triggers only on survival-good shortages. A durable high
  price in any other good attracts no new entrant.
