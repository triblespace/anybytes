mod packedscalar;
mod packedslice;
mod packedstr;

pub use packedscalar::Packed;
pub use packedslice::PackedSlice;
pub use packedstr::PackedStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackError {
    BadLayout,
}
