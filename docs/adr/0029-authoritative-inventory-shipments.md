# ADR 0029: Authoritative inventory shipments

- Status: Accepted foundation
- Date: 2026-07-22

Routes, shared capacity, and firm-to-firm shipments are authoritative World state. Starting a shipment validates firm regions, goods, source inventory, route plan, budget, and all capacity before atomically reserving routes and debiting source stock. Delivery releases capacity and credits destination inventory exactly once. Commands and events use the same path for player, AI, save, and replay.
