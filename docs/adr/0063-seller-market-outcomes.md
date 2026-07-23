# ADR 0063: Seller-side market outcomes

- Status: Accepted foundation
- Date: 2026-07-23

Market clearing now returns one canonical outcome for every seller offer: offered, sold, and unsold quantity; unit price; region and good; and total unmet demand remaining in that regional goods market. Settlement validates that outcomes conserve offer quantity and exactly match settled fills before recording them.

A firm carries these outcomes into its bounded operating history. `sold_out_while_demand_remained` is a concrete observation: the firm sold its entire offer while buyer orders for the same regional good remained unfilled. It is evidence of stockout exposure, not an exact claim that every unmet unit would have been purchased from that firm.

Unsold inventory and unmet demand may coexist because buyers lacked money, the offer price was too high, or cheaper sellers were preferred. The model therefore preserves the underlying quantities instead of assigning a generic lost-sales score. Monthly outcome evidence is cleared with monthly firm accounts after it has been captured.
