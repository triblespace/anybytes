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
- added `Bytes::try_unwrap_owner` to reclaim the owner when uniquely held
- simplified `Bytes::try_unwrap_owner` implementation
- fixed potential UB in `Bytes::try_unwrap_owner` for custom owners
- prevent dangling `data` by dropping references before unwrapping the owner
- refined `Bytes::try_unwrap_owner` to cast the data slice to a pointer only
  when the owner type matches
- replaced `ByteOwner::as_any` with trait upcasting for simpler downcasting
- rewrote `winnow::view` to use safe helpers and added `view_elems(count)` parser
- `winnow::view_elems` now returns a Parser closure for idiomatic usage
- replaced `ByteOwner::as_any` with trait upcasting to `Any`
- `Bytes::downcast_to_owner` and `View::downcast_to_owner` now return `Result`
  and return the original value on failure
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
- added `Bytes::map_file` helper for convenient file mapping
  (accepts any `memmap2::MmapAsRawDesc`, e.g. `&File` or `&NamedTempFile`)
- added `Bytes::map_file_region` to map a specific region of a file
- reverted automatic installation of Python development packages in the
  preflight script; rely on the system `python3-dev` package
- set the preflight script to use Python 3.12 for building pyo3 code
- added README example demonstrating `Bytes::try_unwrap_owner`
- expanded `ByteOwner` trait docs to clarify lifetime requirements and trait
  upcasting for downcasting
- removed rope-like store integration and async wrappers from the inventory
- noted new suggestions in `INVENTORY.md` for future work
- clarified that implementing `ByteSource` for `Arc` types would double wrap the
  owner and updated `INVENTORY.md` accordingly
- removed the `serde` support idea from the inventory
- removed the unsafe derive macro idea from the inventory
- removed the `Iterator` support idea from the inventory as `Bytes` already
  dereferences to `[u8]`
- documented creating `Bytes` from `Arc` sources without an extra wrapper and
  removed the corresponding task from the inventory
- implemented `bytes::Buf` for `Bytes` and `From<Bytes>` for `bytes::Bytes` for
  seamless integration with Tokio and other libraries

## 0.19.3 - 2025-05-30
- implemented `Error` for `ViewError`

## 0.19.2 - 2025-01-24
- removed `Sized` constraint from view methods

## 0.19.1 - 2025-01-24
- removed `Sized` bound on `.bytes()`

## 0.19.0 - 2025-01-24
- reworked `take_*` helpers and updated conversions
