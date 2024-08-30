use std::{fmt::Debug, marker::PhantomData, ops::Deref, sync::Arc};

use crate::{ByteOwner, Bytes};
use zerocopy::{AsBytes, FromBytes};

pub struct Packed<T> {
    bytes: Bytes,
    _type: PhantomData<T>,
}

impl<T> Packed<T> {
    pub fn copy_from(value: &T) -> Self
    where T: AsBytes {
        Packed {
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

impl<T> Clone for Packed<T> {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            _type: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for Packed<T>
where
    T: FromBytes + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner: &T = self;
        Debug::fmt(inner, f)
    }
}

impl<T> std::ops::Deref for Packed<T>
where
    T: FromBytes,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        FromBytes::ref_from(&self.bytes).expect("validation should happen at creation")
    }
}

impl<T> AsRef<T> for Packed<T>
where
    T: FromBytes,
{
    #[inline]
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<O, T> From<O> for Packed<T>
where
    O: ByteOwner + AsRef<T>,
{
    fn from(value: O) -> Self {
        Packed {
            bytes: Bytes::from_owner(value),
            _type: PhantomData,
        }
    }
}

impl<O, T> From<Arc<O>> for Packed<T>
where
    O: ByteOwner + AsRef<T>,
{
    fn from(value: Arc<O>) -> Self {
        Packed {
            bytes: Bytes::from_arc(value),
            _type: PhantomData,
        }
    }
}

impl<T> TryFrom<Bytes> for Packed<T>
where
    T: FromBytes,
{
    type Error = PackError;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        if <T as FromBytes>::ref_from(&bytes).is_none() {
            Err(PackError::BadLayout)
        } else {
            Ok(Packed {
                bytes,
                _type: PhantomData,
            })
        }
    }
}

impl<T> TryFrom<&Bytes> for Packed<T>
where
    T: FromBytes,
{
    type Error = PackError;

    fn try_from(bytes: &Bytes) -> Result<Self, Self::Error> {
        if <T as FromBytes>::ref_from(bytes).is_none() {
            Err(PackError::BadLayout)
        } else {
            Ok(Packed {
                bytes: bytes.clone(),
                _type: PhantomData,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Packed;

    #[test]
    fn roundtrip_copy() {
        let l: usize = 42;
        let p = Packed::copy_from(&l);
        let r = *p;
        assert_eq!(l, r)
    }

    #[test]
    fn roundtrip() {
        let l: usize = 42;
        let b = Box::new(l);
        let p: Packed<_> = b.into();
        let r = *p;
        assert_eq!(l, r)
    }
}