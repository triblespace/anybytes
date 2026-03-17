# Inventory

## Potential Removals
- None at the moment.

## Desired Functionality
- Add Kani proofs for winnow view helpers.
- Add test covering freezing an empty section to guard against flush errors on macOS.
- Explore how to model `ByteArea` for Kani or fuzzing without depending on OS-backed memory maps.

## Discovered Issues
- `Bytes::from_source` with `Box<T>` triggers a Stacked Borrows violation
  because moving the Box invalidates prior tags on its heap allocation. This is
  a known limitation of Stacked Borrows with Box; Tree Borrows handles it
  correctly. The Miri tests use Tree Borrows (`-Zmiri-tree-borrows`)
  accordingly.
- The `test_winnow_view_parser` test fails under Miri because it assumes the
  `Vec<u8>` allocation is 2-byte aligned for a `u16`-containing struct; Miri
  doesn't guarantee this alignment.
