use std::{fmt::Debug, hash::Hash, ops::Deref, str::Utf8Error, sync::Arc};

use crate::{ByteOwner, Bytes};

pub struct PackedStr {
    bytes: Bytes,
}

impl PackedStr {
    pub fn copy_from(value: &str) -> Self {
        let bx: Box<[u8]> = value.as_bytes().into();
        PackedStr {
            bytes: Bytes::from_owner(bx),
        }
    }

    pub fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

impl std::ops::Deref for PackedStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }
}

impl AsRef<str> for PackedStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl Clone for PackedStr {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
        }
    }
}

impl std::fmt::Debug for PackedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner: &str = self;
        Debug::fmt(inner, f)
    }
}

impl Default for PackedStr {
    fn default() -> Self {
        Self {
            bytes: Default::default(),
        }
    }
}

impl PartialEq for PackedStr {
    fn eq(&self, other: &Self) -> bool {
        let self_slice = self.deref();
        let other_slice = other.deref();
        self_slice == other_slice
    }
}

impl Eq for PackedStr {}

impl Hash for PackedStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let self_slice = self.deref();
        self_slice.hash(state);
    }
}

impl<O> From<O> for PackedStr
where
    O: ByteOwner + AsRef<str>,
{
    fn from(value: O) -> Self {
        PackedStr {
            bytes: Bytes::from_owner(value),
        }
    }
}

impl<O> From<Arc<O>> for PackedStr
where
    O: ByteOwner + AsRef<str>,
{
    fn from(value: Arc<O>) -> Self {
        PackedStr {
            bytes: Bytes::from_arc(value),
        }
    }
}

impl TryFrom<Bytes> for PackedStr {
    type Error = Utf8Error;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        std::str::from_utf8(&bytes[..])?;
        Ok(PackedStr { bytes })
    }
}

impl TryFrom<&Bytes> for PackedStr {
    type Error = Utf8Error;

    fn try_from(bytes: &Bytes) -> Result<Self, Self::Error> {
        bytes.clone().try_into()
    }
}

#[cfg(test)]
mod test {
    use crate::PackedStr;

    #[test]
    fn roundtrip_copy() {
        let v = "hello world!";
        let p = PackedStr::copy_from(&v);
        let pr: &str = p.as_ref();
        assert_eq!(v, pr)
    }

    #[test]
    fn roundtrip() {
        let v: String = "hello world!".to_string();
        let p: PackedStr = v.clone().into();
        let r: &str = &p;
        assert_eq!(&v, r)
    }
}
