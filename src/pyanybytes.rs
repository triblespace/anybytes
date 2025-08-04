/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use pyo3::{ffi, prelude::*, PyResult};
use std::os::raw::c_int;

use crate::Bytes;

/// Python wrapper around [`Bytes`].
#[pyclass(name = "Bytes")]
pub struct PyAnyBytes {
    bytes: Bytes,
}

#[pymethods]
impl PyAnyBytes {
    /// Exposes the bytes to Python's buffer protocol.
    ///
    /// # Safety
    /// This follows the semantics of the CPython `__getbuffer__` hook and is
    /// therefore unsafe.
    unsafe fn __getbuffer__(
        slf: PyRefMut<Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        let bytes = slf.bytes.as_slice();
        let ret = ffi::PyBuffer_FillInfo(
            view,
            slf.as_ptr() as *mut _,
            bytes.as_ptr() as *mut _,
            bytes.len().try_into().unwrap(),
            1, // read only
            flags,
        );
        if ret == -1 {
            return Err(PyErr::fetch(slf.py()));
        }
        Ok(())
    }
}

impl PyAnyBytes {
    /// Wrap a [`Bytes`] instance for Python exposure.
    pub fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }
}
