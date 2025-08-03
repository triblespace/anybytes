#![cfg(all(feature = "mmap", feature = "zerocopy"))]

use anybytes::area::ByteArea;
use proptest::prelude::*;

fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

#[derive(Debug, Clone)]
enum Segment {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, .. ProptestConfig::default() })]
    #[test]
    fn freeze_preserves_layout(segs in prop::collection::vec(
        prop_oneof![
            prop::collection::vec(any::<u8>(), 1..4).prop_map(Segment::U8),
            prop::collection::vec(any::<u16>(), 1..4).prop_map(Segment::U16),
            prop::collection::vec(any::<u32>(), 1..4).prop_map(Segment::U32),
        ],
        0..4,
    )) {
        let mut area = ByteArea::new().expect("area");
        let mut sections = area.sections();
        let mut expected: Vec<u8> = Vec::new();

        for seg in segs {
            match seg {
                Segment::U8(data) => {
                    let start = align_up(expected.len(), core::mem::align_of::<u8>());
                    expected.resize(start, 0);
                    let mut section = sections.reserve::<u8>(data.len()).expect("reserve u8");
                    section.as_mut_slice().copy_from_slice(&data);
                    let bytes = section.freeze().expect("freeze");
                    expected.extend_from_slice(&data);
                    let end = expected.len();
                    prop_assert_eq!(bytes.as_ref(), &expected[start..end]);
                }
                Segment::U16(data) => {
                    let start = align_up(expected.len(), core::mem::align_of::<u16>());
                    expected.resize(start, 0);
                    let mut section = sections.reserve::<u16>(data.len()).expect("reserve u16");
                    section.as_mut_slice().copy_from_slice(&data);
                    let bytes = section.freeze().expect("freeze");
                    for v in &data {
                        expected.extend_from_slice(&v.to_ne_bytes());
                    }
                    let end = expected.len();
                    prop_assert_eq!(bytes.as_ref(), &expected[start..end]);
                }
                Segment::U32(data) => {
                    let start = align_up(expected.len(), core::mem::align_of::<u32>());
                    expected.resize(start, 0);
                    let mut section = sections.reserve::<u32>(data.len()).expect("reserve u32");
                    section.as_mut_slice().copy_from_slice(&data);
                    let bytes = section.freeze().expect("freeze");
                    for v in &data {
                        expected.extend_from_slice(&v.to_ne_bytes());
                    }
                    let end = expected.len();
                    prop_assert_eq!(bytes.as_ref(), &expected[start..end]);
                }
            }
        }

        drop(sections);
        let all = area.freeze().expect("freeze area");
        prop_assert_eq!(all.as_ref(), expected.as_slice());
    }
}
