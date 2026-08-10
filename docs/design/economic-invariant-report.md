# Economic Invariant Report — Slice 0 Contract

Status: implementation contract for the first slice of `docs/economy-rebuild-plan.md`.

## Decision added by this slice

The simulation gains one deterministic, read-only report which describes whether authoritative
stocks agree at a single point in time. It does not repair state, block actions, move resources,
change actor policy, or make an existing approximation look plausible.

The report is the inspection boundary for later balanced-posting work. A defect must become a named
finding at the state transition that produced it instead of being inferred from a fifty-year output.

## Authoritative inputs

The report reads existing `World` state only:

- household, firm, actor and treasury cash;
- household, public, wage and firm-credit claims;
- region and cohort population/household ledgers;
- active employment agreements and program labor usage;
- committed investment and program balances;
- route, freight and terminal reservations;
- firm inventory, public reserves and goods in transit.

It introduces no second economic ledger. Totals are projections and findings carry stable entity
identifiers back to their authoritative records.

## Hard invariants in Slice 0

1. Every registered region's population equals the sum of its cohort populations.
2. A zero-person cohort has zero households.
3. A non-empty cohort has between one and `people` households.
4. Active employment workers assigned to a cohort do not exceed its people.
5. Every liquid balance included in the current money stock is non-negative.
6. Route and terminal usage do not exceed their declared physical capacities.
7. Aggregate arithmetic is checked; overflow is an error, never saturation.

The report may expose known model gaps without classifying them as hard violations. In this slice:

- household debt and public debt have no named creditor asset;
- investment commitments and program carryover are not held in restricted accounts;
- a firm workforce can disagree with active employment agreements;
- money conservation across a transition cannot be proven from one snapshot.

Those are recorded with audit references and become acceptance inputs for later slices.

## Finding contract

Each finding contains:

- severity: hard violation or known model gap;
- stable code suitable for tests and diagnostics;
- audit reference;
- involved entity when one exists;
- measured and allowed values;
- short causal explanation.

Collection order is canonical. Entity identifiers, not map insertion order, determine order.

## Money transition reconciliation

A separate pure helper compares two reports:

`unexplained change = closing liquid money - opening liquid money - authorized net issuance`

Zero means the observed liquid stock reconciles for that transition. A non-zero value is a hard
violation. The caller must supply authorized net issuance explicitly; debt creation is not assumed
to be issuance.

This helper does not yet create journal entries. Slice 1 will make the authorization itself an
authoritative posting instead of a test parameter.

## Acceptance scenarios

1. A valid minimal region/cohort world reports no hard population or household violation.
2. A region/cohort population mismatch is detected and names the region.
3. A zero-person cohort retaining a household is detected and names the cohort.
4. Employment exceeding cohort population is detected without changing the agreement.
5. A deliberately removed unit of firm cash creates an unexplained negative money transition.
6. Supplying an equal authorized retirement reconciles that same transition.
7. Existing household/public debt appears as a named model gap rather than silently entering cash.
8. Report equality and ordering are deterministic across cloned worlds.
9. Direct API and replayed command worlds produce identical reports.

## Explicit non-goals

- no balanced posting journal yet;
- no repair of payerless income, debt or freight costs;
- no change to simulation version or regression fingerprint;
- no event-log or chronicle change;
- no automatic failure of monthly or annual execution while known gaps remain;
- no new economic constants or clamps.

## Gate

Before acceptance:

- formatting, `cargo check`, strict Clippy, all tests and docs must pass;
- the new report and its tests must not change the seed-1/50-year fingerprint;
- `docs/roadmap.md` must record measured findings, approximations and the next gate;
- no regression fingerprint may be updated merely because diagnostics were added.

## Implementation status - accepted 2026-08-09

Slice 0 is implemented in `crates/adam-core/src/economic_invariants.rs` and
exported from `adam-core`. The implementation is read-only, canonically ordered,
checked for aggregate overflow, and leaves `SIMULATION_VERSION` at 116.

Acceptance evidence:

- eleven focused invariant, defect and reconciliation tests pass;
- monthly and annual economic-cycle results carry their closing invariant report;
- `economy-defect-test-matrix.toml` maps audit ids to non-ignored assertions;
- the entire workspace test suite passes;
- `cargo fmt --all -- --check` passes;
- strict workspace Clippy with `-D warnings` passes;
- workspace documentation builds without dependencies;
- the release seed-1 fifty-year fingerprint remains
  `2108872914777405248`.

Known model gaps remain findings rather than repairs. Their accounting migration
belongs to later slices and must not be smuggled into this observation layer.

