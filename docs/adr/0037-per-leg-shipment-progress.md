# ADR 0037: Per-leg shipment progression

- Status: Accepted foundation
- Date: 2026-07-22

Multi-leg shipments require explicit progress through each route rather than one aggregate arrival timer. The lifecycle stores ordered routes, transit duration per leg, current leg, and remaining leg days. A single time advance may complete several legs deterministically and reports each completed route. Terminal dwell, leg-entry capacity periods, and authoritative World migration remain subsequent layers.
