$ErrorActionPreference = "Stop"

$toolsRoot = "E:\ADAM-Tools"
$cargoBin = Join-Path $toolsRoot "cargo\bin"
if (Test-Path (Join-Path $cargoBin "cargo.exe")) {
    $env:CARGO_HOME = Join-Path $toolsRoot "cargo"
    $env:RUSTUP_HOME = Join-Path $toolsRoot "rustup"
    $env:CARGO_TARGET_DIR = Join-Path $toolsRoot "target\ADAM"
    $env:PATH = "$cargoBin;$env:PATH"
}

function Assert-NativeSuccess {
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

cargo fmt --all -- --check
Assert-NativeSuccess
cargo check --workspace --all-targets
Assert-NativeSuccess
cargo clippy --workspace --all-targets -- -D warnings
Assert-NativeSuccess
cargo test --workspace
Assert-NativeSuccess
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --workspace --no-deps
Assert-NativeSuccess
