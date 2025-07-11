#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Ensure required tools are available
rustup component add rustfmt

# Run formatting check and tests
cargo fmt -- --check
cargo test --all-features
