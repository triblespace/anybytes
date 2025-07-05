use anybytes::{Bytes, PyBytes};
use pyo3::prelude::*;

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let bytes = Bytes::from(vec![1u8, 2, 3, 4]);
        let wrapped = Py::new(py, PyBytes::new(bytes))?;

        let builtins = PyModule::import(py, "builtins")?;
        let memoryview = builtins.getattr("memoryview")?.call1((wrapped.bind(py),))?;
        let length: usize = memoryview.getattr("__len__")?.call0()?.extract()?;
        assert_eq!(length, 4);
        Ok(())
    })
}
