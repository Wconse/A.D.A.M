# 0174. A standing wage is judged by the same arithmetic as a new one

## Status

Accepted.

## Context

ADR 0173 gave the employer a side of the wage bargain: a firm now refuses a hire
whose wage exceeds the value of the work bought. That closed the loop only for
hires yet to be made. Every agreement already in force was left untouched, and
the fifty-year chronicle is a record of exactly those agreements killing the
economy.

The world did have two ways to shed labor, and neither one asks whether the work
pays. `plan_cash_constrained_staffing` is explicitly advisory - it never fires
anyone - and it reasons about wage coverage from cash and forecast revenue, not
about whether the output is worth the payroll. Distress downsizing in
`distress.rs` fires only after the owner has failed to fund persistent wage
arrears, which is to say only after the firm is already ruined, and then it cuts
a fixed fraction of the roster.

So a firm whose payroll outran its output had no corrective move available while
it still had money. It kept producing goods worth less than the wages it paid,
month after month, until its cash was gone and the distress machinery took over
an already dead business. That is not how an employer behaves in life, and the
reason is not a wrong rule but a missing one.

## Decision

Employment ends when the agreed wage outruns the value of the work bought **and**
the employer has stopped meeting that wage.

A new monthly stage reviews every active agreement against the same ceiling ADR
0173 defined: the local observed price of a batch of output, less the local
observed price of every input the recipe consumes, divided by the labor the
recipe says a batch takes. Where the wage stands above that value and the
agreement already carries unpaid arrears, the agreement is stood down and the
release is reported as a typed `EmploymentEndedAsUnprofitable` event carrying
the wage and the ceiling that condemned it.

The arrears condition is not a grace period in disguise; it is the difference
between two decisions that only look alike. Declining a hire is free: nothing is
lost that the firm already had. Ending a standing agreement is not - the firm
gives up a worker it has already found and trained, and must court one again
when the price recovers. An employer facing that cost waits, and what it spends
waiting is its own cash. So the rule lets it wait exactly as long as it can pay.
The moment the wage goes unpaid, the waiting is no longer financed by the
employer but by the worker, who never agreed to lend it anything, and the
question settles itself.

Judging the wage on a single month's observed price without that condition was
the first attempt, and it was wrong in a way worth recording: it fired a whole
roster on one month of bad prices, at no cost to anyone, which is a magic
correction rather than a decision. Every test world in the repository broke
against it - not because those worlds were mistuned, but because a costless
instant layoff is not a thing employers have.

The whole agreement ends rather than some fraction of it. Every worker on an
agreement priced above the value ceiling is losing the firm money at the same
rate, so there is no honest basis for choosing a percentage to keep; a fraction
would be a tuned number standing in for a decision nobody makes.

Wage arrears already owed survive the release, because standing down future work
does not discharge a debt for work already done.

The stage runs before hiring in the monthly cycle. A firm stops paying for work
it cannot earn back before it considers taking on more.

This bound is owned. The firm pays for keeping a loss-making agreement in cash
every month, which is what makes ending it rational, and it pays again in the
search it must repeat once it lets the worker go. The worker pays first in
unpaid wages and then in lost income on returning to the labor market. Both
costs are real, both are recorded, and neither is absorbed by the world.

## Consequences

A collapse in the price of a good now reaches the people who make it, through
the channel it travels in life: the work stops being worth doing and the jobs
go. Employment becomes a consequence of prices and productivity rather than a
fixture of the scenario.

Distress downsizing is left as the last resort it should be, handling firms that
failed for reasons other than an unpayable wage.

Because the release waits for arrears, a passing dip in price no longer costs
anyone their job; a lasting one still does, by way of the till running dry.
Churn is therefore paced by the depth of the employer's pocket rather than by
the noise in a single month's price, and no damping constant was needed to say
so.

The rule now overlaps distress downsizing rather than replacing it: both wake up
when wages go unpaid. Distress cuts a fixed fraction of a roster the owner will
not refinance; this rule ends outright those agreements the output can never
cover. The fixed fraction in `distress.rs` remains an unowned number and is
still owed a justification of its own.

## Known gaps carried forward

- Releases are not yet summarized in the chronicle, so a year of shed jobs does
  not read as an event in the annual narrative.
- The demo scenario still never enables the labor market for its recipes, so its
  static rosters are outside both this rule and ADR 0173. Enabling it is the
  step that will finally let these two slices act on the fifty-year run.
- Severance, notice periods, and any obligation beyond accrued arrears do not
  exist.
- Poaching, retention, and job switching still validate against the competitive
  bid alone.
