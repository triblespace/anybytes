use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use crate::{ByteOwner, Bytes};
use zerocopy::{AsBytes, FromBytes};

pub struct PackedSlice<T> {
    bytes: Bytes,
    _type: PhantomData<T>,
}

impl<T> PackedSlice<T> {
    pub fn copy_from(value: &[T]) -> Self
    where T: AsBytes {
        PackedSlice {
            bytes: Bytes::copy_from_slice(value.as_bytes()),
            _type: PhantomData,
        }
    }

    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    BadLayout,
}

impl<T> Clone for PackedSlice<T> {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            _type: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for PackedSlice<T>
where
    T: FromBytes + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner: &[T] = self;
        Debug::fmt(inner, f)
    }
}

impl<T> Deref for PackedSlice<T>
where
    T: FromBytes,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        FromBytes::slice_from(&self.bytes).expect("validation should happen at creation")
    }
}

impl<T> AsRef<[T]> for PackedSlice<T>
where
    T: FromBytes,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.deref()
    }
}

impl<O, T> From<O> for PackedSlice<T>
where
    O: ByteOwner + AsRef<[T]>,
{
    fn from(value: O) -> Self {
        PackedSlice {
            bytes: Bytes::from_owner(value),
            _type: PhantomData,
        }
    }
}

impl<O, T> From<Arc<O>> for PackedSlice<T>
where
    O: ByteOwner + AsRef<[T]>,
{
    fn from(value: Arc<O>) -> Self {
        PackedSlice {
            bytes: Bytes::from_arc(value),
            _type: PhantomData,
        }
    }
}

impl<T> TryFrom<Bytes> for PackedSlice<T>
where
    T: FromBytes,
{
    type Error = PackError;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        if <T as FromBytes>::slice_from(&bytes).is_none() {
            Err(PackError::BadLayout)
        } else {
            Ok(PackedSlice {
                bytes,
                _type: PhantomData,
            })
        }
    }
}

impl<T> TryFrom<&Bytes> for PackedSlice<T>
where
    T: FromBytes,
{
    type Error = PackError;

    fn try_from(bytes: &Bytes) -> Result<Self, Self::Error> {
        if <T as FromBytes>::slice_from(bytes).is_none() {
            Err(PackError::BadLayout)
        } else {
            Ok(PackedSlice {
                bytes: bytes.clone(),
                _type: PhantomData,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::PackedSlice;

    #[test]
    fn roundtrip_copy() {
        let v: Vec<usize> = vec![1, 2, 3, 4];
        let p = PackedSlice::copy_from(&v);
        let vr: &[_] = v.as_ref();
        let pr: &[usize] = p.as_ref();
        assert_eq!(vr, pr)
    }

    #[test]
    fn roundtrip() {
        let v: Vec<usize> = vec![1, 2, 3, 4];
        let p: PackedSlice<_> = v.clone().into();
        let r:&[_] = &p;
        assert_eq!(v.as_slice(), r)
    }
}