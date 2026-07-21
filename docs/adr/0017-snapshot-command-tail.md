# ADR 0017: Snapshot plus ordered command tail

- Status: Accepted foundation
- Date: 2026-07-21

A continuation save may contain an authoritative snapshot and the ordered commands accepted after that snapshot. Commands use the same domain boundary for player and AI. Recovery restores the snapshot and replays the tail in order. Tests require snapshot-plus-tail to equal uninterrupted execution exactly. Future snapshots can compact an old tail without changing history.
