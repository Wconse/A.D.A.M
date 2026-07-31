# Step 100 — The Governing State Becomes Visible

## Milestone promise

Step 100 is one anniversary vertical slice delivered as numbered substeps, not a single oversized commit. It joins two goals:

1. **The Governing State** — persistent government programs, contested execution, winners and losers, and political memory.
2. **World Observatory** — the first Bevy client that displays the authoritative world and submits the same `WorldCommand` values as CLI or AI actors.

The playable loop at the milestone gate is:

> inspect the world → announce a program → promise regions money and an outcome → advance time → watch appropriation and physical delivery diverge → inspect beneficiaries, excluded regions, political resistance, trust, legitimacy, and the historical record.

## Non-negotiable architecture

```text
player / AI / CLI
        ↓
    WorldCommand
        ↓
 adam-core authority
        ↓
DomainEvent + World state
        ↓
adam-viewer projection
```

Bevy owns input, presentation, camera, animation, and local UI selection only. It never owns population, budgets, program status, service delivery, political effects, simulation time, or randomness.

## Anniversary sequence

### 100.1 — Program charter and political promise

- Add a persistent `GovernmentProgram` identified by `ProgramId`.
- Store initiator, country, name, target regions and exact shares, promised annual funding, duration, public-service priority, promised improvement, start year, status, appropriation, delivery, carryover, and delay memory.
- Declare programs through a replayable `WorldCommand` and typed event.
- Separate announcement from funding: an authorized actor may legally over-promise, while later execution exposes the shortfall.
- Reject only invalid accounting, unknown/foreign regions, past scheduling, duplicate identity, or absent authority.

### 100.2 — Appropriation, debt choice, and competing programs

- Add annual appropriation commands and autonomous proposals.
- Let programs compete for the real discretionary envelope.
- Permit zero funding, concentration, cancellation, continuation, and debt-financed appropriation when law and debt capacity allow.
- Record promised versus appropriated funding without silently rewriting the promise.

### 100.3 — Contested physical execution

- Convert appropriations into regional execution plans.
- Bound delivery by administration, infrastructure, labor, goods, logistics, and time.
- Carry unused appropriations explicitly; do not invent corruption before a concrete mechanism exists.
- Let political offices and influence edges create support or administrative friction, never an unexplained veto of an authorized act.

### 100.4 — Winners, losers, and political memory

- Identify direct beneficiaries, excluded regions, net fiscal contributors, patrons, and bypassed power centers.
- Apply the promise gap separately from ordinary service shortfall: a broken public promise is worse than silence.
- Feed delayed consequences into regional satisfaction, confidence, legitimacy, elite cohesion, migration pressure, and future priorities.
- Preserve program-specific cumulative memory after completion, cancellation, or failure.

### 100.5 — Program chronicle

- Narrate announcement, reaction, appropriation, execution constraints, outcomes, and legacy as one historical arc.
- Attribute actors, offices, regions, money, delays, fulfilled share, and political effects from typed events.
- Keep program cards addressable so a future UI can open the complete history of one program.

### 100.6 — Bevy foundation and deterministic world map

- Add `apps/adam-viewer` without adding Bevy to `adam-core`.
- Load the existing demo world.
- Render a deterministic stylized 2D map, countries, regions, capitals/centers, routes, and stable same-world layout.
- Add pan, zoom, region selection, date/state display, pause, month/year step, normal speed, and fast speed.

### 100.7 — Inspector and map overlays

- Region inspector: population, output, services, unemployment, pressure, satisfaction, interest, housing, migration, firms, tax contribution, ordinary budget, and program receipts.
- Overlays: political ownership, population, economy, services, social pressure, satisfaction, fiscal incidence, housing, migration, political influence, and active programs.
- Visual changes are projections of authoritative snapshots and events.

### 100.8 — Playable program desk

- Build the first real decision UI around `DeclareGovernmentProgram` and appropriation commands.
- Allow 100% to one region, 0% to others, over-promising, parallel programs, debt proposals, cancellation, and politically dangerous choices.
- Show warnings and forecasts without disabling otherwise legal commands.
- Animate appropriation and delivery only after accepted core events.

### 100.9 — Graphical political timeline

- Event cards with country, region, actor, program, and event-type filters.
- Selecting an event focuses the map and highlights involved regions and political actors.
- Program pages compare promise, appropriation, delivery, delay, beneficiaries, opposition, and legacy over time.

### 100.10 — Anniversary release gate

- Deterministic replay and fingerprints remain stable.
- CLI remains functional and headless.
- Viewer can be removed without changing simulation behavior.
- Full formatting, Clippy, tests, docs, release build, and diff checks pass.
- A player can complete the milestone loop without editing configuration or using the terminal.

## Deliberate scope limits

- The first map is stylized, not a geographic simulator.
- No abstract theft percentage; corruption waits for actors, opportunities, transfers, detection, and consequences.
- No electoral simulator is required for Step 100.
- No UI-only authority, resources, outcomes, or player bonuses.
- No fairness caps: concentration and self-defeating policy remain legal when institutionally and materially feasible.

## Completion definition

Step 100 is complete only when the political program arc and the graphical client meet in one playable deterministic loop. The core political slices come first so the viewer visualizes real causality rather than a decorative mock-up.

## Completion status

**Accepted and complete.** Steps 100.1 through 100.10 passed the anniversary release gate. See `docs/release/step-100-anniversary-acceptance.md`.
