# Changelog

## Unreleased
- split Kani verification into `verify.sh` and streamline `preflight.sh`
- clarify that `verify.sh` runs on a dedicated system and document avoiding async code
- install `rustfmt` and the Kani verifier automatically via `cargo install`
- restore Kani proof best practices in `AGENTS.md` and note that proofs run via `verify.sh`
- limit Kani loop unwind by default and set per-harness bounds
- increase unwind for prefix/suffix overflow proofs
- move weak reference and downcasting examples into module docs
- expand module introduction describing use cases
- document rationale for separating `ByteSource` and `ByteOwner`
- added optional `winnow` feature for parser integration
- added `INVENTORY.md` for tracking future work and noted it in `AGENTS.md`
- documented safety rationale for `winnow` integration
- implemented `Stream` directly for `Bytes` with a safe `iter_offsets` iterator
- added `pop_back` and `pop_front` helpers and rewrote parser examples
- removed the Completed Work section from `INVENTORY.md` and documented its use
- rewrote `winnow::view` to use safe helpers and added `view_elems(count)` parser
- `winnow::view_elems` now returns a Parser closure for idiomatic usage
  in a dedicated AGENTS section
- add tests for weak reference upgrade/downgrade and Kani proofs for view helpers
- add examples for quick start and PyBytes usage
- add example showing how to wrap Python `bytes` into `Bytes`
- summarize built-in `ByteSource`s and show how to extend them
- added tests verifying `WeakView` upgrade and drop semantics
- clarify library overview and development instructions in README
- added crate-level examples for weak references and owner downcasting
- expanded module introduction describing use cases
- update bytes, ownedbytes, memmap2, zerocopy and pyo3 dependencies
- documented rationale for separating `ByteSource` and `ByteOwner`
- verify `cargo fmt` availability and install `rustfmt` via rustup if missing
- note that the `pyo3` feature requires Python development libraries
- documented safety requirements for `erase_lifetime`
- warn about missing documentation by enabling the `missing_docs` lint
- derive `Clone` and `Debug` for `WeakBytes` and `WeakView`
- replaced `quickcheck` property tests with `proptest`
- added `ByteSource` support for `memmap2::MmapMut` and `Cow<'static, [T]>` with `zerocopy`
- split `Cow` ByteSource tests into dedicated cases
- skip Python examples when the `pyo3` feature is disabled to fix `cargo test`

## 0.19.3 - 2025-05-30
- implemented `Error` for `ViewError`

## 0.19.2 - 2025-01-24
- removed `Sized` constraint from view methods

## 0.19.1 - 2025-01-24
- removed `Sized` bound on `.bytes()`

## 0.19.0 - 2025-01-24
- reworked `take_*` helpers and updated conversions
