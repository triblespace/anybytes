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

#[cfg(feature = "zerocopy")]
impl<T> ByteOwner for Vec<T>
where T: AsBytes + Sync + Send + 'static {
    fn as_bytes(&self) -> &[u8] {
        let slice: &[T] = self.as_ref();
        AsBytes::as_bytes(slice)
    }
}

#[cfg(not(feature = "zerocopy"))]
impl ByteOwner for Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

#[cfg(feature = "zerocopy")]
impl<T> ByteOwner for Box<T>
where T: AsBytes + Sync + Send + 'static {
    fn as_bytes(&self) -> &[u8] {
        let inner = self.as_ref();
        AsBytes::as_bytes(inner)
    }
}

#[cfg(not(feature = "zerocopy"))]
impl ByteOwner for Box<[u8]> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl ByteOwner for String {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
#[cfg(feature = "frommmap")]
impl ByteOwner for memmap2::Mmap {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
#[cfg(feature = "frombytes")]
impl ByteOwner for bytes::Bytes {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
