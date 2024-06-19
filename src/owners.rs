/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 * 
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Implement [`BytesOwner`] and [`TextOwner`] for common types.

use crate::ByteOwner;

impl ByteOwner for Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}
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
