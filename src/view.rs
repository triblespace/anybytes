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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewError {
    Alignment(Bytes),
    Size(Bytes),
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
/// on [WeakBytes::upgrade].
pub struct WeakView<T: Immutable + ?Sized + 'static> {
    pub(crate) data: *const T,
    pub(crate) owner: Weak<dyn ByteOwner>,
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
}
