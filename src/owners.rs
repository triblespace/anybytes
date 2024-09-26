/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Implement [`BytesOwner`] and [`TextOwner`] for common types.

#[cfg(feature = "zerocopy")]
use zerocopy::AsBytes;

use crate::ByteOwner;

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteOwner for &'static [T]
where
    T: AsBytes + Sync + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        AsBytes::as_bytes(*self)
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteOwner for &'static [u8] {
    fn as_bytes(&self) -> &[u8] {
        *self
    }
}

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteOwner for Box<T>
where
    T: AsBytes + ?Sized + Sync + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        let inner = self.as_ref();
        AsBytes::as_bytes(inner)
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteOwner for Box<[u8]> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "zerocopy")]
unsafe impl<T> ByteOwner for Vec<T>
where
    T: AsBytes + Sync + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        let slice: &[T] = self.as_ref();
        AsBytes::as_bytes(slice)
    }
}

#[cfg(not(feature = "zerocopy"))]
unsafe impl ByteOwner for Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

unsafe impl ByteOwner for String {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "bytes")]
unsafe impl ByteOwner for bytes::Bytes {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "ownedbytes")]
unsafe impl ByteOwner for ownedbytes::OwnedBytes {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "mmap")]
unsafe impl ByteOwner for memmap2::Mmap {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
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
        c: u32
    }

    #[cfg(feature = "zerocopy")]
    static STATIC_ZC: [ComplexZC; 32] = [ComplexZC {
        a: 42,
        b: [0; 4],
        c: 9000
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