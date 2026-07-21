# ADR 0021: Scoped firm-management commands

- Status: Accepted foundation
- Date: 2026-07-21

Marketing and inventory changes are separate serialized commands. Each checks the action-specific authority matrix and updates only its policy field through immutable validated reconstruction. Marketing managers cannot alter operations; operations managers cannot alter marketing. Missing policy, invalid allocation, and unauthorized access fail without mutation.
