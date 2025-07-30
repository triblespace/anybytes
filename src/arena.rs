/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Temporary byte arena backed by a file.
//!
//! The arena allows staged writing through [`ByteArena::write`]. Each
//! call returns a mutable [`Buffer`] bound to the arena so only one
//! writer can exist at a time. Finalizing the buffer via
//! [`Buffer::finish`] remaps the written range as immutable and
//! returns [`Bytes`].

use std::io::{self, Seek, SeekFrom};
use std::marker::PhantomData;

use memmap2;
use page_size;
use tempfile::NamedTempFile;

use crate::Bytes;

#[cfg(feature = "zerocopy")]
use zerocopy::{FromBytes, Immutable};

/// Alignment helper.
fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

/// Arena managing a temporary file.
#[derive(Debug)]
pub struct ByteArena {
    /// Temporary file backing the arena.
    file: NamedTempFile,
    /// Current length of initialized data in bytes.
    len: usize,
}

impl ByteArena {
    /// Create a new empty arena.
    pub fn new() -> io::Result<Self> {
        let file = NamedTempFile::new()?;
        Ok(Self { file, len: 0 })
    }

    /// Start a new write of `elems` elements of type `T`.
    pub fn write<'a, T>(&'a mut self, elems: usize) -> io::Result<Buffer<'a, T>>
    where
        T: FromBytes + Immutable,
    {
        let page = page_size::get();
        let align = core::mem::align_of::<T>();
        let len_bytes = core::mem::size_of::<T>() * elems;
        let start = align_up(self.len, align);
        let end = start + len_bytes;
        self.file.as_file_mut().set_len(end as u64)?;
        // Ensure subsequent mappings see the extended size.
        self.file.as_file_mut().seek(SeekFrom::Start(end as u64))?;

        // Map must start on a page boundary; round `start` down while
        // keeping track of how far into the mapping the buffer begins.
        let aligned_offset = start & !(page - 1);
        let offset = start - aligned_offset;
        let map_len = end - aligned_offset;

        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .offset(aligned_offset as u64)
                .len(map_len)
                .map_mut(self.file.as_file())?
        };
        Ok(Buffer {
            arena: self,
            mmap,
            start,
            offset,
            elems,
            _marker: PhantomData,
        })
    }

    fn update_len(&mut self, end: usize) {
        self.len = end;
    }

    /// Finalize the arena and return immutable bytes for the entire file.
    pub fn finish(self) -> io::Result<Bytes> {
        let file = self.file.into_file();
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
        Ok(Bytes::from_source(mmap))
    }

    /// Persist the temporary arena file to `path` and return the underlying [`File`].
    pub fn persist<P: AsRef<std::path::Path>>(self, path: P) -> io::Result<std::fs::File> {
        self.file.persist(path).map_err(Into::into)
    }
}

/// Mutable buffer for writing into a [`ByteArena`].
#[derive(Debug)]
pub struct Buffer<'a, T> {
    /// Arena that owns the underlying file.
    arena: &'a mut ByteArena,
    /// Writable mapping for the current allocation.
    mmap: memmap2::MmapMut,
    /// Start position of this buffer within the arena file in bytes.
    start: usize,
    /// Offset from the beginning of `mmap` to the start of the buffer.
    offset: usize,
    /// Number of elements in the buffer.
    elems: usize,
    /// Marker to tie the buffer to element type `T`.
    _marker: PhantomData<T>,
}

impl<'a, T> Buffer<'a, T>
where
    T: FromBytes + Immutable,
{
    /// Access the backing slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.mmap.as_mut_ptr().add(self.offset) as *mut T;
            core::slice::from_raw_parts_mut(ptr, self.elems)
        }
    }

    /// Finalize the buffer and return immutable [`Bytes`].
    pub fn finish(self) -> io::Result<Bytes> {
        self.mmap.flush()?;
        let len_bytes = self.elems * core::mem::size_of::<T>();
        let offset = self.offset;
        let arena = self.arena;
        // Convert the writable mapping into a read-only view instead of
        // unmapping and remapping the region.
        let map = self.mmap.make_read_only()?;
        arena.update_len(self.start + len_bytes);
        Ok(Bytes::from_source(map).slice(offset..offset + len_bytes))
    }
}

impl<'a, T> core::ops::Deref for Buffer<'a, T>
where
    T: FromBytes + Immutable,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = self.mmap.as_ptr().add(self.offset) as *const T;
            core::slice::from_raw_parts(ptr, self.elems)
        }
    }
}

impl<'a, T> core::ops::DerefMut for Buffer<'a, T>
where
    T: FromBytes + Immutable,
{
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.mmap.as_mut_ptr().add(self.offset) as *mut T;
            core::slice::from_raw_parts_mut(ptr, self.elems)
        }
    }
}

impl<'a, T> AsRef<[T]> for Buffer<'a, T>
where
    T: FromBytes + Immutable,
{
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, T> AsMut<[T]> for Buffer<'a, T>
where
    T: FromBytes + Immutable,
{
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}
