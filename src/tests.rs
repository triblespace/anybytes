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

    fn test_range_of_slice(v: Vec<u8>) -> bool {
        let a: Bytes = v.into();
        let range1 = a.len() / 3.. a.len() * 2 / 3;
        let slice = a.slice(range1.clone());
        if slice.is_empty() {
            true
        } else {
            let range2 = a.range_of_slice(&slice).unwrap();
            range1 == range2
        }
    }
}

#[test]
fn test_downcast_mut() {
    let v = b"abcd".to_vec();
    let mut b = Bytes::from(v);
    assert!(b.downcast_mut::<Vec<u8>>().is_some());
    assert!(b.downcast_mut::<String>().is_none());
    let mut c = b.clone();
    assert!(b.downcast_mut::<Vec<u8>>().is_none());
    assert!(c.downcast_mut::<Vec<u8>>().is_none());
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

    // `downgrade` -> `upgrade` returns the same buffer.
    // Slicing is ignored. Full range is used.
    let b1: crate::WeakBytes = b.slice(1..=2).downgrade().unwrap();
    let b2 = Bytes::upgrade(&b1).unwrap();
    assert_eq!(b, b2);
    assert_eq!(b.as_ptr(), b2.as_ptr());

    // `upgrade` returns `None` if all strong refs are dropped.
    drop(b2);
    drop(b);
    let b3 = Bytes::upgrade(&b1);
    assert!(b3.is_none());
}
