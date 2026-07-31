# ADR 0137: Bounded household supplier loyalty

## Status
Accepted.

## Decision
After each settled market, a household cohort remembers the seller that supplied the largest quantity of each good. In later local clearing, that supplier is considered first only while its price is no more than ten percent above the cheapest available local offer. If it is unavailable or exceeds the premium bound, the household switches to the cheaper supplier. Ties remain canonical by firm ID. Preferences persist without decaying when a good is not purchased and are included in the stable fingerprint.

## Consequences
Firms compete for durable customers rather than only winning an isolated price sort. A small reputation-like premium becomes viable, but loyalty cannot trap households into severe overpayment. Supply failure and aggressive markup create a concrete path for competitors to gain market share.
