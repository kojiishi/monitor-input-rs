#!/bin/bash
set -x
cargo test
# cargo fmt --all --check
cargo fmt --all
# cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --fix --allow-dirty --all-targets --all-features -- -D warnings
