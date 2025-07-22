#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Ensure required tools are available
rustup component add rustfmt

# Use Python 3.12 for pyo3 builds when available
PYTHON_BIN="python3.12"
if command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  export PYO3_PYTHON="$PYTHON_BIN"
else
  echo "warning: $PYTHON_BIN not found; using default python3" >&2
fi

# Run formatting check and tests
cargo fmt -- --check
cargo test --all-features
