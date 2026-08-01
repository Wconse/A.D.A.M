# ADR 0170: The fiscal circuit closes, and only borrowing creates money

## Status

Accepted.

## Context

An external review found that the demo economy collapses to a few hundred people on every seed. ADR 0169 recorded the finding without fixing it, because a build fix must never be mistaken for an economic fix.

The cause was located numerically and then proved. Firms paid a 20% final-sales tax that left the circuit and never returned to it. Government `spending` moved the treasury and the public-debt balance, but it was never credited to any agent: no household, firm, or worker received it. Money therefore left the economy every month and re-entered only through emergency relief.

The collapse was not a balance problem, and no configuration change can repair it. With a tax rate `t` on final sales, firms can only stay solvent if `(1 - t) * P * Q` covers wages plus intermediate inputs, while households can only buy the output if wages are at least `P * Q`. For `t > 0` those two requirements are jointly unsatisfiable in a closed circuit with no other source of household income. Raising prices in `demo.toml` was tried and made the outcome strictly worse: every seed fell to zero population. That experiment was reverted.

The defect was structural. A tax that vanishes is not taxation; it is destruction of money that happens to be labelled revenue.

## Decision

Executed public spending becomes household purchasing power in the same annual closure that records it. The material country planner reports the executed outlay on `CountryUpdate`, and closure distributes it to household cohorts as wages of public employees, transfers, and coupon paid to domestic holders of the public debt.

The distribution is by population, using the largest-remainder method with a `CohortId` tie-break, so the split is exact, deterministic, and loses no minor unit to rounding.

The legacy non-material planner pays nothing. Its "revenue" is a coefficient applied to output rather than tax collected from identifiable firms, so crediting households from it would invent money instead of moving it.

Each payment emits a typed `PublicOutlayDistributed { country, cohort, amount }` event, so the chronicle and the tests observe a causal chain between named agents rather than a change in a balance.

## Invariants

1. Collected sales tax equals recorded fiscal revenue.
2. Every minor unit of executed public spending reaches a household cohort.
3. Taxation moves money and never creates or destroys it.
4. `firm cash + household liquid wealth + treasury = initial money stock + public debt`. Money is created only by public borrowing.
5. The outlay split by population loses nothing to rounding and is invariant to iteration order.

Invariant 4 replaces the previous test assertion in `intermediate_procurement.rs`, which added the vanished tax back by hand so the books would appear to balance. That assertion encoded the leak it should have caught.

## Consequences

The demo economy no longer dies immediately. Measured over fifty years on seeds 1 to 5, Southvale now sustains 144000 of output against 144000 of final consumption for roughly four decades, with a growing population, where previously output was zero almost at once.

The worlds still end, and the remaining causes are now visible instead of masked by the monetary leak:

- Eastport depopulates within about thirteen years on every seed. Borealia collects revenue once and then nothing; its treasury is frozen thereafter.
- Northreach depopulates over the following decades. When it reaches zero the grain farm loses its workforce, the Southvale bakery loses its only input, output falls to zero, and the surviving region starves within three years. This is a genuine causal chain rather than an accounting artifact.
- Arcadia runs a permanent small deficit against a flat 28800 of revenue, accumulating debt to roughly 100000 before restructuring.

These are consequences of static prices and of demography modelled as a rate rather than a process. They are the subjects of the next two steps and are deliberately not addressed here.

Simulation version is now 96. Fingerprints change accordingly, which is expected: this ADR changes what the world does, not only how it is reported.
