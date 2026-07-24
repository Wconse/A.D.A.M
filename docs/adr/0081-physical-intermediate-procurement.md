# ADR 0081: Physical intermediate procurement

## Status

Accepted

## Context

The producing Stage 0 economy could turn labor directly into final goods. Firm-to-firm inputs were absent, so cash, ownership of inventories, input scarcity, and value-added accounting were not causally connected. Adding B2B turnover as an accounting number would double-count output and allow production without physical materials.

## Decision

The monthly commercial chain now performs production from opening working stock, settles local intermediate procurement, and only then opens the household market. This is a one-cycle working-capital model: buyers hold one production cycle of inputs, consume them, and replenish from goods that suppliers physically produced in the same month for the next cycle.

Procurement obeys these invariants:

- an order is the input shortfall for the authorized, labor- and capacity-bounded production plan;
- sellers can transfer only inventory they own;
- buyers can spend only cash they own;
- local offers are selected deterministically by region, good, unit price, and seller ID;
- a firm cannot buy from itself;
- settlement debits buyer cash and seller inventory and credits seller cash and buyer inventory;
- B2B sales enter seller revenue, tax accounts, and management observations;
- contracted intermediate sales are observed as sold supply, so management does not mistake them for absent demand;
- B2B turnover is not final consumption and is never added directly to regional output;
- regional inventory accounting cancels local ownership transfers, while later input consumption reduces aggregate inventory and therefore subtracts intermediate consumption from value added.

Stage 0 uses energy as a minimal intermediate input for food, housing, and healthcare. Input users begin with one production cycle of working stock. Energy capacity covers both intermediate contracts and household demand, preventing a circular same-month bootstrap.

## Consequences

The executable chain now contains physical intermediate goods, cash settlement, inventory ownership, liquidity limits, a working-capital lag, seller taxation, and value-added closure. Input scarcity can stop production without creating goods or credit from nowhere. Long runs can expose supply-chain failure rather than hiding it behind final-good recipes.

The model still assumes spot local procurement and immediate settlement. Trade credit, contracts, transport, imports, and input-price accounting remain later vertical slices.
