/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::any::Any;
use std::ascii::escape_default;
use std::borrow;
use std::cmp;
use std::fmt;
use std::hash;
use std::ops;
use std::ops::Range;
use std::slice::SliceIndex;
use std::sync::Arc;
use std::sync::Weak;

pub unsafe trait ByteOwner: Send + Sync + 'static {
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
    pub(crate) data: &'static [u8],
    // Actual owner of the bytes. None for static buffers.
    pub(crate) owner: Option<Arc<dyn AnyByteOwner>>,
}

// ByteOwner is Send + Sync and Bytes is immutable.
unsafe impl Send for Bytes {}
unsafe impl Sync for Bytes {}

impl Clone for Bytes {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            owner: self.owner.clone(),
        }
    }
}

// Core implementation of Bytes.
impl Bytes {
    /// Creates `Bytes` from a static slice.
    pub const fn from_static(slice: &'static [u8]) -> Self {
        Self {
            // This is safe because slices always have a non-null address.
            data: slice,
            owner: None,
        }
    }

    /// Returns a slice of self for the provided range.
    /// This operation is `O(1)`.
    pub fn slice(&self, range: impl SliceIndex<[u8], Output = [u8]>) -> Self {
        Self {
            data: &self.data[range],
            owner: self.owner.clone(),
        }
    }

    /// Attempt to convert `slice` to a zero-copy slice of this `Bytes`.
    ///
    /// Returns `None` if `slice` is outside the memory range of this
    /// `Bytes`.
    ///
    /// This is similar to `bytes::Bytes::slice_ref` from `bytes 0.5.4`,
    /// but does not panic.
    pub fn slice_to_bytes(&self, slice: &[u8]) -> Option<Self> {
        let range = self.range_of_slice(slice)?;
        Some(self.slice(range))
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
        let bytes_start = self.data.as_ptr() as usize;
        let bytes_end = bytes_start + self.len();
        if slice_start >= bytes_start && slice_end <= bytes_end {
            let start = slice_start - bytes_start;
            Some(start..start + slice.len())
        } else {
            None
        }
    }

    /// Creates an empty `Bytes`.
    #[inline]
    pub fn empty() -> Self {
        Self {
            data: &[],
            owner: None,
        }
    }

    /// Creates `Bytes` from a [`BytesOwner`] (for example, `Vec<u8>`).
    pub fn from_owner(owner: impl ByteOwner) -> Self {
        let arc = Arc::new(owner);
        Self::from_arc(arc)
    }

    /// Creates `Bytes` from an `Arc<BytesOwner>`.
    ///
    /// This provides a potentially faster path for `Bytes` creation
    /// as it can forgoe an additional allocation for the wrapping Arc.
    /// For example when you implement `ByteOwner` for a `zerocopy` type,
    /// sadly we can't provide a blanked implementation for those types
    /// because of the orphane rule.
    pub fn from_arc(arc: Arc<impl ByteOwner>) -> Self {
        let data = arc.as_bytes();
        // Erase the lifetime.
        let data = unsafe { &*(data as *const [u8]) };
        Self {
            // This is safe because slices always have a non-null address.
            data,
            owner: Some(arc),
        }
    }

    #[inline]
    pub(crate) fn as_slice<'a>(&'a self) -> &'a [u8] {
        self.data
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
    pub fn downgrade(&self) -> Option<WeakBytes> {
        self.owner.as_ref().map(Arc::downgrade)
    }

    /// The reverse of `downgrade`. Returns `None` if the value was dropped.
    /// Note the upgraded `Bytes` has the full range of the buffer.
    pub fn upgrade(weak: &WeakBytes) -> Option<Self> {
        let arc = weak.upgrade()?;
        let data: &[u8] = arc.as_ref().as_bytes();
        // Erase the lifetime.
        let data = unsafe { &*(data as *const [u8]) };
        Some(Self {
            // This is safe because slices always have a non-null address.
            data,
            owner: Some(arc),
        })
    }
}

impl<T: ByteOwner> From<T> for Bytes {
    fn from(value: T) -> Self {
        Self::from_owner(value)
    }
}

impl<T: ByteOwner> From<Arc<T>> for Bytes {
    fn from(arc: Arc<T>) -> Self {
        Self::from_arc(arc)
    }
}

impl From<&'static [u8]> for Bytes {
    fn from(value: &'static [u8]) -> Self {
        Self::from_static(value)
    }
}

impl From<&'static str> for Bytes {
    fn from(value: &'static str) -> Self {
        Self::from_static(value.as_bytes())
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl ops::Deref for Bytes {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

#[cfg(feature = "fromownedbytes")]
unsafe impl ownedbytes::StableDeref for Bytes {}

impl hash::Hash for Bytes {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl borrow::Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Default for Bytes {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for Bytes {
    fn eq(&self, other: &T) -> bool {
        self.as_slice() == other.as_ref()
    }
}

impl Eq for Bytes {}

impl<T: AsRef<[u8]>> PartialOrd<T> for Bytes {
    fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}

impl Ord for Bytes {
    fn cmp(&self, other: &Bytes) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `[u8]::escape_ascii` when inherent_ascii_escape is stabilized.
        f.write_str("b\"")?;
        for &byte in self.as_slice() {
            fmt::Display::fmt(&escape_default(byte), f)?;
        }
        f.write_str("\"")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;

    #[test]
    fn niche_optimisation() {
        assert_eq!(size_of::<Bytes>(), size_of::<Option<Bytes>>());
    }
}