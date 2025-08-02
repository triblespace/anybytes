#![cfg(all(feature = "pyo3", feature = "winnow"))]

use anybytes::Bytes;
use pyo3::{prelude::*, types::PyBytes};
use winnow::{error::ContextError, stream::AsBytes, token::take, Parser};

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let obj = PyBytes::new(py, b"abcde");
        let mut bytes = Bytes::from_source(obj);

        // Take the first three bytes using a winnow parser
        let mut parser = take::<_, _, ContextError>(3usize);
        let prefix: Bytes = parser.parse_next(&mut bytes).expect("take");
        assert_eq!(prefix.as_bytes(), b"abc".as_ref());
        assert_eq!(bytes.as_bytes(), b"de".as_ref());
        Ok(())
    })
}
