# 0190 - Money moves only through balanced postings

## Status

Accepted.

## Context

The economy audit found that the same currency is represented by four liquid
stocks - household wealth, firm cash, actor cash and country treasury - but is
moved by unrelated setters in each subsystem. Some paths conserve the total by
convention. Others create income without a payer, destroy household debt service
without a creditor, destroy freight operating costs, or promise spending without
holding the promised cash. A later aggregate measurement can report the damage,
but cannot reject the operation that caused it.

Slice 0 made those defects observable. It does not make a transfer legal. Slice
1 needs one narrow operation that can say which accounts moved, why they moved,
and whether the complete movement balances before any account changes.

The first migration is deliberately limited to three flows that already intend
to conserve money: paid payroll, household market purchases and direct emergency
relief. Prices, investment, public borrowing and debt settlement remain outside
this migration until their counterparties are modelled.

## Decision

### Accounts are typed and owned

A money account id identifies both the balance class and its owner. The initial
registry contains:

- household liquid wealth, owned by one `CohortId`;
- firm operating cash, owned by one `FirmId`;
- country treasury, owned by one `CountryId`;
- actor liquid cash, owned by one `ActorId`.

An account cannot be opened by spelling an arbitrary string. Resolving an id
against a world must find the named owner and balance. Unknown accounts reject a
transaction.

Restricted and escrow accounts use the same account boundary, but have an
additional purpose and reference id. They are real money balances included in
the money stock. Their owner may not spend them through an unrelated purpose.
They will be introduced when investment commitments and government-program
carryover migrate; a promise field is not treated as an account in the interim.

### A transaction is a balanced set of positive postings

Every posting names an account, a direction (`Decrease` or `Increase`) and a
strictly positive amount. Transaction construction canonicalizes repeated
postings and rejects:

- an empty or one-sided transaction;
- zero or negative posting amounts;
- unequal total decreases and increases;
- arithmetic overflow.

Preflight then resolves every account, calculates every closing balance and
rejects an unknown account or a negative close before changing anything. Only a
fully preflighted transaction may be applied. Application writes the exact
precomputed balances and appends one canonical journal entry. A failed preflight
changes neither balances nor journal.

The first transaction purposes are payroll, household purchase and emergency
relief. Purpose metadata names the economic participants; a market purchase may
credit both seller and carrier, but the buyer's decrease must equal the complete
set of credits.

### Existing balances remain the temporary storage projection

During vertical migration, the existing household, firm, actor and treasury
fields remain the stored balances so untouched systems continue to run. The
posting kernel is the only writer used by a migrated path, and its typed account
resolver projects postings into those fields. A migrated function must not also
call its former debit or credit path.

This is a migration boundary, not permanent dual authority: once every money
flow uses accounts, direct balance mutation becomes private implementation of
the account store and then disappears from domain modules.

### The journal is authoritative history

A successful migrated transfer appends a deterministic entry containing a
monotonic id, simulation date, purpose and canonical postings. Journal order is
command order. It is serialized and included in the stable fingerprint. Equal
seed and commands therefore produce byte-for-byte equal journals.

The journal records settlement, not intention. Offers, obligations and failed
payments do not enter it. Wage arrears and debts are claims, not cash postings.

### Claims never imply money issuance

The shared claim model will name debtor, creditor, principal, accrued amount,
priority, maturity and status. Creating, accruing, restructuring or writing off
a claim does not move currency. Paying a claim requires a separate balanced
transaction from debtor account to creditor account.

This separates two operations that the current model conflates: changing what
is owed and changing who holds money.

### Currency issue and retirement are separate authorized operations

Ordinary transactions must balance and cannot change the liquid-money total.
Currency issue and retirement are not represented as debt creation, negative
expense, or a magic counter-account. They require a distinct monetary-authority
operation, explicit authority, an issue/retire journal record and an amount that
matches the observed stock change.

No issue/retire API is added in the first three-flow migration because the model
has not yet chosen a monetary authority or public-debt holder. Until that choice,
public borrowing remains an explicit known gap rather than silently authorized
issuance.

### Rounding has an owner

A posting is denominated in whole minor units. Splits must either allocate the
remainder to a named account or use deterministic largest-remainder allocation
with typed ids as the final tie-breaker. Dropping a remainder is an unbalanced
transaction and is rejected.

## First migration gate

The initial implementation must prove all of the following before another money
flow moves:

1. unbalanced construction is rejected;
2. insufficient funds reject before any balance changes;
3. paid payroll posts firm operating cash to household liquid wealth;
4. a household purchase posts the buyer decrease to seller goods revenue and,
   when applicable, carrier freight revenue;
5. direct emergency relief posts country treasury to household liquid wealth;
6. each migrated result matches the previous intended balances;
7. migrated functions contain no duplicate direct cash mutation;
8. direct and replayed commands produce identical balances and journal;
9. the simulation version and new fingerprint are documented rather than
   silently blessed.

