# ADR 0169: Cross-platform observatory build and honest acceptance

## Status

Accepted.

## Context

An external review found that `cargo build --workspace` fails on Linux with `no PlatformIcon in platform_impl`, and that the last CI run failed on `cargo check --workspace --all-targets`.

The cause is precise. `apps/adam-observatory` depends on Bevy with `default-features = false` and enables `bevy_winit`. In Bevy 0.16 the Linux windowing backend arrives through the `x11` (or `wayland`) feature, which is normally pulled in by Bevy's default features. With defaults disabled and no backend requested, `winit` compiles without a platform implementation. Windows and macOS select their backend unconditionally, so the breakage was invisible during local Windows development.

The consequence was worse than the build error itself: Step 100 was recorded in the roadmap as COMPLETE while the workspace did not compile on the only platform CI runs. A completion claim that was never verified on the verifying platform is not a minor documentation defect; it invalidates the acceptance.

A related documentation defect was found in the same review. `README.md` and `docs/performance.md` instruct the reader to run `cargo run --release -p adam-core --example foundation_scale`, but that example was deleted, so both documents describe a command that cannot run.

## Decision

Request the Linux windowing backends explicitly through a `cfg(target_os = "linux")` dependency block that enables `x11` and `wayland`, keeping the lean default-features-off feature set on every other platform. Record the reason beside the block so the features are not removed as redundant.

Treat a green cross-platform CI run as a precondition for any completion claim. Correct the Step 100 entry in the roadmap in place rather than silently rewriting it, so the record shows both the false acceptance and its correction.

Point the documentation at commands that exist. The fifty-year determinism and throughput probe is `adam-cli`, which is a real binary target. Mark the retired `foundation_scale` figures as historical provenance instead of presenting them as reproducible.

## Invariants

1. The workspace compiles on Linux, Windows, and macOS.
2. Disabling Bevy default features never removes a platform windowing backend.
3. A step is not COMPLETE while CI is red.
4. Documented commands refer to targets that exist in the workspace.
5. Performance figures are either reproducible by a current command or explicitly marked retired.

## Consequences

CI can compile the workspace again, so the quality gate becomes meaningful rather than decorative. The roadmap now carries an explicit correction, which is preferable to a clean but false history. The retired performance baseline stays visible as provenance without being citable as a current measurement.

This ADR deliberately does not address the two substantive findings from the same review: the demo economy collapses on every seed, and the chronicle is unreadable with `importance` saturated at 100. Those are simulation-design defects, not build defects, and are recorded separately so that a build fix is never mistaken for an economic fix.
