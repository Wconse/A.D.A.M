# ADR 0033: Contract tariffs and carrier operating settlement

- Status: Accepted foundation
- Date: 2026-07-22

Shipment routing uses active shipper/carrier/route contracts to derive effective tariffs. Settlement charges the shipper discounted revenue and applies each carrier's revenue minus fuel, labor, and maintenance as its cash delta. All party balances are prevalidated before route reservation, inventory debit, or money transfer. Contract capacity guarantees remain a separate future ledger.
