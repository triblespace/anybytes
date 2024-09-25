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

fn range_of_subslice(slice: &[u8], subslice: &[u8]) -> Option<Range<usize>> {
    let slice_start = slice.as_ptr() as usize;
    let slice_end = slice_start + slice.len();
    let subslice_start = subslice.as_ptr() as usize;
    let subslice_end = subslice_start + subslice.len();
    if subslice_start >= slice_start && subslice_end <= slice_end {
        let start = subslice_start - slice_start;
        Some(start..start + subslice.len())
    } else {
        None
    }
}

fn is_subslice(slice: &[u8], subslice: &[u8]) -> bool {
    let slice_start = slice.as_ptr() as usize;
    let slice_end = slice_start + slice.len();
    let subslice_start = subslice.as_ptr() as usize;
    let subslice_end = subslice_start + subslice.len();
    subslice_start >= slice_start && subslice_end <= slice_end
}

unsafe fn erase_lifetime<'a>(slice: &'a [u8]) -> &'static [u8] {
    &*(slice as *const [u8])
}

pub unsafe trait ByteOwner: Sync + Send + 'static {
    fn as_bytes(&self) -> &[u8];
}
pub trait AnyByteOwner: ByteOwner {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send>;
}

impl<T: ByteOwner> AnyByteOwner for T {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send> {
        self
    }
}

/// Immutable bytes with zero-copy slicing and cloning.
pub struct Bytes {
    pub(crate) data: &'static [u8],
    // Actual owner of the bytes.
    pub(crate) owner: Arc<dyn AnyByteOwner>,
}

/// Weak variant of Bytes that doesn't retain the data
/// unless a strong Bytes is referencing it.
pub struct WeakBytes {
    pub(crate) range: Range<usize>,
    // Actual owner of the bytes.
    pub(crate) owner: Weak<dyn AnyByteOwner>,
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
    /// Creates an empty `Bytes`.
    #[inline]
    pub fn empty() -> Self {
        Self::from_owner(&[0u8; 0][..])
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
        let data = unsafe { erase_lifetime(data) };
        Self {
            // This is safe because slices always have a non-null address.
            data,
            owner: arc,
        }
    }

    #[inline]
    pub(crate) fn as_slice<'a>(&'a self) -> &'a [u8] {
        self.data
    }

    /// Returns the owner of the Bytes as a `Arc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use anybytes::Bytes;
    /// use std::sync::Arc;
    /// let owner: Vec<u8> = vec![0,1,2,3];
    /// let bytes = Bytes::from_owner(owner);
    /// let owner: Arc<Vec<u8>> = bytes.downcast().expect("Downcast of known type.");
    /// ```
    pub fn downcast<T>(self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let owner = self.owner;
        let owner = AnyByteOwner::as_any(owner);
        owner.downcast::<T>().ok()
    }

    pub fn reset(&mut self) {
        let data = self.owner.as_bytes();
        self.data = unsafe { erase_lifetime(data) };
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
        if is_subslice(self.data, slice) {
            let data = unsafe { erase_lifetime(slice) };
            let owner = self.owner.clone();
            Some(Self { data, owner })
        } else {
            None
        }
    }

    /// Create a weak pointer. Returns `None` if backed by a static buffer.
    /// Note the weak pointer has the full range of the buffer.
    pub fn downgrade(&self) -> WeakBytes {
        let owned_slice = self.owner.as_bytes();
        WeakBytes {
            range: range_of_subslice(owned_slice, self.data)
                .expect("self.data is always a valid range"),
            owner: Arc::downgrade(&self.owner),
        }
    }
}

impl WeakBytes {
    /// The reverse of `downgrade`. Returns `None` if the value was dropped.
    pub fn upgrade(&self) -> Option<Bytes> {
        let arc = self.owner.upgrade()?;
        let data: &[u8] = arc.as_ref().as_bytes();
        let data = &data[self.range.clone()];
        let data = unsafe { erase_lifetime(data) };
        Some(Bytes { data, owner: arc })
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

impl From<&'static str> for Bytes {
    fn from(value: &'static str) -> Self {
        Self::from_owner(value.as_bytes())
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

#[cfg(feature = "ownedbytes")]
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
