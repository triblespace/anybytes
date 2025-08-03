use anybytes::{area::ByteArea, Bytes};
use tempfile::tempdir;

fn main() -> std::io::Result<()> {
    let mut area = ByteArea::new()?;
    let mut sections = area.sections();

    // Reserve two sections at once and mutate them independently.
    let mut raw = sections.reserve::<u8>(4)?;
    let mut nums = sections.reserve::<u32>(2)?;

    raw.as_mut_slice().copy_from_slice(b"test");
    nums.as_mut_slice().copy_from_slice(&[1, 2]);

    // Freeze the sections into immutable `Bytes`.
    let frozen_raw: Bytes = raw.freeze()?;
    let frozen_nums: Bytes = nums.freeze()?;
    assert_eq!(frozen_raw.as_ref(), b"test");
    assert_eq!(frozen_nums.view::<[u32]>().unwrap().as_ref(), &[1, 2]);

    drop(sections);

    // Decide whether to keep the area in memory or persist it to disk.
    let memory_or_file = true;
    if memory_or_file {
        // Freeze the whole area into immutable `Bytes`.
        let all: Bytes = area.freeze()?;
        assert_eq!(&all[..4], b"test");
        assert_eq!(all.slice(4..).view::<[u32]>().unwrap().as_ref(), &[1, 2]);
    } else {
        // Persist the temporary file.
        let dir = tempdir()?;
        let path = dir.path().join("area.bin");
        area.persist(&path)?;
        assert!(path.exists());
    }

    Ok(())
}
