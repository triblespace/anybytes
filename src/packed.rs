mod packedscalar;
mod packedslice;
mod packedstr;

use std::mem::replace;

pub use packedscalar::Packed;
pub use packedslice::PackedSlice;
pub use packedstr::PackedStr;
use zerocopy::FromBytes;

use crate::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    BadLayout,
}

impl Bytes {
    pub fn packed_prefix<T>(&mut self) -> Option<Packed<T>>
    where T: FromBytes {
        let slice = self.as_slice();
        let size = size_of::<T>();
        let (data, rest) = slice.split_at(size);
        let data = self.slice_to_bytes(data)?;
        let rest = self.slice_to_bytes(rest)?;
        let packed: Packed<T> = data.try_into().ok()?;
        _ = replace(self, rest);
        Some(packed)
    }

    pub fn packedslice_prefix<T>(&mut self, count: usize) -> Option<PackedSlice<T>>
    where T: FromBytes {
        let slice = self.as_slice();
        let size = size_of::<T>() * count;
        let (data, rest) = slice.split_at(size);
        let data = self.slice_to_bytes(data)?;
        let rest = self.slice_to_bytes(rest)?;
        let packedslice: PackedSlice<T> = data.try_into().ok()?;
        _ = replace(self, rest);
        Some(packedslice)
    }

    pub fn packedstr_prefix<T>(&mut self, count: usize) -> Option<PackedStr>
    where T: FromBytes {
        let slice = self.as_slice();
        let size = size_of::<T>() * count;
        let (data, rest) = slice.split_at(size);
        let data = self.slice_to_bytes(data)?;
        let rest = self.slice_to_bytes(rest)?;
        let packedstr: PackedStr = data.try_into().ok()?;
        _ = replace(self, rest);
        Some(packedstr)
    }

    pub fn packed_suffix<T>(&mut self) -> Option<Packed<T>>
    where T: FromBytes {
        let slice = self.as_slice();
        let size = size_of::<T>();
        let (data, rest) = slice.split_at(slice.len() - size);
        let data = self.slice_to_bytes(data)?;
        let rest = self.slice_to_bytes(rest)?;
        let packed: Packed<T> = data.try_into().ok()?;
        _ = replace(self, rest);
        Some(packed)
    }

    pub fn packedslice_suffix<T>(&mut self, count: usize) -> Option<PackedSlice<T>>
    where T: FromBytes {
        let slice = self.as_slice();
        let size = size_of::<T>() * count;
        let (data, rest) = slice.split_at(slice.len() - size);
        let data = self.slice_to_bytes(data)?;
        let rest = self.slice_to_bytes(rest)?;
        let packedslice: PackedSlice<T> = data.try_into().ok()?;
        _ = replace(self, rest);
        Some(packedslice)
    }

    pub fn packedstr_suffix(&mut self, size: usize) -> Option<PackedStr> {
        let slice = self.as_slice();
        let (data, rest) = slice.split_at(slice.len() - size);
        let data = self.slice_to_bytes(data)?;
        let rest = self.slice_to_bytes(rest)?;
        let packedstr: PackedStr = data.try_into().ok()?;
        _ = replace(self, rest);
        Some(packedstr)
    }
}