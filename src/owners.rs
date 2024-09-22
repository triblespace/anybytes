/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Implement [`BytesOwner`] and [`TextOwner`] for common types.

use zerocopy::AsBytes;

use crate::ByteOwner;

#[cfg(feature = "fromzerocopy")]
unsafe impl<T> ByteOwner for Vec<T>
where
    T: AsBytes + Sync + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        let slice: &[T] = self.as_ref();
        AsBytes::as_bytes(slice)
    }
}

#[cfg(not(feature = "fromzerocopy"))]
unsafe impl ByteOwner for Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "fromzerocopy")]
unsafe impl<T> ByteOwner for Box<T>
where
    T: AsBytes + ?Sized + Sync + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        let inner = self.as_ref();
        AsBytes::as_bytes(inner)
    }
}

#[cfg(not(feature = "fromzerocopy"))]
unsafe impl ByteOwner for Box<[u8]> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "fromzerocopy")]
unsafe impl<T> ByteOwner for &'static [T]
where
    T: AsBytes + Sync + Send + 'static,
{
    fn as_bytes(&self) -> &[u8] {
        AsBytes::as_bytes(*self)
    }
}

#[cfg(not(feature = "fromzerocopy"))]
unsafe impl ByteOwner for &'static [u8] {
    fn as_bytes(&self) -> &[u8] {
        *self
    }
}

unsafe impl ByteOwner for String {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "frombytes")]
unsafe impl ByteOwner for bytes::Bytes {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "fromownedbytes")]
unsafe impl ByteOwner for ownedbytes::OwnedBytes {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "frommmap")]
unsafe impl ByteOwner for memmap2::Mmap {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
