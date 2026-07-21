# ADR 0028: Route capacity reservations and shipment lifecycle

- Status: Accepted foundation
- Date: 2026-07-21

Every planned shipment reserves its full quantity on every route atomically. Failed capacity checks create no partial reservations. In-transit shipments hold capacity until delivery, when all route reservations are released. Shipment status and remaining transit time are authoritative and serializable. Cancellation, queues, partial loads, and actual inventory transfer remain future layers.
