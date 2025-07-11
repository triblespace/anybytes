/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::borrow::Borrow;
use std::cmp::Ordering;

use crate::bytes::is_subslice;
use crate::erase_lifetime;
use crate::{bytes::ByteOwner, Bytes};
use std::sync::Weak;
use std::{fmt::Debug, hash::Hash, ops::Deref, sync::Arc};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryCastError, TryFromBytes};

/// Errors that can occur when constructing a [`View`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewError {
    /// The provided bytes were not properly aligned for the target type.
    Alignment(Bytes),
    /// The provided bytes were of incorrect size for the target type.
    Size(Bytes),
    /// The bytes contained invalid data for the target type.
    Validity(Bytes),
}

impl std::fmt::Display for ViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewError::Alignment(_) => write!(
                f,
                "failed to create view: The conversion source was improperly aligned."
            ),
            ViewError::Size(_) => write!(
                f,
                "failed to create view: The conversion source was of incorrect size."
            ),
            ViewError::Validity(_) => write!(
                f,
                "failed to create view: The conversion source contained invalid data."
            ),
        }
    }
}

impl std::error::Error for ViewError {}

impl ViewError {
    pub(crate) fn from_cast_error<T: ?Sized + TryFromBytes>(
        bytes: &Bytes,
        err: TryCastError<&[u8], T>,
    ) -> Self {
        match err {
            TryCastError::Alignment(err) => {
                Self::Alignment(bytes.slice_to_bytes(err.into_src()).unwrap())
            }
            TryCastError::Size(err) => Self::Size(bytes.slice_to_bytes(err.into_src()).unwrap()),
            TryCastError::Validity(err) => {
                Self::Validity(bytes.slice_to_bytes(err.into_src()).unwrap())
            }
        }
    }
}

impl Bytes {
    /// Interpret the bytes as a view of `T`.
    pub fn view<T>(self) -> Result<View<T>, ViewError>
    where
        T: ?Sized + TryFromBytes + KnownLayout + Immutable,
    {
        unsafe {
            match <T as TryFromBytes>::try_ref_from_bytes(self.get_data()) {
                Ok(data) => Ok(View {
                    data,
                    owner: self.take_owner(),
                }),
                Err(err) => Err(ViewError::from_cast_error(&self, err)),
            }
        }
    }

    /// Split off the beginning of this `Bytes` as a view of `T`.
    pub fn view_prefix<T>(&mut self) -> Result<View<T>, ViewError>
    where
        T: ?Sized + TryFromBytes + KnownLayout + Immutable,
    {
        unsafe {
            match <T as TryFromBytes>::try_ref_from_prefix(self.get_data()) {
                Ok((data, rest)) => {
                    self.set_data(rest);
                    Ok(View {
                        data,
                        owner: self.get_owner(),
                    })
                }
                Err(err) => Err(ViewError::from_cast_error(self, err)),
            }
        }
    }

    /// Split off the beginning of this `Bytes` as a slice-like view containing
    /// `count` elements of `T`.
    pub fn view_prefix_with_elems<T>(&mut self, count: usize) -> Result<View<T>, ViewError>
    where
        T: ?Sized + TryFromBytes + KnownLayout<PointerMetadata = usize> + Immutable,
    {
        unsafe {
            match <T as TryFromBytes>::try_ref_from_prefix_with_elems(self.get_data(), count) {
                Ok((data, rest)) => {
                    self.set_data(rest);
                    Ok(View {
                        data,
                        owner: self.get_owner(),
                    })
                }
                Err(err) => Err(ViewError::from_cast_error(self, err)),
            }
        }
    }

    /// Split off the end of this `Bytes` as a view of `T`.
    pub fn view_suffix<T>(&mut self) -> Result<View<T>, ViewError>
    where
        T: ?Sized + TryFromBytes + KnownLayout + Immutable,
    {
        unsafe {
            match <T as TryFromBytes>::try_ref_from_suffix(self.get_data()) {
                Ok((rest, data)) => {
                    self.set_data(rest);
                    Ok(View {
                        data,
                        owner: self.get_owner(),
                    })
                }
                Err(err) => Err(ViewError::from_cast_error(self, err)),
            }
        }
    }

