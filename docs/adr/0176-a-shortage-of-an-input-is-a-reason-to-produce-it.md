# 0176 - A shortage of an input is a reason to produce it

## Status

Accepted.

## Context

The demo economy dies. Eastport has a bakery and no farm, its imported grain
runs out in the fourth month of the first year, its people stop eating, and the
resulting monthly shortfall is read by the grievance machinery as Arcadia
withholding food until the two countries are hostile and the road closes for
good. Fifty years later the world produces nothing.

The investigation behind `docs/roadmap.md` "Step B6" found three unclosed
halves feeding each other. This ADR closes the third and most decisive one,
because the first two cannot help while it stands.

`execute_observed_firm_entry` built its evidence from
`current_regional_survival_shortages`, which reads `unmet_demand` - the needs of
households. The only consumption profile in the demo is bread. Grain is an
intermediate good that no household ever eats, so no quantity of missing grain
ever appeared in that map. A region could report `FirmProcurementShortfall` for
the same input every month for fifty years while entry reviewed nothing, and a
farm was never even considered. The shortage was journaled as an event and then
thrown away; nothing in the world remembered it.

That is a magic constraint in the sense AGENTS.md means: supply could not
answer an input shortage, and nobody paid for that inability. It was simply
invisible. It also explains why the two price-side halves are inert. A
displacement price for a contested input, measured on the
`spike/contested-input-price` branch, correctly reorders who eats and lifts the
seller's revenue, and it still saves nobody - it only moves the hunger from
Eastport to Northreach, because raising the price of a fixed quantity decides
who goes without rather than calling more of the good into existence.

## Decision

A region's firms failing to buy an input is entry evidence of the same standing
as its households failing to eat.

- `World` gains `monthly_firm_input_shortfalls: BTreeMap<(RegionId, GoodId),
  QuantityMilli>`, cleared with the other monthly firm accounts in
  `reset_monthly_firm_accounts` and folded into the determinism fingerprint.
- `record_procurement_outcomes` accumulates every unmet procurement order
  against the buyer's region as it journals the shortfall event.
- `execute_observed_firm_entry` merges that map into the household survival
  shortages before accruing entry pressure, so an input channel that stays
  starved accrues pressure month after month exactly as an unfed household
  does.

Both causes of an unmet order count, including an exhausted route pool. A good
that cannot be delivered here and a good that nobody here has are the same
invitation to produce it here; only the grievance machinery needs to tell them
apart, and it already does.

## Consequences

- Intermediate goods are no longer second-class. Any recipe input can now pull
  entry pressure toward the region that keeps running short of it.
- Measured over fifty years from seed 1, first-year excess deaths fall from 73
  to 58 and the yearly toll decays faster, because entry pressure now diverts
  production planning toward the starved channel even before any firm is
  founded. Fingerprint `8238339567197527992`; `SIMULATION_VERSION` 102 -> 103.
- The demo still starves, and the reason is now visible and specific rather
  than mysterious. See the known gaps.

## Known gaps

- **Entry can still only hire the unemployed.** `select_entry_worker_cohort`
  skips every cohort whose `EmploymentStatus` is not `Unemployed`, and every
  demo cohort is employed, so `plan_firm_entry` returns `None` however loud the
  evidence gets. In life a new employer takes staff from an existing one by
  offering more; here it waits for someone to be idle. This is the next slice,
  and until it lands the pressure this ADR creates has nowhere to go.
- Entry evidence is a quantity, not a profit. An entrant should weigh the price
  of the scarce good against the cost of producing it, which is what makes the
  displacement price on the spike branch worth merging afterwards rather than
  before.
- Existing firms still do not expand toward a contested input; only entry
  reacts.
- Grievance evidence still cannot distinguish "the foreign seller had none
  left" from "the foreign seller refused us", so a queue-driven shortage keeps
  manufacturing hostility.
