![Crates.io Version](https://img.shields.io/crates/v/anybytes)
![docs.rs](https://img.shields.io/docsrs/anybytes)
![Discord Shield](https://discordapp.com/api/guilds/795317845181464651/widget.png?style=shield)

**This Library is still pre-0.1.0 the API is therefore in heavy flux, and everything should be considered alpha!**

A small library for conveniently working with immutables bytes from different sources, providing zero-copy slicing and cloning.

Access itself is extremely cheap via no-op conversion to a `&[u8]`.
 
The storage mechanism backing the bytes can be extended
and is implemented for a variety of sources already,
including other byte handling crates `Bytes`, mmap-ed files,
`String`s and `Zerocopy` types.

See `INVENTORY.md` for notes on possible cleanup and future functionality.

## Overview

`Bytes` decouples data access from lifetime management through two traits:
[`ByteSource`](src/bytes.rs) and [`ByteOwner`](src/bytes.rs).  A `ByteSource`
can yield a slice of its bytes and then convert itself into a `ByteOwner` that
keeps the underlying storage alive.  This separation lets callers obtain a
borrow of the bytes, drop any locks or external guards, and still retain the
data by storing the owner behind an `Arc`.  No runtime indirection is required
when constructing a `Bytes`, and custom storage types integrate by
implementing `ByteSource`.

## Quick Start

```rust
use anybytes::Bytes;

fn main() {
    // create `Bytes` from a vector
    let bytes = Bytes::from(vec![1u8, 2, 3, 4]);

    // take a zero-copy slice
    let slice = bytes.slice(1..3);

    // convert it to a typed View
    let view = slice.view::<[u8]>().unwrap();
    assert_eq!(&*view, &[2, 3]);
}
```

The full example is available in [`examples/quick_start.rs`](examples/quick_start.rs).

## Arc Sources

`Bytes` can also be created directly from an `Arc` holding a byte container.
This avoids allocating another `Arc` wrapper:

```rust
use anybytes::Bytes;
use std::sync::Arc;

let data = Arc::new(vec![1u8, 2, 3, 4]);
let bytes = Bytes::from(data.clone());
assert_eq!(bytes.as_ref(), data.as_slice());
```

Implementing `ByteSource` for `Arc<[u8]>` or `Arc<Vec<u8>>` is therefore
unnecessary, since `Bytes::from` already reuses the provided `Arc`.

## Reclaiming Ownership

`Bytes::try_unwrap_owner` allows recovering the original owner when no other
references exist.

```rust
use anybytes::Bytes;

let bytes = Bytes::from(vec![1u8, 2, 3]);
let vec = bytes.try_unwrap_owner::<Vec<u8>>().expect("unique owner");
assert_eq!(vec, vec![1, 2, 3]);
```

## Advanced Usage

`Bytes` can directly wrap memory-mapped files or other large buffers.  Combined
with the [`view`](src/view.rs) module this enables simple parsing of structured
data without copying:

```rust
use anybytes::Bytes;
use zerocopy::{FromBytes, Immutable, KnownLayout};

#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
struct Header { magic: u32, count: u32 }

// `file` can be any type that implements `memmap2::MmapAsRawDesc` such as
// `&std::fs::File` or `&tempfile::NamedTempFile`.
fn read_header(file: &std::fs::File) -> std::io::Result<anybytes::view::View<Header>> {
    let bytes = unsafe { Bytes::map_file(file)? };
    Ok(bytes.view().unwrap())
}
```

To map only a portion of a file use the unsafe helper
`Bytes::map_file_region(file, offset, len)`.

### Byte Area

Use `ByteArea` to incrementally build immutable bytes on disk:

```rust
use anybytes::area::ByteArea;

let mut area = ByteArea::new().unwrap();
let mut sections = area.sections();
let mut section = sections.reserve::<u8>(4).unwrap();
section.copy_from_slice(b"test");
let bytes = section.freeze().unwrap();
assert_eq!(bytes.as_ref(), b"test".as_ref());
drop(sections);
let all = area.freeze().unwrap();
assert_eq!(all.as_ref(), b"test".as_ref());
```

Call `area.persist(path)` to keep the temporary file instead of mapping it.

The area only aligns allocations to the element type and may share pages
between adjacent sections to minimize wasted space. Multiple sections may be
active simultaneously; their byte ranges do not overlap.

## Features

By default the crate enables the `mmap` and `zerocopy` features.
Other optional features provide additional integrations:

- `bytes` &ndash; support for the [`bytes`](https://crates.io/crates/bytes) crate so `bytes::Bytes` can act as a `ByteSource`.
- `ownedbytes` &ndash; adds compatibility with [`ownedbytes`](https://crates.io/crates/ownedbytes) and implements its `StableDeref` trait.
- `mmap` &ndash; enables memory-mapped file handling via the `memmap2` crate.
- `zerocopy` &ndash; exposes the [`view`](src/view.rs) module for typed zero-copy access and allows using `zerocopy` types as sources.
- `pyo3` &ndash; builds the [`pybytes`](src/pybytes.rs) module to provide Python bindings for `Bytes`.
- `winnow` &ndash; implements the [`Stream`](https://docs.rs/winnow/) traits for `Bytes` and offers parsers (`view`, `view_elems(count)`) that return typed `View`s.

Enabling the `pyo3` feature requires the Python development headers and libraries
(for example `libpython3.x`). Running `cargo test --all-features` therefore
needs these libraries installed; otherwise disable the feature during testing.

## Examples

- [`examples/quick_start.rs`](examples/quick_start.rs) – the quick start shown above
- [`examples/try_unwrap_owner.rs`](examples/try_unwrap_owner.rs) – reclaim the owner when uniquely referenced
- [`examples/pybytes.rs`](examples/pybytes.rs) – demonstrates the `pyo3` feature using `PyBytes`
- [`examples/from_python.rs`](examples/from_python.rs) – wrap a Python `bytes` object into `Bytes`

## Comparison

| Crate | Active | Extensible | mmap support | Zerocopy Integration | Pyo3 Integration | kani verified |
| ----- | ------ | ---------- | ------------ | -------------------- | ---------------- | -------- |
| anybytes | ✅ | ✅ | ✅ | ✅ | ✅ | 🚧 |
| [bytes](https://crates.io/crates/bytes) | ✅ | ✅ | ✅[^1] | ❌ | ❌ | ❌ |
| [ownedbytes](https://crates.io/crates/ownedbytes) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [minibytes](https://crates.io/crates/sapling-minibytes) | ✅[^2] | ✅ | ✅ | ❌ | ❌ | ❌ |

[^1]: Recently added a new "Owned Bytes" variant, which still has all the downsides of a V-Table.
[^2]: Recently published again.

## Development

Run `./scripts/preflight.sh` from the repository root before committing. The
script formats the code and executes all tests using Python 3.12 for the `pyo3`
feature.

Kani proofs are executed separately with `./scripts/verify.sh`, which should be
run on a dedicated system. The script will install the Kani verifier
automatically. Verification can take a long time and isn't needed for quick
development iterations.

## Glossary

- [`Bytes`](src/bytes.rs) &ndash; primary container type.
- [`ByteSource`](src/bytes.rs) &ndash; trait for objects that can provide bytes.
- [`ByteOwner`](src/bytes.rs) &ndash; keeps backing storage alive.
- [`view` module](src/view.rs) &ndash; typed zero-copy access to bytes.
- [`pybytes` module](src/pybytes.rs) &ndash; Python bindings.

## Acknowledgements
This library started as a fork of the minibyte library in facebooks [sapling scm](https://github.com/facebook/sapling).

Thanks to @kylebarron for his feedback and ideas on Pyo3 integration.
