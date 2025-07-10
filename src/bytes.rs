/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
*/

//! Core byte container types.
//!
//! [`Bytes`] provides cheap, zero-copy access to immutable bytes from a
//! variety of sources. Implement the [`ByteSource`] trait for your type to
//! integrate it with the container. Each `Bytes` keeps its backing storage
//! alive through a reference-counted [`ByteOwner`], ensuring the data stays
//! valid for as long as needed.
//!
//! `Bytes` decouples data access from ownership so that callers can obtain a
//! slice and then release any external locks. After reading the bytes from a
//! [`ByteSource`], convert it into its [`ByteOwner`] to keep the data alive
//! without retaining the original guard. This is especially useful for Python
//! integration where acquiring the raw pointer to a `bytes` object requires
//! holding the GIL, but once the slice is acquired, only the owner needs to be
//! kept alive.
//!
//! ## Weak references
//!
//! A `Bytes` can be downgraded to a [`WeakBytes`] to hold a non-owning reference
//! without keeping the underlying data alive:
//!
//! ```
//! use anybytes::Bytes;
//!
//! let bytes = Bytes::from(vec![1u8, 2, 3]);
//! let weak = bytes.downgrade();
//! assert!(weak.upgrade().is_some());
//! drop(bytes);
//! assert!(weak.upgrade().is_none());
//! ```
//!
//! ## Downcasting owners
//!
//! When the backing type is known, [`Bytes::downcast_to_owner`] retrieves it
//! again:
//!
//! ```
//! use anybytes::Bytes;
//! use std::sync::Arc;
//!
//! let bytes = Bytes::from_source(vec![1u8, 2, 3, 4]);
//! let owner: Arc<Vec<u8>> = bytes.downcast_to_owner().unwrap();
//! assert_eq!(&*owner, &[1, 2, 3, 4]);
//! ```

use std::any::Any;
use std::ascii::escape_default;
use std::borrow::Borrow;
use std::cmp;
use std::fmt;
use std::hash;
use std::ops::Deref;
use std::slice::SliceIndex;
use std::sync::Arc;
use std::sync::Weak;

use crate::erase_lifetime;

pub(crate) fn is_subslice(slice: &[u8], subslice: &[u8]) -> bool {
    let slice_start = slice.as_ptr() as usize;
    let slice_end = slice_start + slice.len();
    let subslice_start = subslice.as_ptr() as usize;
    let subslice_end = subslice_start + subslice.len();
    subslice_start >= slice_start && subslice_end <= slice_end
}

/// A type that can provide its bytes and yield an owner for them.
///
/// Implementors of this trait serve as sources for [`Bytes`].  The slice
/// returned by [`ByteSource::as_bytes`] must remain valid for as long as the
/// source itself is alive **or** the [`ByteOwner`] obtained from
/// [`ByteSource::get_owner`] is kept alive.  Splitting these capabilities lets
/// callers obtain a byte slice and then drop any locks or guards by converting
/// the source into its owner while still keeping the data valid.
///
/// The returned owner keeps the underlying data alive as long as any [`Bytes`]
/// derived from it are in scope.
pub unsafe trait ByteSource {
    /// The type that owns the bytes.
    type Owner: ByteOwner;

    /// Returns a view of the contained bytes.
    fn as_bytes(&self) -> &[u8];

    /// Consumes the source and returns the owning value.
    fn get_owner(self) -> Self::Owner;
}

/// A trait for types that keep the backing bytes of [`Bytes`] alive.
pub trait ByteOwner: Sync + Send + 'static {
    /// Convert the owner into a type-erased [`Arc`] for downcasting.
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send>;
}

impl<T: ByteSource + Sync + Send + 'static> ByteOwner for T {
    fn as_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send> {
        self
    }
}

