// Winnow integration for anybytes

use crate::Bytes;
use std::num::NonZeroUsize;
use winnow::error::{ErrMode, Needed, ParserError};
use winnow::stream::{
    AsBytes, Compare, CompareResult, FindSlice, Offset, SliceLen, Stream, StreamIsPartial,
    UpdateSlice,
};

#[cfg(feature = "zerocopy")]
use crate::view::View;
#[cfg(feature = "zerocopy")]
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

/// Checkpoint for [`Bytes`] parsing with winnow.
#[derive(Clone, Debug)]
pub struct BytesCheckpoint(Bytes);

/// Iterator yielding `(offset, byte)` pairs for [`Bytes`].
#[derive(Clone, Debug)]
pub struct BytesIterOffsets {
    bytes: Bytes,
    offset: usize,
}

impl Iterator for BytesIterOffsets {
    type Item = (usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.bytes.pop_front()?;
        let offset = self.offset;
        self.offset += 1;
        Some((offset, token))
    }
}

impl SliceLen for Bytes {
    #[inline(always)]
    fn slice_len(&self) -> usize {
        self.as_slice().len()
    }
}

impl Stream for Bytes {
    type Token = u8;
    type Slice = Bytes;

    type IterOffsets = BytesIterOffsets;

    type Checkpoint = BytesCheckpoint;

    #[inline(always)]
    fn iter_offsets(&self) -> Self::IterOffsets {
        BytesIterOffsets {
            bytes: self.clone(),
            offset: 0,
        }
    }

    #[inline(always)]
    fn eof_offset(&self) -> usize {
        self.as_slice().len()
    }

    #[inline(always)]
    fn next_token(&mut self) -> Option<Self::Token> {
        self.pop_front()
    }

    #[inline(always)]
    fn peek_token(&self) -> Option<Self::Token> {
        self.as_slice().first().copied()
    }

    #[inline(always)]
    fn offset_for<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        self.as_slice().iter().position(|b| predicate(*b))
    }

    #[inline(always)]
    fn offset_at(&self, tokens: usize) -> Result<usize, Needed> {
        let remaining = self.as_slice().len();
        if let Some(needed) = tokens.checked_sub(remaining).and_then(NonZeroUsize::new) {
            Err(Needed::Size(needed))
        } else {
            Ok(tokens)
        }
    }

    #[inline(always)]
    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        self.take_prefix(offset).expect("offset within bounds")
    }

    #[inline(always)]
    fn peek_slice(&self, offset: usize) -> Self::Slice {
        self.slice(..offset)
    }

    #[inline(always)]
    fn checkpoint(&self) -> Self::Checkpoint {
        BytesCheckpoint(self.clone())
    }

    #[inline(always)]
    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        *self = checkpoint.0.clone();
    }

    #[allow(deprecated)]
    #[inline(always)]
    fn raw(&self) -> &dyn core::fmt::Debug {
        self
    }
}

impl StreamIsPartial for Bytes {
    type PartialState = ();

    #[inline]
    fn complete(&mut self) -> Self::PartialState {}

    #[inline]
    fn restore_partial(&mut self, _state: Self::PartialState) {}

    #[inline(always)]
    fn is_partial_supported() -> bool {
        false
    }
}

impl Offset for Bytes {
    #[inline(always)]
    fn offset_from(&self, start: &Self) -> usize {
        let self_ptr = self.as_slice().as_ptr() as usize;
        let start_ptr = start.as_slice().as_ptr() as usize;
        self_ptr - start_ptr
    }
}

impl Offset<BytesCheckpoint> for Bytes {
    #[inline(always)]
    fn offset_from(&self, other: &BytesCheckpoint) -> usize {
        self.offset_from(&other.0)
    }
}

impl Offset for BytesCheckpoint {
    #[inline(always)]
    fn offset_from(&self, start: &Self) -> usize {
        self.0.offset_from(&start.0)
    }
}

impl AsBytes for Bytes {
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<T> Compare<T> for Bytes
where
    for<'a> &'a [u8]: Compare<T>,
{
    #[inline(always)]
    fn compare(&self, t: T) -> CompareResult {
        self.as_slice().compare(t)
    }
}

impl<S> FindSlice<S> for Bytes
where
    for<'a> &'a [u8]: FindSlice<S>,
{
    #[inline(always)]
    fn find_slice(&self, substr: S) -> Option<core::ops::Range<usize>> {
        self.as_slice().find_slice(substr)
    }
}

impl UpdateSlice for Bytes {
    #[inline(always)]
    fn update_slice(self, inner: Self::Slice) -> Self {
        inner
    }
}

#[cfg(feature = "zerocopy")]
/// Parse a `View` of `T` from the beginning of the input.
pub fn view<T, E>(input: &mut Bytes) -> Result<View<T>, ErrMode<E>>
where
    T: ?Sized + TryFromBytes + KnownLayout + Immutable,
    E: ParserError<Bytes>,
{
    input
        .view_prefix::<T>()
        .map_err(|_| ErrMode::Backtrack(E::from_input(input)))
}

#[cfg(feature = "zerocopy")]
/// Return a parser producing a slice-like `View` with `count` elements.
pub fn view_elems<T, E>(count: usize) -> impl winnow::Parser<Bytes, View<T>, ErrMode<E>>
where
    T: ?Sized + TryFromBytes + KnownLayout<PointerMetadata = usize> + Immutable,
    E: ParserError<Bytes>,
{
    move |input: &mut Bytes| {
        input
            .view_prefix_with_elems::<T>(count)
            .map_err(|_| ErrMode::Backtrack(E::from_input(input)))
    }
}
