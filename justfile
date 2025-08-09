#!/usr/bin/env just --justfile

# if running in CI, treat warnings as errors by setting RUSTFLAGS and RUSTDOCFLAGS to '-D warnings' unless they are already set
# Use `CI=true just ci-test` to run the same tests as in GitHub CI.
# Use `just env-info` to see the current values of RUSTFLAGS and RUSTDOCFLAGS
ci_mode := if env('CI', '') != '' {'1'} else {''}
export RUSTFLAGS := env('RUSTFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUSTDOCFLAGS := env('RUSTDOCFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUST_BACKTRACE := env('RUST_BACKTRACE', if ci_mode == '1' {'1'} else {''})

@_default:
    {{just_executable()}} --list

# Quick compile without building a binary
check:
    cargo check
    @echo "--------------  Checking individual crate features"
    cargo check --features pmtiles
    cargo check --features geo
    cargo check --features geodesy
    cargo check --all-features

# Run all tests as expected by CI
ci-test: test-fmt clippy check test test-doc

# Run cargo clippy to lint the code
clippy *args:
    cargo clippy {{args}}
    cargo clippy --all-features {{args}}

# Reformat all code `cargo fmt`.
fmt:
    cargo fmt --all

# Run all unit and integration tests
test:
    cargo test
    cargo test --doc
    @echo "--------------  Testing individual crate features"
    cargo test --features pmtiles
    cargo test --features geo
    cargo test --features geodesy
    cargo test --all-features

# Build and open code documentation
docs *args='--open':
    DOCS_RS=1 cargo doc --no-deps {{args}}

# Test documentation generation
test-doc:  (docs '')

# Test code formatting
test-fmt:
    cargo fmt --all -- --check

run-examples:
    cargo run --example info data/tiff/N265E425.tif
    cargo run --example pixel data/tiff/N265E425.tif 2550 3050
    cargo run --example crop data/tiff/N265E425.tif 100x100+2500+3000 /tmp/dtm.png
    cargo run --example img2ascii data/tiff/sat.tif >/tmp/sat.txt
    cargo run --example http_dtm