    /// Split off the end of this `Bytes` as a slice-like view containing
    /// `count` elements of `T`.
    pub fn view_suffix_with_elems<T>(&mut self, count: usize) -> Result<View<T>, ViewError>
    where
        T: ?Sized + TryFromBytes + KnownLayout<PointerMetadata = usize> + Immutable,
    {
        unsafe {
            match <T as TryFromBytes>::try_ref_from_suffix_with_elems(self.get_data(), count) {
                Ok((rest, data)) => {
                    self.set_data(rest);
                    Ok(View {
                        data,
                        owner: self.get_owner(),
                    })
                }
                Err(err) => Err(ViewError::from_cast_error(self, err)),
            }
        }
    }
}

/// Immutable view with zero-copy field derive and cloning.
///
/// Access itself is the same as accessing a `&T`.
///
/// Has a backing `ByteOwner` that retains the bytes until all views are dropped,
/// analogue to `Bytes`.
///
/// See [ByteOwner] for an exhaustive list and more details.
pub struct View<T: Immutable + ?Sized + 'static> {
    pub(crate) data: &'static T,
    // Actual owner of the bytes.
    pub(crate) owner: Arc<dyn ByteOwner>,
}

/// Weak variant of [View] that doesn't retain the data
/// unless a strong [View] is referencing it.
///
/// The referenced subrange of the [View] is reconstructed
/// on [`WeakView::upgrade`].
pub struct WeakView<T: Immutable + ?Sized + 'static> {
    pub(crate) data: *const T,
    pub(crate) owner: Weak<dyn ByteOwner>,
}

impl<T: ?Sized + Immutable> Clone for WeakView<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            owner: self.owner.clone(),
        }
    }
}

impl<T: ?Sized + Immutable> Debug for WeakView<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakView")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

// ByteOwner is Send + Sync and View is immutable.
unsafe impl<T: ?Sized + Immutable> Send for View<T> {}
unsafe impl<T: ?Sized + Immutable> Sync for View<T> {}

impl<T: ?Sized + Immutable> Clone for View<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            owner: self.owner.clone(),
        }
    }
}

// Core implementation of View.
impl<T: ?Sized + Immutable> View<T> {
    /// Creates a view from raw parts without any checks.
    ///
    /// # Safety
    /// The caller must guarantee that `data` remains valid for the lifetime of
    /// `owner`.
    pub unsafe fn from_raw_parts(data: &'static T, owner: Arc<dyn ByteOwner>) -> Self {
        Self { data, owner }
    }

    /// Returns the owner of the View in an `Arc`.
    pub fn downcast_to_owner<O>(self) -> Option<Arc<O>>
    where
        O: Send + Sync + 'static,
    {
        let owner = self.owner;
        let owner = ByteOwner::as_any(owner);
        owner.downcast::<O>().ok()
    }

    /// Create a weak pointer.
    pub fn downgrade(&self) -> WeakView<T> {
        WeakView {
            data: self.data as *const T,
            owner: Arc::downgrade(&self.owner),
        }
    }
}

impl<T: ?Sized + Immutable + IntoBytes> View<T> {
    /// Converts this view back into [`Bytes`].
    pub fn bytes(self) -> Bytes {
        let bytes = IntoBytes::as_bytes(self.data);
        unsafe { Bytes::from_raw_parts(bytes, self.owner) }
    }

    /// Attempt to convert `reference` to a zero-copy subview of this `View`.
    ///
    /// Returns `None` if the bytes of the child are outside the memory range of
    /// the bytes of this view.
    ///
    /// This is similar to `Bytes::slice_to_bytes` but for `View`.
    pub fn field_to_view<F: ?Sized + Immutable + IntoBytes>(&self, field: &F) -> Option<View<F>> {
        let self_bytes = IntoBytes::as_bytes(self.data);
        let field_bytes = IntoBytes::as_bytes(field);
        if is_subslice(self_bytes, field_bytes) {
            let data = unsafe { erase_lifetime(field) };
            let owner = self.owner.clone();
            Some(View::<F> { data, owner })
        } else {
            None
        }
    }
}

