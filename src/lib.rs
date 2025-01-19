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
pub mod view;

#[cfg(feature = "pyo3")]
pub mod pybytes;

#[cfg(test)]
mod tests;

pub use crate::bytes::ByteOwner;
pub use crate::bytes::ByteSource;
pub use crate::bytes::Bytes;
pub use crate::bytes::WeakBytes;
#[cfg(feature = "zerocopy")]
pub use crate::view::View;
#[cfg(feature = "pyo3")]
pub use crate::pybytes::PyBytes;

unsafe fn erase_lifetime<'a, T: ?Sized>(slice: &'a T) -> &'static T {
    &*(slice as *const T)
}