#!/bin/bash
set -e
(
  set -x
  cargo test --all-features
)
if [[ "$1" == '-f' ]]; then
  set -x
  cargo clippy --fix --allow-dirty --all-targets --all-features -- -D warnings
  cargo fmt --all
  git diff
else
  set -x
  cargo clippy --all-targets --all-features -- -D warnings
  cargo fmt --all --check
fi
