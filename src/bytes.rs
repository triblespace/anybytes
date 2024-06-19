/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::any::Any;
use std::ops::Range;
use std::ops::RangeBounds;
use std::sync::Arc;
use std::sync::Weak;

pub trait ByteOwner: Send + Sync + 'static {
    fn as_bytes(&self) -> &[u8];
}
pub trait AnyByteOwner: ByteOwner {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: ByteOwner> AnyByteOwner for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub type WeakBytes = Weak<dyn AnyByteOwner>;

/// Immutable bytes with zero-copy slicing and cloning.
pub struct Bytes {
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,

    // Actual owner of the bytes. None for static buffers.
    pub(crate) owner: Option<Arc<dyn AnyByteOwner>>,
}

// ByteOwner is Send + Sync and Bytes is immutable.
unsafe impl Send for Bytes {}
unsafe impl Sync for Bytes {}

// #[derive(Clone)] does not work well with type parameters.
// Therefore implement Clone manually.
impl Clone for Bytes {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            len: self.len,
            owner: self.owner.clone(),
        }
    }
}

// Core implementation of Bytes.
impl Bytes {
    /// Creates `Bytes` from a static slice.
    pub const fn from_static(slice: &'static [u8]) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
            owner: None,
        }
    }

    /// Returns a slice of self for the provided range.
    /// This operation is `O(1)`.
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        use std::ops::Bound;
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.len,
        };
        assert!(start <= end, "invalid slice {}..{}", start, end);
        assert!(end <= self.len, "{} exceeds Bytes length {}", end, self.len);
        if start == end {
            Self::new()
        } else {
            Self {
                ptr: unsafe { self.ptr.add(start) },
                len: end - start,
                owner: self.owner.clone(),
            }
        }
    }

    /// Attempt to convert `slice` to a zero-copy slice of this `Bytes`.
    /// Copy the `slice` if zero-copy cannot be done.
    ///
    /// This is similar to `bytes::Bytes::slice_ref` from `bytes 0.5.4`,
    /// but does not panic.
    pub fn slice_to_bytes(&self, slice: &[u8]) -> Self {
        match self.range_of_slice(slice) {
            Some(range) => self.slice(range),
            None => Self::copy_from_slice(slice),
        }
    }

    /// Return a range `x` so that `self[x]` matches `slice` exactly
    /// (not only content, but also internal pointer addresses).
    ///
    /// Returns `None` if `slice` is outside the memory range of this
    /// `Bytes`.
    ///
    /// This operation is `O(1)`.
    pub fn range_of_slice(&self, slice: &[u8]) -> Option<Range<usize>> {
        let slice_start = slice.as_ptr() as usize;
        let slice_end = slice_start + slice.len();
        let bytes_start = self.ptr as usize;
        let bytes_end = bytes_start + self.len;
        if slice_start >= bytes_start && slice_end <= bytes_end {
            let start = slice_start - bytes_start;
            Some(start..start + slice.len())
        } else {
            None
        }
    }

    /// Creates an empty `Bytes`.
    #[inline]
    pub fn new() -> Self {
        let empty = &[];
        Self {
            ptr: empty.as_ptr(),
            len: empty.len(),
            owner: None,
        }
    }

    /// Creates `Bytes` from a [`BytesOwner`] (for example, `Vec<u8>`).
    pub fn from_owner(owner: impl ByteOwner) -> Self {
        let bytes = owner.as_bytes();
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            owner: Some(Arc::new(owner)),
        }
    }

    /// Creates `Bytes` instance from slice, by copying it.
    pub fn copy_from_slice(data: &[u8]) -> Self {
        Self::from_owner(data.to_vec())
    }

    #[inline]
    pub(crate) fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Attempt to downcast to an exclusive mut reference.
    ///
    /// Returns None if the type mismatches, or the internal reference count is
    /// not 0.
    pub fn downcast_mut<A: Any>(&mut self) -> Option<&mut A> {
        let arc_owner = self.owner.as_mut()?;
        let owner = Arc::get_mut(arc_owner)?;
        let any = owner.as_any_mut();
        any.downcast_mut()
    }

    /// Create a weak pointer. Returns `None` if backed by a static buffer.
    /// Note the weak pointer has the full range of the buffer.
    pub fn downgrade(&self) -> Option<Weak<dyn AnyByteOwner>> {
        self.owner.as_ref().map(Arc::downgrade)
    }

    /// The reverse of `downgrade`. Returns `None` if the value was dropped.
    /// Note the upgraded `Bytes` has the full range of the buffer.
    pub fn upgrade(weak: &Weak<dyn AnyByteOwner>) -> Option<Self> {
        let arc = weak.upgrade()?;
        let slice_like: &[u8] = arc.as_ref().as_bytes();
        Some(Self {
            ptr: slice_like.as_ptr(),
            len: slice_like.len(),
            owner: Some(arc),
        })
    }
}
