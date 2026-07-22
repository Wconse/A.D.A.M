# ADR 0044: Authoritative intermodal shipment integration

- Status: Accepted foundation
- Date: 2026-07-22

Inventory shipments now use intermodal lifecycle state. Intermediate regions select the lowest-ID registered terminal deterministically. Route completion releases route capacity and automatically queues the shipment. FIFO terminal admission starts handling; transfer completion releases terminal throughput and starts the next route. Terminal selection and phase state are saved and fingerprinted.
