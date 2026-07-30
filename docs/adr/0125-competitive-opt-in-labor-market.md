# ADR 0125: Competitive opt-in labor market

## Status

Accepted.

## Context

Shortage-driven entry can create employers while regional cohorts contain only a finite number of workers. Letting every firm obtain its nominal workforce independently would duplicate labor and remove wage and qualification pressure. Activating a new matcher for every existing recipe, however, changes legacy scenario economics even when content authors never selected the new rules.

## Decision

Competitive vacancy matching is enabled only for recipes with an explicit labor profile. The profile currently states the minimum education level; absence means the recipe keeps legacy staffing behavior.

For enabled recipes, monthly vacancies are derived from the authorized production target and recipe labor requirement, bounded by installed firm worker capacity, and reduced by active agreements. Each operating, solvent firm may make at most one offer per month. An offer must be covered by current firm cash.

The wage bid uses cohort annual income per person as a local reservation-wage anchor and adds a bounded production-urgency premium. A qualified cohort must be local and have at least one worker not already allocated to an active agreement. Offers are ranked by wage descending, then stable firm and cohort identity. A worker can therefore accept only one offer, and equal simulations resolve competition identically.

Matches use the shared command boundary, revalidate vacancy, cash, locality, qualification, profile identity, and worker availability, then create or expand the employment agreement. Matching itself does not prepay or reserve wages; ordinary payroll remains the monetary settlement and can still produce arrears if conditions change before payment.

Shortage-driven firm entry retains the historical Basic qualification fallback when no profile exists. This preserves step 076 behavior while ensuring that later vacancy matching remains opt-in.

Content schema v8 accepts optional `minimum_education` on recipes with values `none`, `basic`, `secondary`, `vocational`, and `tertiary`.

## Invariants

- A cohort worker cannot be allocated to more than one active agreement.
- A firm cannot hire beyond installed worker capacity or target-derived labor demand.
- Unfunded, insolvent, remote, or underqualified matches are rejected atomically.
- Each firm hires at most one worker in one monthly matching stage.
- Higher wage wins scarce labor; all ties use canonical identifiers.
- Recipes without an explicit labor profile do not participate in automatic matching.
- Direct execution and replayed commands produce equivalent state and fingerprints.

## Acceptance criteria

- Two firms competing for one worker award that worker to the higher ranked funded offer.
- A qualification floor blocks an underqualified cohort without mutating firms or agreements.
- Matching replay produces equivalent domain state and the same stable fingerprint.
- The embedded legacy demo has no configured labor profiles and retains its accepted 50-year path.
- A schema-level profile loads the selected education floor.
- Formatting, Clippy, workspace tests, documentation, and the 50-year deterministic comparison pass.

## Consequences

The simulation now has finite local labor, wage competition, and skill bottlenecks where content explicitly asks for them, without silently rewriting old worlds. The first model intentionally omits worker mobility, reservation savings, contract duration, poaching of already employed workers, training, bargaining, and multi-worker hiring bursts. The next useful slice is persistent unemployment and vacancy evidence, followed by bounded wage adaptation and migration or training decisions driven by that evidence.
