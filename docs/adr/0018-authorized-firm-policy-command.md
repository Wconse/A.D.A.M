# ADR 0018: Authorized firm-policy commands

- Status: Accepted foundation
- Date: 2026-07-21

Firm ownership, voting control, and policy are authoritative world state. Economic and voting rights are tracked separately and cannot exceed 100% per firm. A policy command is accepted only when its actor currently holds a strict majority of voting rights. Player and AI use the same serialized `WorldCommand::SetFirmPolicy`; unauthorized commands return an error without changing state. Board authority, delegated executives, voting coalitions, share classes, and creditor covenants will extend this resolver rather than bypass it.