/// Immutable bytes with zero-copy slicing and cloning.
///
/// Access itself is extremely cheap via no-op conversion to a `&[u8]`.
///
/// The storage mechanism backing the bytes can be extended
/// and is implemented for a variety of sources already,
/// including other byte handling crates `Bytes`, mmap-ed files,
/// `String`s and `Zerocopy` types.
///
/// See [ByteOwner] for an exhaustive list and more details.
pub struct Bytes {
    data: &'static [u8],
    // Actual owner of the bytes.
    owner: Arc<dyn ByteOwner>,
}

/// Weak variant of [Bytes] that doesn't retain the data
/// unless a strong [Bytes] is referencing it.
///
/// The referenced subrange of the [Bytes] is reconstructed
/// on [WeakBytes::upgrade].
#[derive(Clone, Debug)]
pub struct WeakBytes {
    data: *const [u8],
    owner: Weak<dyn ByteOwner>,
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
    #[inline]
    pub(crate) unsafe fn get_data(&self) -> &'static [u8] {
        self.data
    }

    #[inline]
    pub(crate) unsafe fn set_data(&mut self, data: &'static [u8]) {
        self.data = data;
    }

    #[inline]
    pub(crate) fn get_owner(&self) -> Arc<dyn ByteOwner> {
        self.owner.clone()
    }

    #[inline]
    pub(crate) fn take_owner(self) -> Arc<dyn ByteOwner> {
        self.owner
    }

    /// Creates an empty `Bytes`.
    #[inline]
    pub fn empty() -> Self {
        Self::from_source(&[0u8; 0][..])
    }

    /// Creates `Bytes` from an arbitrary slice and its owner.
    ///
    /// # Safety
    /// The caller must ensure that `data` remains valid for the lifetime of
    /// `owner`. No lifetime checks are performed.
    pub unsafe fn from_raw_parts(data: &'static [u8], owner: Arc<dyn ByteOwner>) -> Self {
        Self { data, owner }
    }

    /// Creates `Bytes` from a [`ByteSource`] (for example, `Vec<u8>`).
    pub fn from_source(source: impl ByteSource) -> Self {
        let data = source.as_bytes();
        // Erase the lifetime.
        let data = unsafe { erase_lifetime(data) };

        let owner = source.get_owner();
        let owner = Arc::new(owner);

        Self { data, owner }
    }

    /// Creates `Bytes` from an `Arc<ByteSource + ByteOwner>`.
    ///
    /// This provides a potentially faster path for `Bytes` creation
    /// as it can forgoe an additional allocation for the wrapping Arc.
    /// For example when you implement `ByteOwner` for a `zerocopy` type,
    /// sadly we can't provide a blanked implementation for those types
    /// because of the orphane rule.
    pub fn from_owning_source_arc(arc: Arc<impl ByteSource + ByteOwner>) -> Self {
        let data = arc.as_bytes();
        // Erase the lifetime.
        let data = unsafe { erase_lifetime(data) };
        Self { data, owner: arc }
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
    /// let bytes = Bytes::from_source(owner);
    /// let owner: Arc<Vec<u8>> = bytes.downcast_to_owner().expect("Downcast of known type.");
    /// ```
    pub fn downcast_to_owner<T>(self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let owner = self.owner;
        let owner = ByteOwner::as_any(owner);
        owner.downcast::<T>().ok()
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

    /// Returns a `Bytes` with the first `len` bytes of `self`.
    /// Modifies `self` to contain the remaining bytes.
    /// Returns `None` if `len` is greater than the length of `self`.
    /// This operation is `O(1)`.
    pub fn take_prefix(&mut self, len: usize) -> Option<Self> {
        if len > self.data.len() {
            return None;
        }
        let (data, rest) = self.data.split_at(len);
        self.data = rest;
        Some(Self {
            data,
            owner: self.owner.clone(),
        })
    }

    /// Returns a `Bytes` with the last `len` bytes of `self`.
    /// Modifies `self` to contain the remaining bytes.
    /// Returns `None` if `len` is greater than the length of `self`.
    /// This operation is `O(1)`.
    pub fn take_suffix(&mut self, len: usize) -> Option<Self> {
        if len > self.data.len() {
            return None;
        }
        let (rest, data) = self.data.split_at(self.data.len() - len);
        self.data = rest;
        Some(Self {
            data,
            owner: self.owner.clone(),
        })
    }

    /// Create a weak pointer.
    pub fn downgrade(&self) -> WeakBytes {
        WeakBytes {
            data: self.data as *const [u8],
            owner: Arc::downgrade(&self.owner),
        }
    }
}

impl WeakBytes {
    /// The reverse of `downgrade`. Returns `None` if the value was dropped.
    pub fn upgrade(&self) -> Option<Bytes> {
        let arc = self.owner.upgrade()?;
        let data = unsafe { &*(self.data) };
        Some(Bytes { data, owner: arc })
    }
}

impl<T: ByteSource> From<T> for Bytes {
    fn from(value: T) -> Self {
        Self::from_source(value)
    }
}

impl<T: ByteSource + ByteOwner> From<Arc<T>> for Bytes {
    fn from(arc: Arc<T>) -> Self {
        Self::from_owning_source_arc(arc)
    }
}

impl Deref for Bytes {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

#[cfg(feature = "ownedbytes")]
unsafe impl ownedbytes::StableDeref for Bytes {}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl hash::Hash for Bytes {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl Default for Bytes {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for Bytes {}

impl PartialOrd for Bytes {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
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

#[cfg(kani)]
mod verification {
    use super::*;
    use kani::BoundedArbitrary;

    #[kani::proof]
    #[kani::unwind(16)]
    pub fn check_take_prefix_ok() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        kani::assume(data.len() >= 5);
        let mut bytes = Bytes::from_source(data.clone());
        let original = bytes.clone();
        let prefix = bytes.take_prefix(5).expect("prefix exists");
        assert_eq!(prefix.as_ref(), &original.as_ref()[..5]);
        assert_eq!(bytes.as_ref(), &original.as_ref()[5..]);
    }

    #[kani::proof]
    #[kani::unwind(32)]
    pub fn check_take_prefix_too_large() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        let mut bytes = Bytes::from_source(data.clone());
        let copy = bytes.clone();
        let res = bytes.take_prefix(32);
        assert!(res.is_none());
        assert_eq!(bytes.as_ref(), copy.as_ref());
    }

    #[kani::proof]
    #[kani::unwind(16)]
    pub fn check_take_suffix_ok() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        kani::assume(data.len() >= 4);
        let mut bytes = Bytes::from_source(data.clone());
        let original = bytes.clone();
        let suffix = bytes.take_suffix(4).expect("suffix exists");
        assert_eq!(suffix.as_ref(), &original.as_ref()[original.len() - 4..]);
        assert_eq!(bytes.as_ref(), &original.as_ref()[..original.len() - 4]);
    }

    #[kani::proof]
    #[kani::unwind(32)]
    pub fn check_take_suffix_too_large() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        let mut bytes = Bytes::from_source(data.clone());
        let copy = bytes.clone();
        let res = bytes.take_suffix(64);
        assert!(res.is_none());
        assert_eq!(bytes.as_ref(), copy.as_ref());
    }

    #[kani::proof]
    #[kani::unwind(16)]
    pub fn check_slice_to_bytes_ok() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        kani::assume(data.len() >= 8);
        let bytes = Bytes::from_source(data.clone());
        let slice = &bytes.as_ref()[3..8];
        let sub = bytes.slice_to_bytes(slice).expect("slice from same bytes");
        assert_eq!(sub.as_ref(), slice);
    }

    #[kani::proof]
    #[kani::unwind(16)]
    pub fn check_slice_to_bytes_unrelated() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        let bytes = Bytes::from_source(data.clone());
        let other: [u8; 4] = kani::any();
        assert!(bytes.slice_to_bytes(&other).is_none());
    }
}
