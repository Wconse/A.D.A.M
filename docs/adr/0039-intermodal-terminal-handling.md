# ADR 0039: Intermodal terminal handling phases

- Status: Accepted foundation
- Date: 2026-07-22

Movement between route legs is not instantaneous. Intermodal shipment state alternates between route transit and explicit terminal handling, with positive transfer duration for every intermediate connection. Time advancement reports route completion and transfer completion separately, enabling capacity release, queues, customs, storage costs, and next-leg dispatch to attach to causal transitions.
