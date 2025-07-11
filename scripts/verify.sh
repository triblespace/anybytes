#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Ensure required tools are available
# Install rustfmt. The command is idempotent so we simply ignore failures.
cargo install rustfmt || true

# Install the Kani verifier as needed; this is also idempotent.
cargo install --locked kani-verifier || true

# Run all Kani proofs in the workspace
cargo kani --workspace --all-features

