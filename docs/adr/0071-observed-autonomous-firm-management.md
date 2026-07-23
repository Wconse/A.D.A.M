# ADR 0071: Observed autonomous firm management closes the production loop

- Status: Accepted foundation
- Date: 2026-07-23

The monthly economic cycle now turns observed firm outcomes into an authorized next-month production decision. After production, offer formation, clearing, settlement, and observation capture, each managed firm derives a three-month forecast from its bounded operating history. The existing production-adjustment planner compares stockouts, unsold inventory, observed sales, demand exposure, expected margin, and physical feasibility. If its advice differs from the current target, a concrete actor applies the ordinary replayable `SetFirmProductionTarget` command.

Actor selection is deterministic and authority-preserving: an operations manager is preferred, then a chief executive, then the lowest-ID actor already authorized through strict-majority ownership. Unmanaged firms are reviewed without invented control. The management stage is atomic, guarded against duplicate execution, represented in the event archive, and included in stable fingerprints.

This closes `production -> market outcome -> firm memory -> expectation -> authorized management decision -> next production`. It prevents a managed firm from producing the same unsold output indefinitely while its own advisory evidence recommends contraction. A test verifies that a target of five batches falls to zero after a month with production but no funded sales, and that direct and command replay remain identical.

The rule is intentionally a Stage 0 baseline rather than final business AI. Managers currently accept deterministic advice without risk preference, confidence thresholds, board conflict, financing strategy, strategic inventory, market-share goals, or deliberate price changes. Those dimensions should be added as competing actor objectives after the complete world-response chain is operational.
