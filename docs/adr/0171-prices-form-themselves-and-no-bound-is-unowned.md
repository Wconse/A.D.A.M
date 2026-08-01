# ADR 0171: Prices form themselves, and no bound may be unowned

## Status

Accepted.

## Context

ADR 0170 closed the monetary circuit and named the two defects it deliberately left standing. The first was that prices are static. A regional reference price was a number set at world creation, and nothing that happened in the market could change it. A region could leave nine tenths of its demand unfilled for fifty years and the price would not move by one minor unit, so scarcity had no way to reach producers, buyers, or the chronicle.

This ADR makes prices form themselves out of settled trade. In doing so it also establishes a general rule about bounds, because the first two attempts at this slice both failed on the same mistake.

The first attempt raised a price whenever demand went unfilled. That is wrong: demand left unfilled while goods sat unsold on the shelf is unpaid need, not effective demand. Households with no money cannot bid a price up, and the unsold stock beside them argues for a cut. The relief test caught this immediately, because a cohort that had just been given emergency money faced a higher price the following month, which recreated the affordability gap the relief was meant to close.

The second attempt bounded the monthly move to three percent. That bound was magic. Nobody in the model was paying to hold the price down. Worse, because reference prices are whole minor units, three percent of a price of five rounds to zero, so the rule was paired with "always move by at least one minor unit" -- which meant a cheap good moved twenty percent per month while the stated bound claimed three. Grain went from 5 to 16 in one year with the physical economy unchanged. The bound did not restrain anything; it hid the fact that supply never responds to scarcity.

## Decision

A regional reference price is revised each settled month from the sellers' own books.

A market that **sold out** while demand went unfilled raises its price in proportion to the share of demand that went away empty. A market that left goods **unsold** cuts its price in proportion to the share of the offer that did not sell, whether or not needs went unmet beside it. A market that cleared exactly is left alone.

The move is not bounded. A market that emptied completely reprices by the full scarcity elasticity in a single month, and a shortage that severe produces a price shock, because that is what the observed scarcity implies and nobody in this model is paying to prevent it.

Two limits remain, and both name who pays:

- **The variable-cost floor.** No price settles below the variable cost per unit of the cheapest solvent local producer, because that producer loses money on every unit sold below it and simply refuses. A price found below that floor goes to it at once rather than creeping toward it.
- **The fractional carry.** Currency is discrete. A move worth a quarter of a minor unit is carried in thousandths into the following months until it is worth a whole unit, instead of being rounded up into a large jump. This is arithmetic, not economics: it exists because money is an integer, and it is what makes an unbounded rule safe on a coarse price grid.

Every revision emits a typed `RegionalPriceAdjusted` event carrying the previous price, the new price, the offered, sold, unsold, and unfilled quantities, the cost floor, and whether the floor bound the result, so a price movement is auditable back to the trade that caused it.

## The general rule

Recorded in `AGENTS.md` as **No unowned constraints**, and binding on all future work:

> Every limit in the economy must name the party that pays for it. A constraint is admissible only when some modelled actor bears a concrete cost -- money, health, reputation, forgone profit, or a broken contract -- for the limit being observed. If nobody pays, the limit is magic and must be deleted rather than tuned.

Two corollaries follow. First, a magic bound is worse than the instability it hides, because it conceals the unfinished loop that produced the instability. Second, divergence from real life is not evidence that a rule is wrong; the first hypothesis is an unfinished causal loop. Legitimate exceptions are the domain of a type, the discreteness of a unit, and a division guard, and they must be commented as such.

## Invariants

1. A price rises only when the market sold out with demand still unfilled.
2. A market holding unsold stock never raises its price, however much need went unmet.
3. No price settles below the variable cost of its cheapest solvent local producer.
4. Fractional pressure is conserved: nothing is rounded away and nothing is invented.
5. Carried pressure is part of world state and of the deterministic fingerprint, so a replayed month reproduces the same price path exactly.

## Consequences

Scarcity is now visible in the price system. In the two-firm production chain used by the procurement tests, the farm sells out every month while the bakery still wants more, so grain reprices upward all year, and the bakery's bread is short against household demand, so bread reprices upward as well. The annual grocery bill in that world rose roughly fourfold.

That number is not a balance failure and has not been damped. It is the exact shape of the next missing loop: **supply does not respond to scarcity.** No existing farm expands its production target because grain is dear, and no new baker enters because bread is dear. Until entry and capacity investment answer the price signal, a permanent shortage compounds forever, and it should be allowed to, because the compounding is what makes the gap impossible to overlook.

Three tests that asserted fixed prices as constants -- "twelve monthly grain trades at 5", "twelve monthly grain trades at 6", "twelve months of two bread units at 10" -- were rewritten as invariants. Inventory investment is still required to be valued at observed transaction prices rather than from the reference table, but the test now derives that requirement from the live prices instead of hard-coding the answer.

The audit that this ADR mandates has begun. Five surviving bounds have been identified as unowned and are scheduled for removal: the loan rate clamped to 15-45%, the population growth rate clamped to +/-3% per year, the output growth rate clamped to -15/+20%, the smoothing of regional satisfaction shifts, and the clamped effect of government programmes. Each is expected to be hiding a loop in the same way the price ceiling was.

Simulation version is now 97. Fingerprints change accordingly.
