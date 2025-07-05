use anybytes::Bytes;

fn main() {
    // create `Bytes` from a vector
    let bytes = Bytes::from(vec![1u8, 2, 3, 4]);

    // take a zero-copy slice
    let slice = bytes.slice(1..3);

    // convert it to a typed View
    let view = slice.view::<[u8]>().unwrap();
    assert_eq!(&*view, &[2, 3]);
}
