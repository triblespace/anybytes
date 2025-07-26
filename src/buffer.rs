/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
*/

//! Owned byte buffer with fixed alignment.

use core::alloc::Layout;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

/// A raw byte buffer with a fixed alignment.
///
/// `ByteBuffer` owns its allocation and guarantees that the backing
/// memory is aligned to `ALIGN` bytes.
#[derive(Debug)]
pub struct ByteBuffer<const ALIGN: usize> {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
}

unsafe impl<const ALIGN: usize> Send for ByteBuffer<ALIGN> {}
unsafe impl<const ALIGN: usize> Sync for ByteBuffer<ALIGN> {}

impl<const ALIGN: usize> ByteBuffer<ALIGN> {
    const _ASSERT_POWER_OF_TWO: () = assert!(ALIGN.is_power_of_two(), "ALIGN must be power-of-two");
    /// Create an empty buffer.
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    /// Create a buffer with the given capacity.
    pub fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            return Self::new();
        }
        unsafe {
            let layout = Layout::from_size_align_unchecked(cap, ALIGN);
            let ptr = std::alloc::alloc(layout);
            let ptr = NonNull::new(ptr).expect("alloc failed");
            Self { ptr, len: 0, cap }
        }
    }

    /// Current length of the buffer.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Ensure that the buffer can hold at least `total` bytes.
    ///
    /// Does nothing if the current capacity is already sufficient.
    pub fn reserve_total(&mut self, total: usize) {
        if total <= self.cap {
            return;
        }
        unsafe {
            let old = Layout::from_size_align_unchecked(self.cap.max(1), ALIGN);
            let new_layout = Layout::from_size_align_unchecked(total, ALIGN);
            let new_ptr = if self.cap == 0 {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(self.ptr.as_ptr(), old, total)
            };
            let new_ptr = NonNull::new(new_ptr).expect("realloc failed");
            self.ptr = new_ptr;
        }
        self.cap = total;
    }

    #[inline]
    fn reserve_more(&mut self, additional: usize) {
        let needed = self.len.checked_add(additional).expect("overflow");
        if needed <= self.cap {
            return;
        }
        let new_cap = core::cmp::max(self.cap * 2, needed);
        unsafe {
            let old = Layout::from_size_align_unchecked(self.cap.max(1), ALIGN);
            let new_layout = Layout::from_size_align_unchecked(new_cap, ALIGN);
            let new_ptr = if self.cap == 0 {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(self.ptr.as_ptr(), old, new_cap)
            };
            let new_ptr = NonNull::new(new_ptr).expect("realloc failed");
            self.ptr = new_ptr;
        }
        self.cap = new_cap;
    }

    /// Push a byte to the end of the buffer.
    pub fn push(&mut self, byte: u8) {
        self.reserve_more(1);
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), byte);
        }
        self.len += 1;
    }

    /// Returns a raw pointer to the buffer's memory.
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }
}

impl<const ALIGN: usize> Drop for ByteBuffer<ALIGN> {
    fn drop(&mut self) {
        if self.cap != 0 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.cap, ALIGN);
                std::alloc::dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}

impl<const ALIGN: usize> Deref for ByteBuffer<ALIGN> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<const ALIGN: usize> DerefMut for ByteBuffer<ALIGN> {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<const ALIGN: usize> AsRef<[u8]> for ByteBuffer<ALIGN> {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl<const ALIGN: usize> AsMut<[u8]> for ByteBuffer<ALIGN> {
    fn as_mut(&mut self) -> &mut [u8] {
        self
    }
}
