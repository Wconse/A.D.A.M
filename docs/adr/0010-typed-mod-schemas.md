# ADR 0010: Typed mod schemas and cross-registry references

Goods and production recipes have strict Serde schemas. After per-entry decoding, a second validation pass resolves namespaced references across typed registries and accumulates every missing reference and invalid physical value before world construction.
