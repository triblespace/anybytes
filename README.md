A small byte management library, that can abstract over various byte owning types, like `Vec`, `bytes::Bytes`, or `memmap2::Mmap`.

## Comparison

| --- | Extensible | Zerocopy Integration | mmap support |
| --- | ---------- | -------------------- | ------------ |
| anybytes | ✅ | ✅ | ✅ |
| [bytes](https://crates.io/crates/bytes)| ✅ | ❌ | ❌ |
| [ownedbytes](https://crates.io/crates/ownedbytes)| ✅ | ❌ | ✅ |
| [minibytes](https://crates.io/crates/esl01-minibytes) [^1]| ✅ | ❌ | ✅ |

[^1]: No longer maintained as an individual crate.

## Acknowledgements
This library started as a fork of the minibyte library in facebooks [sapling scm](https://github.com/facebook/sapling).
