# ADR 0097: Market Import Freight Revenue

## Status

Accepted

## Context

Immediate household and firm-procurement imports already used delivered prices: the goods offer price plus the selected route tariff. Settlement nevertheless credited the entire delivered payment to the goods seller. The route carrier moved no inventory through the explicit shipment lifecycle and received no market freight revenue.

That mismatch made logistics firms physically present but economically absent from immediate imports. It also overstated seller revenue, taxed household freight as if it were a final goods sale, and recorded the seller-side B2B price with freight included.

## Decision

Every imported market fill carries three monetary values:

- `spend`: the exact delivered amount debited from the buyer;
- `goods_spend`: offer price multiplied by filled quantity, rounded down in fixed-point arithmetic;
- `freight_spend`: `spend - goods_spend`, including any residual rounding unit.

Local fills use `goods_spend == spend`, zero freight, and no route. Imported fills retain the selected route. Settlement credits the goods seller only `goods_spend` and credits the route's carrier `freight_spend`. A market route without a registered carrier is invalid at settlement.

Household goods revenue remains final-sales revenue and forms the sales-tax base. Carrier freight revenue is ordinary non-final firm revenue: it enters cash flow, accounts, expectations, and management evidence but is not taxed as final household-goods turnover. B2B procurement remains entirely non-final.

Buyer-side procurement observations retain total delivered spend, because freight is a real input cost. Seller-side market outcomes use the goods offer price, because freight is not supplier revenue.

The existing `MarketTrade.spend` and `FirmProcurementTrade.spend` fields keep total-buyer-spend semantics. Two appended event variants expose the carrier leg without changing existing event shapes: `MarketFreightPaid` and `FirmProcurementFreightPaid`. The yearly chronicle aggregates both as route-carrier revenue.

## Consequences

- Buyer outflow exactly equals goods-seller inflow plus carrier inflow; no rounding money is created or destroyed.
- Route carriers now earn auditable revenue from immediate retail and B2B imports.
- Household freight no longer inflates the seller's taxable final-sales base.
- Firm buyers still observe the full delivered input price, while suppliers observe their actual goods price.
- Focused tests use separate seller and carrier firms and prove cash allocation, accounting classification, typed evidence, replay, and stable fingerprint equality.
- The rule changes authoritative cash, accounting, expectations, tax, and long-run history, so `SIMULATION_VERSION` advances from 35 to 36.
- Full workspace validation passes 125 tests. The seed-1 50-year fingerprint is `6856032423052036241`; release runtime is 27.6 ms/year.

## Follow-up

Carrier revenue is now real enough to support a later route-capacity investment decision. That later rule should respond to observed freight demand, margins, congestion, and financing rather than an artificial infrastructure trigger.
