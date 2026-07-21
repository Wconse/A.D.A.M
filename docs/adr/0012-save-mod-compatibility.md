# ADR 0012: Save and mod-set compatibility contract

- Status: Accepted foundation
- Date: 2026-07-21

Save metadata records simulation version, world schema version, ordered mod IDs and versions, exact package fingerprint, and canonical post-merge content fingerprint. Compatibility checks report each mismatch independently rather than silently loading a different world. Registry conflict reports expose every definition changed by multiple mod layers and its ordered provenance chain.
