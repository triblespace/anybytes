#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Ensure required tools are available
# Install rustfmt. The command is idempotent so we simply ignore failures.
cargo install rustfmt || true

# Install the Kani verifier as needed; this is also idempotent.
cargo install --locked kani-verifier || true

# Install cargo-fuzz to run libFuzzer targets. This subcommand is optional
# during development so we ignore installation failures here as well.
cargo install cargo-fuzz || true

# Ensure the nightly toolchain is present for cargo-fuzz and required LLVM tools
rustup toolchain install nightly || true
rustup component add llvm-tools-preview --toolchain nightly || true

# Run all Kani proofs in the workspace
cargo kani --workspace --all-features

# Execute fuzz targets with deterministic settings so verification stays
# reproducible. The default run count can be overridden by setting FUZZ_ARGS.
FUZZ_TARGET="${FUZZ_TARGET:-bytes_mut_ops}"
FUZZ_ARG_STRING="${FUZZ_ARGS:-}"
if [[ -n "$FUZZ_ARG_STRING" ]]; then
  IFS=' ' read -r -a FUZZ_ARG_ARRAY <<< "$FUZZ_ARG_STRING"
else
  FUZZ_ARG_ARRAY=(-seed=1 -runs=50000)
fi

cargo +nightly fuzz run "$FUZZ_TARGET" -- "${FUZZ_ARG_ARRAY[@]}"

