$ErrorActionPreference = "Stop"

cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --workspace --no-deps
