# Changelog

## Unreleased
- limit Kani loop unwind by default and set per-harness bounds
- increase unwind for prefix/suffix overflow proofs
- added crate-level examples for weak references and owner downcasting
- expanded module introduction describing use cases
- documented rationale for separating `ByteSource` and `ByteOwner`

## 0.19.3 - 2025-05-30
- implemented `Error` for `ViewError`

## 0.19.2 - 2025-01-24
- removed `Sized` constraint from view methods

## 0.19.1 - 2025-01-24
- removed `Sized` bound on `.bytes()`

## 0.19.0 - 2025-01-24
- reworked `take_*` helpers and updated conversions
