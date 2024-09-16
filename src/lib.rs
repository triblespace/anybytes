/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! # minibytes
//!
//! This create provides the [`Bytes`] type. It is similar to `&[u8]`: cloning
//! or slicing are zero-copy. Unlike `&[u8]`, `Bytes` does not have lifetime.
//! This is done by maintaining the life cycle of the underlying storage using
//! reference count.
//!
//! Aside from supporting `Vec<u8>` as the underlying storage, [`Bytes`] also
//! supports [`memmap2::Mmap`]. Libraries can implement [`BytesOwner`] for other
//! types to further extend storage support.

pub mod bytes;
mod owners;

#[cfg(feature = "zerocopy")]
pub mod packed;

#[cfg(test)]
mod tests;

pub use crate::bytes::ByteOwner;
pub use crate::bytes::Bytes;
pub use crate::bytes::WeakBytes;
#[cfg(feature = "zerocopy")]
pub use crate::packed::Packed;
#[cfg(feature = "zerocopy")]
pub use crate::packed::PackedSlice;
#[cfg(feature = "zerocopy")]
pub use crate::packed::PackedStr;
