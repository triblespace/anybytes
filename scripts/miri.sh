#!/usr/bin/env bash
set -euo pipefail

# Move to repository root
cd "$(dirname "$0")/.."

# Run Miri tests with Tree Borrows.
#
# Tree Borrows is the successor to Stacked Borrows and is the recommended
# model for new code. The from_source path has a known Stacked Borrows
# incompatibility with Box (moving a Box invalidates prior tags on its
# allocation) that Tree Borrows handles correctly.
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test --test miri "$@"
