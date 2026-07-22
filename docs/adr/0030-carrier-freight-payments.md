# ADR 0030: Carrier-owned routes and freight payments

- Status: Accepted foundation
- Date: 2026-07-22

Every authoritative route has a carrier firm. Shipment start computes each leg charge from fixed-point quantity and route tariff, verifies sender liquidity, reserves all capacity, debits source inventory and cash, and credits carriers deterministically. Shipment state records total cost. Fuel, wages, maintenance, contracts, insurance, and carrier insolvency remain future operating layers.
