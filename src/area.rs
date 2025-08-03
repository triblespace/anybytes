/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Temporary byte area backed by a file.
//!
//! The area offers staged writing through a [`SectionWriter`]. Each call to
//! [`SectionWriter::reserve`] returns a mutable [`Section`] tied to the area's
//! lifetime. Multiple sections may coexist; their byte ranges do not overlap.
//! Freezing a section via [`Section::freeze`] remaps its range as immutable and
//! returns [`Bytes`].
//!
//! # Examples
//!
//! ```
//! # #[cfg(all(feature = "mmap", feature = "zerocopy"))]
//! # {
//! use anybytes::area::ByteArea;
//!
//! let mut area = ByteArea::new().unwrap();
//! let mut sections = area.sections();
//!
//! let mut a = sections.reserve::<u8>(1).unwrap();
//! a.as_mut_slice()[0] = 1;
//!
//! let mut b = sections.reserve::<u32>(1).unwrap();
//! b.as_mut_slice()[0] = 2;
//!
//! let bytes_a = a.freeze().unwrap();
//! let bytes_b = b.freeze().unwrap();
//! drop(sections);
//! let all = area.freeze().unwrap();
//!
//! assert_eq!(bytes_a.as_ref(), &[1]);
//! assert_eq!(bytes_b.as_ref(), &2u32.to_ne_bytes());
//!
//! let mut expected = Vec::new();
//! expected.extend_from_slice(&[1]);
//! expected.extend_from_slice(&[0; 3]);
//! expected.extend_from_slice(&2u32.to_ne_bytes());
//! assert_eq!(all.as_ref(), expected.as_slice());
//! # }
//! ```

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

/// Area managing a temporary file.
#[derive(Debug)]
pub struct ByteArea {
    /// Temporary file backing the area.
    file: NamedTempFile,
    /// Current length of initialized data in bytes.
    len: usize,
}

impl ByteArea {
    /// Create a new empty area.
    pub fn new() -> io::Result<Self> {
        let file = NamedTempFile::new()?;
        Ok(Self { file, len: 0 })
    }

    /// Obtain a handle for reserving sections.
    pub fn sections(&mut self) -> SectionWriter<'_> {
        SectionWriter { area: self }
    }

    /// Freeze the area and return immutable bytes for the entire file.
    pub fn freeze(self) -> io::Result<Bytes> {
        let file = self.file.into_file();
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };
        Ok(Bytes::from_source(mmap))
    }

    /// Persist the temporary area file to `path` and return the underlying [`File`].
    pub fn persist<P: AsRef<std::path::Path>>(self, path: P) -> io::Result<std::fs::File> {
        self.file.persist(path).map_err(Into::into)
    }
}

/// RAII guard giving temporary exclusive write access.
#[derive(Debug)]
pub struct SectionWriter<'area> {
    area: &'area mut ByteArea,
}

impl<'area> SectionWriter<'area> {
    /// Reserve a new section inside the area.
    pub fn reserve<T>(&mut self, elems: usize) -> io::Result<Section<'area, T>>
    where
        T: FromBytes + Immutable,
    {
        let page = page_size::get();
        let align = core::mem::align_of::<T>();
        let len_bytes = core::mem::size_of::<T>() * elems;
        let start = align_up(self.area.len, align);
        let end = start + len_bytes;

        let aligned_offset = start & !(page - 1);
        let offset = start - aligned_offset;
        let map_len = end - aligned_offset;

        let file = &mut self.area.file;
        file.as_file_mut().set_len(end as u64)?;
        // Ensure subsequent mappings see the extended size.
        file.as_file_mut().seek(SeekFrom::Start(end as u64))?;
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .offset(aligned_offset as u64)
                .len(map_len)
                .map_mut(file.as_file())?
        };

        self.area.len = end;

        Ok(Section {
            mmap,
            offset,
            elems,
            _marker: PhantomData,
        })
    }
}

/// Mutable section reserved from a [`ByteArea`].
#[derive(Debug)]
pub struct Section<'arena, T> {
    /// Writable mapping for the current allocation.
    mmap: memmap2::MmapMut,
    /// Offset from the beginning of `mmap` to the start of the buffer.
    offset: usize,
    /// Number of elements in the buffer.
    elems: usize,
    /// Marker tying the section to the area and element type.
    _marker: PhantomData<(&'arena ByteArea, *mut T)>,
}

impl<'arena, T> Section<'arena, T>
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

    /// Freeze the section and return immutable [`Bytes`].
    pub fn freeze(self) -> io::Result<Bytes> {
        self.mmap.flush()?;
        let len_bytes = self.elems * core::mem::size_of::<T>();
        let offset = self.offset;
        // Convert the writable mapping into a read-only view instead of
        // unmapping and remapping the region.
        let map = self.mmap.make_read_only()?;
        Ok(Bytes::from_source(map).slice(offset..offset + len_bytes))
    }
}

impl<'arena, T> core::ops::Deref for Section<'arena, T>
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

impl<'arena, T> core::ops::DerefMut for Section<'arena, T>
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

impl<'arena, T> AsRef<[T]> for Section<'arena, T>
where
    T: FromBytes + Immutable,
{
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'arena, T> AsMut<[T]> for Section<'arena, T>
where
    T: FromBytes + Immutable,
{
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}
