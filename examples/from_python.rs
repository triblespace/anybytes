use anybytes::Bytes;
use pyo3::{prelude::*, types::PyBytes};

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let obj = PyBytes::new(py, &[5u8, 6, 7]);
        let any = Bytes::from_source(obj);
        assert_eq!(&*any, &[5, 6, 7]);
        Ok(())
    })
}
