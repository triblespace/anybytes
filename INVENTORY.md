# Inventory

## Potential Removals
- None at the moment.

## Desired Functionality
- Document that `Bytes::from(Arc<Vec<u8>>)` and similar constructors already
  handle owning `Arc` types without an extra wrapper. Implementing
  `ByteSource` for `Arc<[u8]>` or `Arc<Vec<u8>>` would double-wrap the arc and is
  therefore unnecessary.
- Helper `map_file_region` to map only part of a file.
- Example demonstrating Python + winnow parsing.
- Additional Kani proofs covering `try_unwrap_owner` and weak references.

## Discovered Issues
- Missing tests for `pop_front` and `pop_back` helpers.
