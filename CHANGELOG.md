# Changelog

## Unreleased
- limit Kani loop unwind by default and set per-harness bounds
- increase unwind for prefix/suffix overflow proofs
- move weak reference and downcasting examples into module docs
- expand module introduction describing use cases
- document rationale for separating `ByteSource` and `ByteOwner`
- add tests for weak reference upgrade/downgrade and Kani proofs for view helpers
- add examples for quick start and PyBytes usage
- add example showing how to wrap Python `bytes` into `Bytes`
- summarize built-in `ByteSource`s and show how to extend them
- added tests verifying `WeakView` upgrade and drop semantics
- clarify library overview and development instructions in README
- added crate-level examples for weak references and owner downcasting
- expanded module introduction describing use cases
- documented rationale for separating `ByteSource` and `ByteOwner`
- verify `cargo fmt` availability and install `rustfmt` via rustup if missing
- note that the `pyo3` feature requires Python development libraries
- documented safety requirements for `erase_lifetime`

## 0.19.3 - 2025-05-30
- implemented `Error` for `ViewError`

## 0.19.2 - 2025-01-24
- removed `Sized` constraint from view methods

## 0.19.1 - 2025-01-24
- removed `Sized` bound on `.bytes()`

## 0.19.0 - 2025-01-24
- reworked `take_*` helpers and updated conversions
