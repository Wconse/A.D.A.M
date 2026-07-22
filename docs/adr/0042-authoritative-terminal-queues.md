# ADR 0042: Authoritative terminal queues

- Status: Accepted foundation
- Date: 2026-07-22

Terminal waiting queues are authoritative World state. Existing shipments can be queued once, FIFO admission reserves shared terminal throughput, and queue/admission transitions emit replayable events. Queue order and terminal usage participate in snapshots and fingerprints. Shipment phase mutation and handling completion remain the next integration layer.
