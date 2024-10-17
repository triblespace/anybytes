**This Library is still pre-1.0.0 the API is therefore in heavy flux!**

A small library for conveniently working with immutables bytes from different sources, providing zero-copy slicing and cloning.

Access itself is extremely cheap via no-op conversion to a `&[u8]`.
 
The storage mechanism backing the bytes can be extended
and is implemented for a variety of sources already,
including other byte handling crates `Bytes`, mmap-ed files,
`String`s and `Zerocopy` types.

## Comparison

| Crate | Active | Extensible | Zerocopy Integration | mmap support | kani verified |
| ----- | ------ | ---------- | -------------------- | ------------ | -------- |
| anybytes | ✅ | ✅ | ✅ | ✅ | 🚧 |
| [bytes](https://crates.io/crates/bytes) | ✅ | ✅ | ❌ | ❌ | ❌ |
| [ownedbytes](https://crates.io/crates/ownedbytes) | ✅ | ✅ | ❌ | ✅ | ❌ |
| [minibytes](https://crates.io/crates/esl01-minibytes) | ❌[^1] | ✅ | ❌ | ✅ | ❌ |

[^1]: No longer maintained as an individual crate.

## Acknowledgements
This library started as a fork of the minibyte library in facebooks [sapling scm](https://github.com/facebook/sapling).
