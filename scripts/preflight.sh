#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Ensure required tools are available
# Install rustfmt unconditionally. The command is idempotent and will
# skip installation if the tool is already available.
cargo install rustfmt || true

# Run formatting check and tests
cargo fmt -- --check
cargo test --all-features
