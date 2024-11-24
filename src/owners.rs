/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Implement [`ByteOwner`] and [`TextOwner`] for common types.

#[cfg(feature = "zerocopy")]
use zerocopy::AsBytes;

use crate::{bytes::ByteOwner, ByteSource};

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteSource for &'static [T]
where
    T: AsBytes + Sync + Send + 'static,
{
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        AsBytes::as_bytes(*self)
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteSource for &'static [u8] {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        *self
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteSource for Box<T>
where
    T: AsBytes + ?Sized + Sync + Send + 'static,
{
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        let inner = self.as_ref();
        AsBytes::as_bytes(inner)
    }
    
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteSource for Box<[u8]> {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteSource for Vec<T>
where
    T: AsBytes + Sync + Send + 'static,
{
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        let slice: &[T] = self.as_ref();
        AsBytes::as_bytes(slice)
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteSource for Vec<u8> {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
        
    fn as_owner(self) -> Self::Owner {
        todo!()
    }
}

unsafe impl ByteSource for String {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

unsafe impl ByteSource for &'static str {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        (*self).as_bytes()
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "bytes")]
unsafe impl ByteSource for bytes::Bytes {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "ownedbytes")]
unsafe impl ByteSource for ownedbytes::OwnedBytes {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
        
    fn as_owner(self) -> Self::Owner {
        self
    }
}

#[cfg(feature = "mmap")]
unsafe impl ByteSource for memmap2::Mmap {
    type Owner = Self;

    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
        
    fn as_owner(self) -> Self::Owner {
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

    fn as_owner(self) -> Self::Owner {
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
        let owner: &'static [u8] = &STATIC_U8;
        let bytes = Bytes::from_owner(owner);
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(owner, bytes_slice)
    }

    #[kani::proof]
    #[kani::unwind(33)]
    pub fn check_box() {
        let owner: Box<[u8]> = STATIC_U8.into();
        let arc = Arc::new(owner);
        let bytes = Bytes::from_arc(arc.clone());
        let arc_slice: &[u8] = &arc;
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(arc_slice, bytes_slice)
    }

    #[cfg(feature = "zerocopy")]
    #[derive(zerocopy::FromZeroes, zerocopy::FromBytes, zerocopy::AsBytes, Clone, Copy)]
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
        let owner: &'static [ComplexZC] = &STATIC_ZC;
        let bytes = Bytes::from_owner(owner);
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(owner.as_bytes(), bytes_slice)
    }

    #[cfg(feature = "zerocopy")]
    #[kani::proof]
    #[kani::unwind(513)]
    pub fn check_box_zeroconf() {
        let owner: Box<[ComplexZC]> = STATIC_ZC.into();
        let arc = Arc::new(owner);
        let bytes = Bytes::from_arc(arc.clone());
        let arc_slice: &[u8] = arc.as_bytes();
        let bytes_slice: &[u8] = &bytes;
        assert_eq!(arc_slice, bytes_slice)
    }
}
