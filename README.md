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

## Features

By default the crate enables the `mmap` and `zerocopy` features.
Other optional features provide additional integrations:

- `bytes` &ndash; support for the [`bytes`](https://crates.io/crates/bytes) crate so `bytes::Bytes` can act as a `ByteSource`.
- `ownedbytes` &ndash; adds compatibility with [`ownedbytes`](https://crates.io/crates/ownedbytes) and implements its `StableDeref` trait.
- `mmap` &ndash; enables memory-mapped file handling via the `memmap2` crate.
- `zerocopy` &ndash; exposes the [`view`](src/view.rs) module for typed zero-copy access and allows using `zerocopy` types as sources.
- `pyo3` &ndash; builds the [`pybytes`](src/pybytes.rs) module to provide Python bindings for `Bytes`.

## Comparison

| Crate | Active | Extensible | mmap support | Zerocopy Integration | Pyo3 Integration | kani verified |
| ----- | ------ | ---------- | ------------ | -------------------- | ---------------- | -------- |
| anybytes | ✅ | ✅ | ✅ | ✅ | ✅ | 🚧 |
| [bytes](https://crates.io/crates/bytes) | ✅ | ✅ | ✅[^1] | ❌ | ❌ | ❌ |
| [ownedbytes](https://crates.io/crates/ownedbytes) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| [minibytes](https://crates.io/crates/sapling-minibytes) | ✅[^2] | ✅ | ✅ | ❌ | ❌ | ❌ |

[^1]: Recently added a new "Owned Bytes" variant, which still has all the downsides of a V-Table.
[^2]: Recently published again.

## Acknowledgements
This library started as a fork of the minibyte library in facebooks [sapling scm](https://github.com/facebook/sapling).

Thanks to @kylebarron for his feedback and ideas on Pyo3 integration.
