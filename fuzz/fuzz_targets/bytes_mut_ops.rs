#![no_main]

use anybytes::Bytes;
use arbitrary::{Arbitrary, Result as ArbResult, Unstructured};
use libfuzzer_sys::fuzz_target;

#[derive(Debug)]
enum Operation {
    TakePrefix(u16),
    TakeSuffix(u16),
    PopFront,
    PopBack,
}

impl<'a> Arbitrary<'a> for Operation {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbResult<Self> {
        let tag = u.int_in_range::<u8>(0..=3)?;
        let op = match tag {
            0 => Operation::TakePrefix(u.arbitrary()?),
            1 => Operation::TakeSuffix(u.arbitrary()?),
            2 => Operation::PopFront,
            _ => Operation::PopBack,
        };
        Ok(op)
    }
}

#[derive(Debug)]
struct FuzzCase {
    data: Vec<u8>,
    ops: Vec<Operation>,
}

impl<'a> Arbitrary<'a> for FuzzCase {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbResult<Self> {
        let len = u.int_in_range::<usize>(0..=64)?;
        let mut data = Vec::with_capacity(len);
        for _ in 0..len {
            data.push(u.arbitrary()?);
        }

        let ops_len = u.int_in_range::<usize>(0..=64)?;
        let mut ops = Vec::with_capacity(ops_len);
        for _ in 0..ops_len {
            ops.push(u.arbitrary()?);
        }

        Ok(Self { data, ops })
    }
}

fuzz_target!(|case: FuzzCase| {
    let mut bytes = Bytes::from(case.data.clone());
    let mut model = case.data;

    for op in case.ops {
        match op {
            Operation::TakePrefix(n) => {
                let len = n as usize;
                let result = bytes.take_prefix(len);
                if len <= model.len() {
                    let expected: Vec<u8> = model.drain(..len).collect();
                    let prefix = result.expect("prefix should exist");
                    assert_eq!(prefix.as_ref(), expected.as_slice());
                    assert_eq!(bytes.as_ref(), model.as_slice());
                } else {
                    assert!(result.is_none());
                    assert_eq!(bytes.as_ref(), model.as_slice());
                }
            }
            Operation::TakeSuffix(n) => {
                let len = n as usize;
                let result = bytes.take_suffix(len);
                if len <= model.len() {
                    let split = model.len() - len;
                    let expected = model.split_off(split);
                    let suffix = result.expect("suffix should exist");
                    assert_eq!(suffix.as_ref(), expected.as_slice());
                    assert_eq!(bytes.as_ref(), model.as_slice());
                } else {
                    assert!(result.is_none());
                    assert_eq!(bytes.as_ref(), model.as_slice());
                }
            }
            Operation::PopFront => {
                let expected = if model.is_empty() {
                    None
                } else {
                    Some(model.remove(0))
                };
                let got = bytes.pop_front();
                assert_eq!(got, expected);
                assert_eq!(bytes.as_ref(), model.as_slice());
            }
            Operation::PopBack => {
                let expected = model.pop();
                let got = bytes.pop_back();
                assert_eq!(got, expected);
                assert_eq!(bytes.as_ref(), model.as_slice());
            }
        }
    }
});
