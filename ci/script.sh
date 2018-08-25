#!/bin/sh

set -ex

cargo doc --verbose
cargo build --verbose
cargo test --verbose
cargo build --verbose --manifest-path kodama-capi/Cargo.toml
cargo build --verbose --manifest-path kodama-bin/Cargo.toml

if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then
  cargo bench --verbose --no-run
fi
