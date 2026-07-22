# ADR 0043: Terminal admission gates handling

- Status: Accepted foundation
- Date: 2026-07-22

Completing a route now places an intermodal shipment in `WaitingForTerminal` with no handling-time progress. Only explicit external admission can start terminal handling. Advancing simulation time while waiting leaves the shipment stationary. This separates queue delay from physical handling duration and lets authoritative terminal capacity control entry without hidden automatic service.
