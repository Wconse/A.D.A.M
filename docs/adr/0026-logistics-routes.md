# ADR 0026: Physical logistics routes and shipment planning

- Status: Accepted foundation
- Date: 2026-07-21

Logistics begins with directed regional routes carrying mode, monthly capacity, unit cost, transit time, and reliability. Shipment orders contain good, origin, destination, quantity, and budget. Planning rejects wrong direction, insufficient capacity, and excess cost, then chooses the cheapest feasible route with stable RouteId tie-breaking. Multi-leg routing, shared capacity reservation, carriers, congestion, and shipment execution build on this physical contract.
