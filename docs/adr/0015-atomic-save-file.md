# ADR 0015: Checksummed atomic save files

- Status: Accepted foundation
- Date: 2026-07-21

Save payloads are wrapped in a binary container with magic bytes, container version, exact payload length, and checksum. Writes use a synced temporary file, preserve the previous valid file as `.bak`, and rename atomically; failed final replacement attempts restore the backup. Read validates the container before decoding the world envelope.
