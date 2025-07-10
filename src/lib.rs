/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Core byte container types and traits.
pub mod bytes;
mod sources;

#[cfg(feature = "zerocopy")]
/// Types for zero-copy viewing of structured data.
pub mod view;

#[cfg(feature = "pyo3")]
/// Python bindings for [`Bytes`].
pub mod pybytes;

#[cfg(test)]
mod tests;

pub use crate::bytes::ByteOwner;
pub use crate::bytes::ByteSource;
pub use crate::bytes::Bytes;
pub use crate::bytes::WeakBytes;
#[cfg(feature = "pyo3")]
pub use crate::pybytes::PyBytes;
#[cfg(feature = "zerocopy")]
pub use crate::view::View;

/// Erase the lifetime of a reference.
///
/// # Safety
/// The caller must guarantee that the referenced data remains valid for the
/// `'static` lifetime.
unsafe fn erase_lifetime<'a, T: ?Sized>(slice: &'a T) -> &'static T {
    &*(slice as *const T)
}