impl<T: ?Sized + Immutable> WeakView<T> {
    /// The reverse of `downgrade`. Returns `None` if the value was dropped.
    pub fn upgrade(&self) -> Option<View<T>> {
        let arc = self.owner.upgrade()?;
        let data = unsafe { &*(self.data) };
        Some(View { data, owner: arc })
    }
}

impl<T> Deref for View<T>
where
    T: ?Sized + Immutable,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<T> AsRef<T> for View<T>
where
    T: ?Sized + Immutable,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> Borrow<T> for View<T>
where
    T: ?Sized + Immutable,
{
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized + PartialEq + Immutable> PartialEq for View<T> {
    fn eq(&self, other: &Self) -> bool {
        let this: &T = self;
        let other: &T = other;
        this == other
    }
}

impl<T: ?Sized + Eq + Immutable> Eq for View<T> {}

impl<T: ?Sized + PartialOrd + Immutable> PartialOrd for View<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let this: &T = self;
        let other: &T = other;
        this.partial_cmp(other)
    }
}

impl<T: ?Sized + Ord + Immutable> Ord for View<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let this: &T = self;
        let other: &T = other;
        this.cmp(other)
    }
}

impl<T> Debug for View<T>
where
    T: ?Sized + Immutable + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &T = self;
        Debug::fmt(value, f)
    }
}

impl<T> Hash for View<T>
where
    T: ?Sized + Immutable + Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let value = self.deref();
        value.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::ViewError;
    use crate::Bytes;
    use crate::View;

    #[test]
    fn roundtrip() {
        let value: usize = 42;
        let boxed = Box::new(value);
        let bytes = Bytes::from_source(boxed);
        let view = bytes.view::<usize>().unwrap();
        let view_value = *view;
        assert_eq!(value, view_value);
    }
    #[test]
    fn niche_optimisation_option() {
        assert_eq!(size_of::<View<usize>>(), size_of::<Option<View<usize>>>());
    }

    #[test]
    fn slice_roundtrip() {
        let value: Vec<usize> = vec![1, 2, 3, 4];
        let bytes = Bytes::from_source(value.clone());
        let view = bytes.view::<[usize]>().unwrap();
        let view_value = view.as_ref();
        assert_eq!(&value, view_value);
    }

    #[test]
    fn str_roundtrip() {
        let value: String = "hello world!".to_string();
        let bytes = Bytes::from_source(value.clone());
        let view = bytes.view::<str>().unwrap();
        let view_value = view.as_ref();
        assert_eq!(&value, view_value);
    }

    #[test]
    fn view_prefix_split() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3, 4]);
        let view = bytes.view_prefix::<[u8; 2]>().unwrap();
        assert_eq!(*view, [1u8, 2]);
        assert_eq!(&bytes[..], [3u8, 4].as_slice());
    }

    #[test]
    fn view_prefix_with_elems_split() {
        let mut bytes = Bytes::from_source(vec![10u8, 11, 12, 13]);
        let view = bytes.view_prefix_with_elems::<[u8]>(2).unwrap();
        assert_eq!(view.as_ref(), [10u8, 11].as_slice());
        assert_eq!(&bytes[..], [12u8, 13].as_slice());
    }

    #[test]
    fn view_suffix_split() {
        let mut bytes = Bytes::from_source(vec![5u8, 6, 7, 8]);
        let view = bytes.view_suffix::<[u8; 2]>().unwrap();
        assert_eq!(*view, [7u8, 8]);
        assert_eq!(&bytes[..], [5u8, 6].as_slice());
    }

    #[test]
    fn view_suffix_with_elems_split() {
        let mut bytes = Bytes::from_source(vec![20u8, 21, 22, 23]);
        let view = bytes.view_suffix_with_elems::<[u8]>(2).unwrap();
        assert_eq!(view.as_ref(), [22u8, 23].as_slice());
        assert_eq!(&bytes[..], [20u8, 21].as_slice());
    }

    #[test]
    fn view_prefix_size_error() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3]);
        let res = bytes.view_prefix::<[u8; 4]>();
        assert!(matches!(res, Err(ViewError::Size(_))));
        assert_eq!(&bytes[..], [1u8, 2, 3].as_slice());
    }

    #[test]
    fn view_prefix_with_elems_size_error() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3]);
        let res = bytes.view_prefix_with_elems::<[u8]>(4);
        assert!(matches!(res, Err(ViewError::Size(_))));
        assert_eq!(&bytes[..], [1u8, 2, 3].as_slice());
    }

    #[test]
    fn view_suffix_size_error() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3]);
        let res = bytes.view_suffix::<[u8; 4]>();
        assert!(matches!(res, Err(ViewError::Size(_))));
        assert_eq!(&bytes[..], [1u8, 2, 3].as_slice());
    }

    #[test]
    fn view_suffix_with_elems_size_error() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3]);
        let res = bytes.view_suffix_with_elems::<[u8]>(4);
        assert!(matches!(res, Err(ViewError::Size(_))));
        assert_eq!(&bytes[..], [1u8, 2, 3].as_slice());
    }

    #[test]
    fn downgrade_upgrade() {
        let bytes = Bytes::from_source(b"abcd".to_vec());
        let view = bytes.clone().view::<[u8]>().unwrap();

        // `downgrade` -> `upgrade` returns the same view.
        let weak = view.downgrade();
        let upgraded = weak.upgrade().expect("upgrade succeeds");
        assert_eq!(upgraded.as_ref(), view.as_ref());

        // `upgrade` returns `None` if all strong refs are dropped.
        drop(bytes);
        drop(view);
        drop(upgraded);
        assert!(weak.upgrade().is_none());
    }
}

