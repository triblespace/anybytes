use anybytes::Bytes;

fn main() {
    let bytes = Bytes::from(vec![1u8, 2, 3]);
    let original = bytes.try_unwrap_owner::<Vec<u8>>().expect("unique owner");
    assert_eq!(original, vec![1, 2, 3]);
}
