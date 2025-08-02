#![cfg(all(feature = "pyo3", feature = "winnow", feature = "zerocopy"))]

use anybytes::{winnow as ab_winnow, Bytes, View};
use pyo3::{prelude::*, types::PyBytes};
use winnow::{error::ContextError, stream::AsBytes, Parser};
use zerocopy::{Immutable, KnownLayout, TryFromBytes};

#[derive(TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
struct Header {
    magic: u16,
    value: u16,
}

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let obj = PyBytes::new(py, &[0x34, 0x12, 0x78, 0x56]);
        let mut bytes = Bytes::from_source(obj);

        let mut parser = ab_winnow::view::<Header, ContextError>;
        let header: View<Header> = parser.parse_next(&mut bytes).expect("parse header");

        assert_eq!(header.magic, 0x1234);
        assert_eq!(header.value, 0x5678);
        assert_eq!(bytes.as_bytes(), b"".as_ref());
        Ok(())
    })
}
