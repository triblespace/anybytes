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
        &b[..] == &a[..b.len()] && (b.is_empty() || a.as_ptr() == b.as_ptr())
    }
}

#[test]
fn test_downcast() {
    let v = b"abcd".to_vec();
    let b = Bytes::from(v);
    assert!(b.downcast_to_owner::<Vec<u8>>().is_some());
    let v = b"abcd".to_vec();
    let b = Bytes::from(v);
    assert!(b.downcast_to_owner::<String>().is_none());
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
#[test]
fn test_slice_to_bytes_same_source() {
    let bytes = Bytes::from(b"abcdef".to_vec());
    let slice = &bytes[1..4];
    let result = bytes.slice_to_bytes(slice).expect("slice from same bytes");
    assert_eq!(result, bytes.slice(1..4));
}

#[test]
fn test_slice_to_bytes_unrelated_slice() {
    let bytes = Bytes::from(b"abcdef".to_vec());
    let other = b"xyz123".to_vec();
    let slice = &other[1..4];
    assert!(bytes.slice_to_bytes(slice).is_none());
}

#[test]
fn test_weakbytes_multiple_upgrades() {
    let bytes = Bytes::from(b"hello".to_vec());
    let weak = bytes.downgrade();

    // Upgrade works while strong reference exists
    let strong1 = weak.upgrade().unwrap();
    assert_eq!(strong1.as_ref(), bytes.as_ref());
    drop(strong1);

    // Can upgrade multiple times
    let strong2 = weak.upgrade().unwrap();
    assert_eq!(strong2.as_ref(), b"hello".as_ref());

    drop(bytes);
    drop(strong2);

    // After all strong refs are gone, upgrade returns None
    assert!(weak.upgrade().is_none());
}

#[test]
fn test_weakbytes_clone_upgrade() {
    let bytes = Bytes::from(b"hello".to_vec());
    let weak = bytes.downgrade();
    let weak_clone = weak.clone();

    let strong = weak_clone.upgrade().unwrap();
    assert_eq!(strong.as_ref(), bytes.as_ref());

    drop(bytes);
    drop(strong);

    assert!(weak.upgrade().is_none());
    assert!(weak_clone.upgrade().is_none());
}

#[cfg(feature = "zerocopy")]
#[test]
fn test_weakview_downgrade_upgrade() {
    let bytes = Bytes::from(b"abcdef".to_vec());
    let view = bytes.clone().view::<[u8]>().unwrap();

    let weak = view.downgrade();
    let strong = weak.upgrade().unwrap();
    assert_eq!(strong.as_ref(), view.as_ref());

    drop(bytes);
    drop(view);
    drop(strong);

    assert!(weak.upgrade().is_none());
}

#[cfg(feature = "zerocopy")]
#[test]
fn test_weakview_clone_upgrade() {
    let bytes = Bytes::from(b"abcdef".to_vec());
    let view = bytes.clone().view::<[u8]>().unwrap();

    let weak = view.downgrade();
    let weak_clone = weak.clone();

    let strong = weak_clone.upgrade().unwrap();
    assert_eq!(strong.as_ref(), view.as_ref());

    drop(bytes);
    drop(view);
    drop(strong);

    assert!(weak.upgrade().is_none());
    assert!(weak_clone.upgrade().is_none());
}
