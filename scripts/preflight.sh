#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Ensure required tools are available
if ! cargo fmt --version >/dev/null 2>&1; then
  echo "cargo fmt not found. Installing rustfmt via rustup..."
  rustup component add rustfmt
fi

if ! cargo kani --version >/dev/null 2>&1; then
  echo "cargo-kani not found. Installing Kani verifier..."
  cargo install --locked kani-verifier
fi

# Run formatting check, tests, and Kani verification
cargo fmt -- --check
cargo test --all-features
cargo kani --workspace --all-features