#[cfg(kani)]
mod verification {
    use super::*;
    use kani::BoundedArbitrary;

    #[kani::proof]
    #[kani::unwind(16)]
    pub fn check_view_prefix_ok() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        kani::assume(data.len() >= 4);
        let mut bytes = Bytes::from_source(data.clone());
        let original = bytes.clone();
        let view = bytes.view_prefix::<[u8; 4]>().expect("prefix exists");
        let expected: [u8; 4] = original.as_ref()[..4].try_into().unwrap();
        assert_eq!(*view, expected);
        assert_eq!(bytes.as_ref(), &original.as_ref()[4..]);
    }

    #[kani::proof]
    #[kani::unwind(16)]
    pub fn check_view_suffix_ok() {
        let data: Vec<u8> = Vec::bounded_any::<16>();
        kani::assume(data.len() >= 4);
        let mut bytes = Bytes::from_source(data.clone());
        let original = bytes.clone();
        let view = bytes.view_suffix::<[u8; 4]>().expect("suffix exists");
        let start = original.len() - 4;
        let expected: [u8; 4] = original.as_ref()[start..].try_into().unwrap();
        assert_eq!(*view, expected);
        assert_eq!(bytes.as_ref(), &original.as_ref()[..start]);
    }

    #[derive(
        zerocopy::TryFromBytes,
        zerocopy::IntoBytes,
        zerocopy::KnownLayout,
        zerocopy::Immutable,
        Clone,
        Copy,
    )]
    #[repr(C)]
    struct Pair {
        a: u32,
        b: u32,
    }

    #[kani::proof]
    #[kani::unwind(8)]
    pub fn check_field_to_view_ok() {
        let value = Pair {
            a: kani::any(),
            b: kani::any(),
        };
        let bytes = Bytes::from_source(Box::new(value));
        let view = bytes.view::<Pair>().unwrap();
        let field = view.field_to_view(&view.a).expect("field view");
        assert_eq!(*field, view.a);

        let other: u32 = kani::any();
        assert!(view.field_to_view(&other).is_none());
    }
}
