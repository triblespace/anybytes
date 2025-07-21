/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 * Copyright (c) Jan-Paul Bultmann
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use proptest::prelude::*;

use crate::Bytes;

proptest! {
    #[test]
    fn test_shallow_clone(v in proptest::collection::vec(any::<u8>(), 0..256)) {
        let a: Bytes = v.into();
        let b: Bytes = a.clone();
        prop_assert_eq!(a.as_ref(), b.as_ref());
        prop_assert_eq!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn test_shallow_slice(v in proptest::collection::vec(any::<u8>(), 0..256)) {
        let a: Bytes = v.into();
        let b: Bytes = a.slice(..a.len() / 2);
        prop_assert_eq!(&b[..], &a[..b.len()]);
        prop_assert!(b.is_empty() || a.as_ptr() == b.as_ptr());
    }
}

#[test]
fn test_downcast() {
    let v = b"abcd".to_vec();
    let b = Bytes::from(v);
    assert!(b.downcast_to_owner::<Vec<u8>>().is_ok());

    let v = b"abcd".to_vec();
    let b = Bytes::from(v);
    match b.downcast_to_owner::<String>() {
        Ok(_) => panic!("unexpected success"),
        Err(orig) => assert_eq!(orig.as_ref(), b"abcd"),
    }
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

#[cfg(feature = "winnow")]
#[test]
fn test_winnow_stream_take() {
    use winnow::error::ContextError;
    use winnow::stream::AsBytes;
    use winnow::token::take;
    use winnow::Parser;

    let mut input = Bytes::from(vec![1u8, 2, 3, 4]);
    let mut parser = take::<_, _, ContextError>(2usize);
    let prefix: Bytes = parser.parse_next(&mut input).expect("take");
    assert_eq!(prefix.as_ref(), [1u8, 2].as_ref());
    assert_eq!(input.as_bytes(), [3u8, 4].as_ref());
}

#[cfg(all(feature = "winnow", feature = "zerocopy"))]
#[test]
fn test_winnow_view_parser() {
    use winnow::error::ContextError;
    use winnow::stream::AsBytes;
    use winnow::Parser;
    #[derive(zerocopy::TryFromBytes, zerocopy::KnownLayout, zerocopy::Immutable)]
    #[repr(C)]
    struct Pair {
        a: u16,
        b: u16,
    }

    let mut input = Bytes::from(vec![1u8, 0, 2, 0]);
    let mut parser = crate::winnow::view::<Pair, ContextError>;
    let view = parser.parse_next(&mut input).expect("view");
    assert_eq!(view.a, 1);
    assert_eq!(view.b, 2);
    assert!(input.as_bytes().is_empty());
}

#[cfg(all(feature = "winnow", feature = "zerocopy"))]
#[test]
fn test_winnow_view_elems_parser() {
    use winnow::error::ContextError;
    use winnow::stream::AsBytes;
    use winnow::Parser;

    let mut input = Bytes::from(vec![1u8, 2, 3, 4]);
    let mut parser = crate::winnow::view_elems::<[u8], ContextError>(3);
    let view = parser.parse_next(&mut input).expect("view_elems");
    assert_eq!(view.as_ref(), [1u8, 2, 3].as_ref());
    assert_eq!(input.as_bytes(), [4u8].as_ref());
}

#[cfg(feature = "mmap")]
#[test]
fn test_mmap_mut_source() {
    let mut mmap = memmap2::MmapMut::map_anon(4).expect("mmap");
    mmap.copy_from_slice(b"test");
    let bytes = Bytes::from_source(mmap);
    assert_eq!(bytes.as_ref(), b"test");
}

#[cfg(feature = "mmap")]
#[test]
fn test_map_file() {
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new().expect("temp file");
    file.write_all(b"testfile").expect("write");
    file.flush().unwrap();
    let bytes = unsafe { Bytes::map_file(&file) }.expect("map file");
    assert_eq!(bytes.as_ref(), b"testfile");
}

#[test]
fn test_cow_u8_owned_source() {
    use std::borrow::Cow;

    let owned: Cow<'static, [u8]> = Cow::Owned(vec![1, 2, 3, 4]);
    let bytes_owned = Bytes::from_source(owned.clone());
    assert_eq!(bytes_owned.as_ref(), owned.as_ref());
}

#[test]
fn test_cow_u8_borrowed_source() {
    use std::borrow::Cow;

    let borrowed: Cow<'static, [u8]> = Cow::Borrowed(b"abcd");
    let bytes_borrowed = Bytes::from_source(borrowed.clone());
    assert_eq!(bytes_borrowed.as_ref(), borrowed.as_ref());
}

#[cfg(feature = "zerocopy")]
#[test]
fn test_cow_zerocopy_owned_source() {
    use std::borrow::Cow;

    let owned: Cow<'static, [u32]> = Cow::Owned(vec![1, 2, 3, 4]);
    let bytes_owned = Bytes::from_source(owned.clone());
    assert_eq!(
        bytes_owned.as_ref(),
        zerocopy::IntoBytes::as_bytes(owned.as_ref())
    );
}

#[cfg(feature = "zerocopy")]
#[test]
fn test_cow_zerocopy_borrowed_source() {
    use std::borrow::Cow;

    static BORROWED: [u32; 2] = [5, 6];
    let borrowed: Cow<'static, [u32]> = Cow::Borrowed(&BORROWED);
    let bytes_borrowed = Bytes::from_source(borrowed.clone());
    assert_eq!(
        bytes_borrowed.as_ref(),
        zerocopy::IntoBytes::as_bytes(borrowed.as_ref())
    );
}
