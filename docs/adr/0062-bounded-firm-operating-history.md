# ADR 0062: Bounded firm operating history

- Status: Accepted foundation
- Date: 2026-07-23

Each firm may retain up to twelve explicitly captured operating observations. An observation contains its simulation date, realized sales revenue, actually produced batches, and the current regional prices of every recipe input. Capture must occur after monthly sales and production are recorded and before monthly accounts are reset.

The history is authoritative, serializable, replayable, fingerprinted state. When the thirteenth observation is added, the oldest is removed deterministically. Forecast derivation uses integer arithmetic means of observed sales, produced batches, and per-good input prices; without history it retains the previous current-period fallback.

Capturing or averaging observations does not move cash, inventory, or workers. The history records only information available to the firm. It does not yet contain unsatisfied customer orders, lost sales, supplier quotes, contractual prices, or confidence weights. Those require concrete market and information records rather than inferred demand scores.
