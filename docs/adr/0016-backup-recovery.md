# ADR 0016: Validated backup recovery

- Status: Accepted foundation
- Date: 2026-07-21

Save reads validate the primary container first. On any I/O, magic, length, version, or checksum failure, the loader validates the `.bak` container and may return it with an explicit `Backup` source plus the primary diagnostic. Recovery never treats an unvalidated backup as state. If both fail, both errors are returned.
