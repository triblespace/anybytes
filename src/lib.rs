/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(all(feature = "mmap", feature = "zerocopy"))]
pub mod area;
/// Core byte container types and traits.
pub mod bytes;
mod sources;

#[cfg(feature = "zerocopy")]
/// Types for zero-copy viewing of structured data.
pub mod view;

#[cfg(feature = "pyo3")]
/// Python bindings for [`Bytes`].
pub mod pyanybytes;

#[cfg(feature = "winnow")]
/// Integration with the `winnow` parser library.
pub mod winnow;

#[cfg(test)]
mod tests;

#[cfg(all(feature = "mmap", feature = "zerocopy"))]
pub use crate::area::{ByteArea, Section, SectionWriter};
pub use crate::bytes::ByteOwner;
pub use crate::bytes::ByteSource;
pub use crate::bytes::Bytes;
pub use crate::bytes::WeakBytes;
#[cfg(feature = "pyo3")]
pub use crate::pyanybytes::PyAnyBytes;
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
