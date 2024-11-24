/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![doc = include_str!("../README.md")]

pub mod bytes;
mod owners;

#[cfg(feature = "zerocopy")]
pub mod packed;

#[cfg(test)]
mod tests;

pub use crate::bytes::ByteSource;
pub use crate::bytes::Bytes;
pub use crate::bytes::WeakBytes;
#[cfg(feature = "zerocopy")]
pub use crate::packed::Packed;
#[cfg(feature = "zerocopy")]
pub use crate::packed::PackedSlice;
#[cfg(feature = "zerocopy")]
pub use crate::packed::PackedStr;
