# ADR 0065: Authorized firm production targets

- Status: Accepted foundation
- Date: 2026-07-23

A firm may now retain an explicit monthly batch target through the replayable `SetFirmProductionTarget` command. The command names the deciding actor and is authorized under a dedicated operations-scoped corporate action. Majority voting owners, chief executives, and operations managers may set the target; unrelated actors may not.

A target may be zero but may not exceed installed batch capacity. Labor and intermediate inputs remain execution-time constraints rather than target-validation constraints. Monthly production therefore executes the minimum of management target, installed capacity, active-agreement labor, and available recipe inputs. Firms without an explicit target retain the previous behavior of attempting their physical maximum.

Production targets are authoritative world state. They are serialized in saves, included in stable fingerprints, reproduced by command replay, and recorded through a typed event with the actor, previous target, and new target. Invalid or unauthorized changes are atomic and emit no event.

The advisory production planner intentionally evaluates physical feasibility without applying the current management target. This allows advice to recommend increasing a deliberately low target while execution continues to obey the currently authorized decision.
