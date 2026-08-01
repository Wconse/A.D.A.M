# Performance Baseline

Performance claims are accepted only with a reproducible command and representative data shape.

## Foundation graph construction (retired, not reproducible)

This baseline was produced by a `foundation_scale` example that no longer exists in the workspace. The command below is kept only as provenance for the recorded numbers; it will fail with `no example target named foundation_scale`. Do not cite these figures as a current measurement.

```powershell
cargo run --release -p adam-core --example foundation_scale -- 100000
```

Historical numbers on the initial Windows development machine (2026-07-21):

- 100,000 actors;
- 100,000 power nodes;
- 100,000 influence edges;
- 300,003 archived events;
- 82 ms construction time in release mode;
- deterministic fingerprint `521355a0bd96a850`.

This is a construction microbenchmark, not a simulation throughput claim. It deliberately includes ordered-map insertion, name allocation, reference validation, and event archival. The result shows that correctness-first ordered storage is sufficient for the current milestone; it does not justify retaining `BTreeMap` for future hot update loops.

## Reproduction rules

- Run release builds.
- Record CPU, toolchain, entity counts, event counts, elapsed time, and fingerprint.
- Compare the same workload and seed.
- Do not weaken determinism or validation to improve a benchmark without an ADR.
