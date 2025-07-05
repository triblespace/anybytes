/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Implementations of [`ByteSource`] for common byte containers.
//!
//! | Feature      | Implementations                                                   |
//! | ------------ | ---------------------------------------------------------------- |
//! | `zerocopy`   | `&'static [T]`, `Box<T>` and `Vec<T>` for `T: IntoBytes + Immutable` |
//! | *(none)*     | `&'static [u8]`, `Box<[u8]>`, `Vec<u8>`, `String`, `&'static str` |
//! | `bytes`      | `bytes::Bytes`                                                   |
//! | `ownedbytes` | `ownedbytes::OwnedBytes`                                         |
//! | `mmap`       | `memmap2::Mmap` and `ByteOwner` for `memmap2::MmapRaw`           |
//! | `pyo3`       | `pyo3::Bound<'_, PyBytes>` and `ByteOwner` for `Py<PyBytes>`     |
//!
//! To store bytes in your own type, implement [`ByteSource`].
//! [`ByteOwner`] is provided automatically for all `ByteSource`s but can be
//! implemented manually if needed:
//!
//! ```rust
//! use anybytes::{ByteSource, Bytes};
//!
//! struct MyData(Vec<u8>);
//!
//! unsafe impl ByteSource for MyData {
//!     type Owner = Self;
//!
//!     fn as_bytes(&self) -> &[u8] { &self.0 }
//!     fn get_owner(self) -> Self::Owner { self }
//! }
//!
//! # let _ = Bytes::from_source(MyData(vec![1, 2, 3]));
//! ```

use zerocopy::Immutable;
#[cfg(feature = "zerocopy")]
use zerocopy::IntoBytes;

#[allow(unused_imports)]
use crate::{bytes::ByteOwner, ByteSource};

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteSource for &'static [T]
where
    T: IntoBytes + Immutable + Sync + Send + 'static,
{
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        IntoBytes::as_bytes(*self)
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteSource for &'static [u8] {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        *self
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteSource for Box<T>
where
    T: IntoBytes + Immutable + ?Sized + Sync + Send + 'static,
{
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        let inner = self.as_ref();
        IntoBytes::as_bytes(inner)
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteSource for Box<[u8]> {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteSource for Vec<T>
where
    T: IntoBytes + Immutable + Sync + Send + 'static,
{
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        let slice: &[T] = self.as_ref();
        IntoBytes::as_bytes(slice)
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteSource for Vec<u8> {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

unsafe impl ByteSource for String {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

unsafe impl ByteSource for &'static str {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        (*self).as_bytes()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "bytes")]
unsafe impl ByteSource for bytes::Bytes {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "ownedbytes")]
unsafe impl ByteSource for ownedbytes::OwnedBytes {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "mmap")]
unsafe impl ByteSource for memmap2::Mmap {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    fn get_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "mmap")]
impl ByteOwner for memmap2::MmapRaw {
    fn as_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any + Sync + Send> {
        self
    }
}

#[cfg(feature = "pyo3")]
impl ByteOwner for pyo3::Py<pyo3::types::PyBytes> {
    fn as_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any + Sync + Send> {
        self
    }
}

#[cfg(feature = "pyo3")]
unsafe impl<'py> ByteSource for pyo3::Bound<'py, pyo3::types::PyBytes> {
    type Owner = pyo3::Py<pyo3::types::PyBytes>;

    fn as_bytes(&self) -> &[u8] {
        pyo3::types::PyBytesMethods::as_bytes(self)
    }

    fn get_owner(self) -> Self::Owner {
        self.unbind()
    }
}
#[cfg(kani)]
mod verification {
    use std::sync::Arc;

    use crate::Bytes;

    use super::*;

    static STATIC_U8: [u8; 32] = [0; 32];

    #[kani::proof]
    #[kani::unwind(33)]
    pub fn check_static() {
        let source: &'static [u8] = &STATIC_U8;
        let bytes = Bytes::from_source(source);
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(source, bytes_slice)
    }

    #[kani::proof]
    #[kani::unwind(33)]
    pub fn check_box() {
        let owner: Box<[u8]> = STATIC_U8.into();
        let arc = Arc::new(owner);
        let bytes = Bytes::from_owning_source_arc(arc.clone());
        let arc_slice: &[u8] = &arc;
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(arc_slice, bytes_slice)
    }

    #[cfg(feature = "zerocopy")]
    #[derive(zerocopy::TryFromBytes, zerocopy::IntoBytes, zerocopy::Immutable, Clone, Copy)]
    #[repr(C)]
    struct ComplexZC {
        a: u64,
        b: [u8; 4],
        c: u32,
    }

    #[cfg(feature = "zerocopy")]
    static STATIC_ZC: [ComplexZC; 32] = [ComplexZC {
        a: 42,
        b: [0; 4],
        c: 9000,
    }; 32];

    #[cfg(feature = "zerocopy")]
    #[kani::proof]
    #[kani::unwind(513)]
    pub fn check_static_zeroconf() {
        use crate::Bytes;

        let owner: &'static [ComplexZC] = &STATIC_ZC;
        let bytes = Bytes::from_source(owner);
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(owner.as_bytes(), bytes_slice)
    }

    #[cfg(feature = "zerocopy")]
    #[kani::proof]
    #[kani::unwind(513)]
    pub fn check_box_zeroconf() {
        use crate::Bytes;

        let owner: Box<[ComplexZC]> = STATIC_ZC.into();
        let arc = Arc::new(owner);
        let bytes = Bytes::from_owning_source_arc(arc.clone());
        let arc_slice: &[u8] = arc.as_bytes();
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(arc_slice, bytes_slice)
    }
}
