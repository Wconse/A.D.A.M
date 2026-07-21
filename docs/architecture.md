# Architecture

## Direction of dependencies

```text
configuration / player input
            │
            ▼
        commands
            │
            ▼
    adam-core systems ──► domain events ──► archive
            │                                  │
            ▼                                  ▼
       world state                       chronicle views
            ▲                                  │
            └──────── replay / audit ──────────┘
```

`adam-core` knows nothing about Bevy, egui, filesystems, databases, or wall-clock time. Applications compose adapters around it.

## Core concepts

- **WorldState** is the authoritative deterministic state.
- **SimDate** is explicit discrete time; rendering cadence cannot advance it.
- **Commands** represent actor intent. Player and AI produce the same command types.
- **Systems** validate commands and derive state transitions.
- **DomainEvent** records meaningful facts after they occur.
- **EventLog** is append-only and ordered by a monotonic sequence number.
- **Chronicle** is a read model built from events; it never mutates the world.

## Deterministic data structures

Use ordered collections (`BTreeMap`, sorted vectors) in canonical state and serialization paths. A hash map may be used as an internal optimization only when its output is sorted before it can affect behavior.

## Crate evolution

Stage 0 begins with two crates. Split a new crate only when a stable boundary is proven by real code. Expected later adapters include archive storage and chronicle rendering, but premature crate proliferation is intentionally avoided.

## Persistence direction

The event schema and save schema will be versioned independently. SQLite is an archive adapter, not the domain model. A future save may combine periodic snapshots with an ordered input/event tail.

## Performance rule

Measure before optimizing. Off-screen approximation is acceptable only when conserved quantities and causal outputs are specified and tested.
