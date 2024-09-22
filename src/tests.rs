/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use quickcheck::quickcheck;

use crate::Bytes;

quickcheck! {
    fn test_shallow_clone(v: Vec<u8>) -> bool {
        let a: Bytes = v.into();
        let b: Bytes = a.clone();
        a == b && a.as_ptr() == b.as_ptr()
    }

    fn test_shallow_slice(v: Vec<u8>) -> bool {
        let a: Bytes = v.into();
        let b: Bytes = a.slice(..a.len() / 2);
        b == &a[..b.len()] && (b.is_empty() || a.as_ptr() == b.as_ptr())
    }
}

#[test]
fn test_downcast() {
    let v = b"abcd".to_vec();
    let b = Bytes::from(v);
    assert!(b.downcast::<Vec<u8>>().is_some());
    let v = b"abcd".to_vec();
    let b = Bytes::from(v);
    assert!(b.downcast::<String>().is_none());
}

#[test]
fn test_bytes_debug_format() {
    let v = b"printable\t\r\n\'\"\\\x00\x01\x02printable".to_vec();
    let b = Bytes::from(v);
    let escaped = format!("{:?}", b);
    let expected = r#"b"printable\t\r\n\'\"\\\x00\x01\x02printable""#;
    assert_eq!(escaped, expected);
}

#[test]
fn test_downgrade_upgrade() {
    let v = b"abcd".to_vec();
    let b = Bytes::from(v);

    // `downgrade` -> `upgrade` returns the same slice.
    let b1 = b.slice(1..=2);
    let wb = b1.downgrade();
    let b2 = wb.upgrade().unwrap();
    assert_eq!(b1, b2);

    // `upgrade` returns `None` if all strong refs are dropped.
    drop(b);
    drop(b1);
    drop(b2);
    let b3 = wb.upgrade();
    assert!(b3.is_none());
}