## Consequences

The operation that causes an imbalance can now fail locally and atomically. The
journal also gives tests and future UI a causal explanation instead of forcing
them to infer transfers from two snapshots.

The first slice does not fix payerless fallback income, anonymous debt service,
freight operating-cost destruction or public-debt issuance. It prevents the
three migrated conserved flows from acquiring the same class of defect while
the claim and authority models are built.

Persisting the journal changes canonical world state and therefore requires a
simulation-version bump and a measured fingerprint update. That cost is
intentional: accounting history is now a model fact, not disposable debug text.

## Implementation status - first vertical gate accepted

The first executable gate is accepted in simulation version 117. The account
resolver projects postings into the existing firm, cohort and country fields;
paid payroll, household purchases and direct emergency relief use it as their
only balance writer. Successful transactions append a serialized journal entry,
and journal length and content participate in the stable fingerprint.

Construction and preflight tests reject unbalanced postings and insufficient
funds without changing balances. Flow tests assert exact purpose metadata,
balances, money conservation and failure atomicity. Existing direct-versus-command
replay tests now compare the journal as part of `World`, so equal commands must
produce equal entries as well as equal closing balances.

The seed-1 fifty-year release gate records 2017 entries and fingerprint
`5540401480800370794`; runtime is 18.131 seconds (362.6 ms/year). The first run
also established a necessary boundary: a positive-priced sub-unit goods fill can
round to zero minor units. It remains a valid physical settlement, but because no
money changes hands it creates no posting or journal entry. Non-zero monetary
components beside a zero total remain invalid.

The ADR is only partially implemented beyond this accepted vertical gate. Firm
procurement, taxation and owner distributions remain to migrate. Restricted and
escrow accounts, explicit monetary-authority issue/retire operations and the
shared claim registry remain required before Slice 1 as a whole is complete.

### Firm procurement expansion

Simulation version 118 extends the accepted gate to firm procurement. A firm-only
account-store projection settles buyer operating cash to seller and carrier
operating accounts, and `FirmPurchase` records buyer, seller, good and optional
carrier. The seed-1 fifty-year run contains 2700 entries and fingerprints to
`1921886831440795849` in 18.257 seconds (365.1 ms/year).

This expansion preserves the zero-money boundary established by the first gate:
a physical sub-unit fill whose complete price rounds to zero moves no monetary
account and produces no journal entry. Taxes and owner distributions remain the
next ordinary transfers to migrate.

### Fiscal and ownership expansion

Simulation version 119 extends the accepted gate to annual sales taxes and owner
distributions. `SalesTax` transfers firm operating money to the named country
treasury on staged firm/country maps. The existing annual country plan still owns
the complete fiscal close and replaces that temporary treasury balance with its
planned revenue-minus-spending result; the journal is the causal record of the tax
receipt, not a second fiscal planner.

`OwnerDistribution` is one multi-credit transaction: one firm decrease and one
positive actor credit per owner whose integer share is non-zero. Ownership rows
stay canonically ordered, so the first row receives the indivisible remainder
exactly as before. Zero-rounded owner shares are omitted because a zero-valued
posting is not a money movement. Journal entries are prepared before either
settlement commits, and rejection leaves all balances and the journal unchanged.

The seed-1 fifty-year gate contains 2863 entries and fingerprints to
`18361472839204730196` in 18.181 seconds (363.6 ms/year). Tax events, fiscal
revenue, and tax journal decreases reconcile exactly. Restricted/escrow accounts,
explicit monetary-authority issue/retire operations, and the shared claim registry
remain required before Slice 1 as a whole is complete.

### Firm investment escrow expansion

Simulation version 120 makes the existing `committed_investments` balance the
storage projection of a typed `FirmInvestmentEscrow` account instead of an
unheld promise. `InvestmentReservation` is a balanced operating-to-escrow
transfer. The account remains owned by the firm and is included in observed firm
money, but only investment-purpose settlement can decrease it.

Project launch releases the exact budget through one `InvestmentOutlay`
transaction from escrow to the local household cohorts that build the works.
Builder shares use the existing largest-remainder rule, journal preparation and
account preflight happen before state mutation, and insufficient operating cash
or escrow leaves the entire world unchanged. The launch operation now owns this
settlement, so direct command execution and autonomous capacity investment use
the same accounting path and cannot double-pay or omit builders.

The seed-1 fifty-year gate contains 2965 entries and fingerprints to
`15155816647335977890` in 18.011 seconds (360.2 ms/year). The investment
restricted-account model gap is closed. Government-program carryover remains the
next restricted balance to migrate; explicit monetary-authority issue/retire
operations and the shared claim registry are still required for Slice 1.
