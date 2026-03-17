//! Miri test suite targeting unsafe code paths in anybytes.
//!
//! These tests exercise the lifetime erasure, raw pointer dereferences, and
//! ownership tricks that Kani cannot verify. Run with:
//!
//! ```sh
//! cargo +nightly miri test --test miri
//! ```
//!
//! Miri detects undefined behavior such as use-after-free, dangling pointer
//! dereferences, and Stacked Borrows violations, complementing Kani's
//! functional correctness proofs.

use anybytes::Bytes;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// erase_lifetime / from_source soundness
// ---------------------------------------------------------------------------

/// The fundamental invariant: data obtained from a `ByteSource` remains valid
/// as long as the `Bytes` (and its Arc owner) is alive.
#[test]
fn from_source_vec_data_valid() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3, 4]);
    assert_eq!(bytes.as_ref(), &[1, 2, 3, 4]);
}

/// Slicing creates a new Bytes sharing the owner. The original can be dropped
/// and the slice must still be valid (erase_lifetime on the subslice).
#[test]
fn slice_survives_original_drop() {
    let bytes = Bytes::from_source(vec![10u8, 20, 30, 40, 50]);
    let slice = bytes.slice(1..4);
    drop(bytes);
    assert_eq!(slice.as_ref(), &[20, 30, 40]);
}

/// Multiple levels of slicing, each sharing the same owner.
#[test]
fn nested_slices_survive_drops() {
    let bytes = Bytes::from_source(vec![0u8, 1, 2, 3, 4, 5, 6, 7]);
    let a = bytes.slice(2..6);
    let b = a.slice(1..3);
    drop(bytes);
    drop(a);
    assert_eq!(b.as_ref(), &[3, 4]);
}

/// `slice_to_bytes` erases the lifetime of a derived subslice.
#[test]
fn slice_to_bytes_lifetime_erasure() {
    let bytes = Bytes::from_source(vec![10u8, 20, 30, 40]);
    let inner = &bytes.as_ref()[1..3];
    let sub = bytes.slice_to_bytes(inner).expect("subslice");
    drop(bytes);
    assert_eq!(sub.as_ref(), &[20, 30]);
}

/// Empty source is a valid edge case for lifetime erasure.
#[test]
fn empty_source_is_sound() {
    let bytes = Bytes::from_source(Vec::<u8>::new());
    assert!(bytes.is_empty());
    let clone = bytes.clone();
    drop(bytes);
    assert!(clone.is_empty());
}

/// Static source: lifetime erasure is trivially sound for 'static data.
#[test]
fn static_source_is_sound() {
    let bytes = Bytes::from_source(&b"hello"[..]);
    let slice = bytes.slice(1..4);
    drop(bytes);
    assert_eq!(slice.as_ref(), b"ell");
}

/// String source exercises a different ByteSource impl.
#[test]
fn string_source_is_sound() {
    let bytes = Bytes::from_source(String::from("hello world"));
    let slice = bytes.slice(6..11);
    drop(bytes);
    assert_eq!(slice.as_ref(), b"world");
}

/// Arc<Vec<u8>> source reuses the Arc without extra allocation.
#[test]
fn arc_source_is_sound() {
    let arc = Arc::new(vec![1u8, 2, 3]);
    let bytes = Bytes::from_owning_source_arc(arc);
    let slice = bytes.slice(1..3);
    drop(bytes);
    assert_eq!(slice.as_ref(), &[2, 3]);
}

// ---------------------------------------------------------------------------
// WeakBytes: raw pointer deref in upgrade()
// ---------------------------------------------------------------------------

/// upgrade() dereferences `self.data` (a raw pointer) only after confirming
/// the owner is still alive via Weak::upgrade(). Miri validates the pointer.
#[test]
fn weakbytes_upgrade_while_alive() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let weak = bytes.downgrade();
    let upgraded = weak.upgrade().expect("owner alive");
    assert_eq!(upgraded.as_ref(), &[1, 2, 3]);
}

/// After all strong references are dropped, upgrade must return None
/// without dereferencing the dangling pointer.
#[test]
fn weakbytes_upgrade_after_drop() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let weak = bytes.downgrade();
    drop(bytes);
    assert!(weak.upgrade().is_none());
}

/// Weak from a slice: the raw pointer points into the middle of the
/// allocation. Upgrading must still be valid.
#[test]
fn weakbytes_from_slice() {
    let bytes = Bytes::from_source(vec![10u8, 20, 30, 40, 50]);
    let slice = bytes.slice(2..4);
    let weak = slice.downgrade();
    drop(slice);
    let upgraded = weak.upgrade().expect("original keeps owner alive");
    assert_eq!(upgraded.as_ref(), &[30, 40]);
    drop(bytes);
    drop(upgraded);
    assert!(weak.upgrade().is_none());
}

/// Multiple weak references from different slices of the same owner.
#[test]
fn weakbytes_multiple_from_same_owner() {
    let bytes = Bytes::from_source(vec![0u8, 1, 2, 3, 4]);
    let w1 = bytes.slice(0..2).downgrade();
    let w2 = bytes.slice(3..5).downgrade();

    assert_eq!(w1.upgrade().unwrap().as_ref(), &[0, 1]);
    assert_eq!(w2.upgrade().unwrap().as_ref(), &[3, 4]);

    drop(bytes);
    assert!(w1.upgrade().is_none());
    assert!(w2.upgrade().is_none());
}

/// Clone a WeakBytes, drop original weak, upgrade the clone.
#[test]
fn weakbytes_clone_then_upgrade() {
    let bytes = Bytes::from_source(vec![5u8, 6, 7]);
    let weak = bytes.downgrade();
    let weak2 = weak.clone();
    drop(weak);
    let strong = weak2.upgrade().expect("clone still valid");
    assert_eq!(strong.as_ref(), &[5, 6, 7]);
}

// ---------------------------------------------------------------------------
// try_unwrap_owner: the data_ptr trick
// ---------------------------------------------------------------------------

/// Success path: unique owner, Arc::try_unwrap succeeds.
/// The raw `data_ptr` is never dereferenced.
#[test]
fn try_unwrap_owner_unique() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let v = bytes.try_unwrap_owner::<Vec<u8>>().expect("unique owner");
    assert_eq!(v, vec![1u8, 2, 3]);
}

/// Failure path (shared): `data` is converted to raw pointer, dynamic Arc is
/// dropped, then Arc::try_unwrap fails, and the raw pointer is dereferenced
/// to reconstruct the Bytes. This is the critical unsafe path.
#[test]
fn try_unwrap_owner_shared_reconstructs() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let clone = bytes.clone();
    let err = bytes.try_unwrap_owner::<Vec<u8>>().unwrap_err();
    assert_eq!(err.as_ref(), &[1, 2, 3]);
    assert_eq!(clone.as_ref(), &[1, 2, 3]);
}

/// Failure path (wrong type): early return before the data_ptr trick.
#[test]
fn try_unwrap_owner_wrong_type() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let err = bytes.try_unwrap_owner::<String>().unwrap_err();
    assert_eq!(err.as_ref(), &[1, 2, 3]);
}

/// try_unwrap_owner on a sliced Bytes: the data pointer points into the
/// middle of the allocation, not to its start.
#[test]
fn try_unwrap_owner_after_slice() {
    let mut bytes = Bytes::from_source(vec![10u8, 20, 30, 40]);
    let _ = bytes.take_prefix(1);
    // data now points to offset 1 in the Vec's allocation
    let clone = bytes.clone();
    let err = bytes.try_unwrap_owner::<Vec<u8>>().unwrap_err();
    assert_eq!(err.as_ref(), &[20, 30, 40]);
    drop(clone);
    // Now unique - but data ptr still points into middle of Vec
    let v = err.try_unwrap_owner::<Vec<u8>>().expect("now unique");
    assert_eq!(v, vec![10u8, 20, 30, 40]);
}

// ---------------------------------------------------------------------------
// downcast_to_owner
// ---------------------------------------------------------------------------

/// Downcast success: the Arc is cloned and downcast.
#[test]
fn downcast_to_owner_success() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let arc: Arc<Vec<u8>> = bytes.downcast_to_owner().expect("downcast");
    assert_eq!(&*arc, &[1, 2, 3]);
}

/// Downcast failure: original Bytes is returned intact.
#[test]
fn downcast_to_owner_failure_preserves_data() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3]);
    let err = bytes.downcast_to_owner::<String>().unwrap_err();
    assert_eq!(err.as_ref(), &[1, 2, 3]);
}

// ---------------------------------------------------------------------------
// take_prefix / take_suffix / pop_front / pop_back: data pointer mutation
// ---------------------------------------------------------------------------

/// take_prefix mutates self.data via split_at. The erased-lifetime pointer
/// must remain valid.
#[test]
fn take_prefix_pointer_remains_valid() {
    let mut bytes = Bytes::from_source(vec![1u8, 2, 3, 4, 5]);
    let prefix = bytes.take_prefix(3).unwrap();
    assert_eq!(prefix.as_ref(), &[1, 2, 3]);
    assert_eq!(bytes.as_ref(), &[4, 5]);
    // Drop prefix, remainder must still work
    drop(prefix);
    assert_eq!(bytes.as_ref(), &[4, 5]);
}

/// Exhaustive pop_front draining every byte.
#[test]
fn pop_front_until_empty() {
    let mut bytes = Bytes::from_source(vec![10u8, 20, 30]);
    assert_eq!(bytes.pop_front(), Some(10));
    assert_eq!(bytes.pop_front(), Some(20));
    assert_eq!(bytes.pop_front(), Some(30));
    assert_eq!(bytes.pop_front(), None);
    assert!(bytes.is_empty());
}

/// Interleaving pop_front and pop_back.
#[test]
fn interleaved_pop_front_back() {
    let mut bytes = Bytes::from_source(vec![1u8, 2, 3, 4, 5]);
    assert_eq!(bytes.pop_front(), Some(1));
    assert_eq!(bytes.pop_back(), Some(5));
    assert_eq!(bytes.pop_front(), Some(2));
    assert_eq!(bytes.pop_back(), Some(4));
    assert_eq!(bytes.pop_front(), Some(3));
    assert_eq!(bytes.pop_front(), None);
    assert_eq!(bytes.pop_back(), None);
}

// ---------------------------------------------------------------------------
// View: get_data/set_data, view construction, field_to_view
// ---------------------------------------------------------------------------

#[cfg(feature = "zerocopy")]
mod view_tests {
    use anybytes::Bytes;

    #[derive(
        zerocopy::TryFromBytes,
        zerocopy::IntoBytes,
        zerocopy::KnownLayout,
        zerocopy::Immutable,
        Clone,
        Copy,
        Debug,
        PartialEq,
    )]
    #[repr(C)]
    struct Pair {
        a: u32,
        b: u32,
    }

    /// View construction uses get_data() (unsafe lifetime-erased access)
    /// and the zerocopy try_ref_from_bytes path.
    #[test]
    fn view_construction_is_sound() {
        let pair = Pair { a: 42, b: 99 };
        let bytes = Bytes::from_source(Box::new(pair));
        let view = bytes.view::<Pair>().unwrap();
        assert_eq!(*view, Pair { a: 42, b: 99 });
    }

    /// View::bytes() roundtrip: View -> Bytes uses from_raw_parts.
    #[test]
    fn view_to_bytes_roundtrip() {
        let pair = Pair { a: 1, b: 2 };
        let bytes = Bytes::from_source(Box::new(pair));
        let view = bytes.view::<Pair>().unwrap();
        let back = view.bytes();
        let view2 = back.view::<Pair>().unwrap();
        assert_eq!(*view2, Pair { a: 1, b: 2 });
    }

    /// view_prefix uses set_data() to advance the internal pointer.
    #[test]
    fn view_prefix_advances_pointer() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3, 4, 5, 6, 7, 8]);
        let view = bytes.view_prefix::<[u8; 4]>().unwrap();
        assert_eq!(*view, [1, 2, 3, 4]);
        assert_eq!(bytes.as_ref(), &[5, 6, 7, 8]);
        // Drop view, remainder must still be valid
        drop(view);
        assert_eq!(bytes.as_ref(), &[5, 6, 7, 8]);
    }

    /// view_suffix uses set_data() to shrink the internal pointer.
    #[test]
    fn view_suffix_shrinks_pointer() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3, 4, 5, 6, 7, 8]);
        let view = bytes.view_suffix::<[u8; 4]>().unwrap();
        assert_eq!(*view, [5, 6, 7, 8]);
        assert_eq!(bytes.as_ref(), &[1, 2, 3, 4]);
        drop(view);
        assert_eq!(bytes.as_ref(), &[1, 2, 3, 4]);
    }

    /// view_prefix_with_elems: slice-like view with dynamic count.
    #[test]
    fn view_prefix_with_elems_is_sound() {
        let mut bytes = Bytes::from_source(vec![10u8, 20, 30, 40, 50]);
        let view = bytes.view_prefix_with_elems::<[u8]>(3).unwrap();
        assert_eq!(view.as_ref(), &[10, 20, 30]);
        assert_eq!(bytes.as_ref(), &[40, 50]);
    }

    /// view_suffix_with_elems: slice-like view from the end.
    #[test]
    fn view_suffix_with_elems_is_sound() {
        let mut bytes = Bytes::from_source(vec![10u8, 20, 30, 40, 50]);
        let view = bytes.view_suffix_with_elems::<[u8]>(3).unwrap();
        assert_eq!(view.as_ref(), &[30, 40, 50]);
        assert_eq!(bytes.as_ref(), &[10, 20]);
    }

    /// field_to_view: derives a sub-view from a field reference,
    /// erasing its lifetime via erase_lifetime.
    #[test]
    fn field_to_view_is_sound() {
        let pair = Pair { a: 7, b: 13 };
        let bytes = Bytes::from_source(Box::new(pair));
        let view = bytes.view::<Pair>().unwrap();
        let field_view = view.field_to_view(&view.a).expect("field view");
        assert_eq!(*field_view, 7u32);
        // Drop parent view, field view keeps owner alive
        drop(view);
        assert_eq!(*field_view, 7u32);
    }

    /// field_to_view with an unrelated reference must return None.
    #[test]
    fn field_to_view_rejects_unrelated() {
        let pair = Pair { a: 7, b: 13 };
        let bytes = Bytes::from_source(Box::new(pair));
        let view = bytes.view::<Pair>().unwrap();
        let unrelated: u32 = 42;
        assert!(view.field_to_view(&unrelated).is_none());
    }

    /// WeakView upgrade dereferences a raw pointer to T.
    #[test]
    fn weakview_upgrade_while_alive() {
        let bytes = Bytes::from_source(vec![1u8, 2, 3, 4]);
        let view = bytes.clone().view::<[u8]>().unwrap();
        let weak = view.downgrade();
        let upgraded = weak.upgrade().expect("alive");
        assert_eq!(upgraded.as_ref(), &[1, 2, 3, 4]);
    }

    /// WeakView upgrade after all strong refs dropped.
    #[test]
    fn weakview_upgrade_after_drop() {
        let bytes = Bytes::from_source(vec![1u8, 2, 3, 4]);
        let view = bytes.view::<[u8]>().unwrap();
        let weak = view.downgrade();
        drop(view);
        assert!(weak.upgrade().is_none());
    }

    /// WeakView from a struct view, checking that the typed pointer
    /// (not just byte pointer) is correctly handled.
    #[test]
    fn weakview_typed_struct() {
        let pair = Pair { a: 100, b: 200 };
        let bytes = Bytes::from_source(Box::new(pair));
        let view = bytes.view::<Pair>().unwrap();
        let weak = view.downgrade();
        let upgraded = weak.upgrade().expect("alive");
        assert_eq!(upgraded.a, 100);
        assert_eq!(upgraded.b, 200);
        drop(view);
        drop(upgraded);
        assert!(weak.upgrade().is_none());
    }

    /// Multiple view_prefix calls chained together.
    #[test]
    fn chained_view_prefix() {
        let mut bytes = Bytes::from_source(vec![1u8, 2, 3, 4, 5, 6]);
        let v1 = bytes.view_prefix::<[u8; 2]>().unwrap();
        let v2 = bytes.view_prefix::<[u8; 2]>().unwrap();
        assert_eq!(*v1, [1, 2]);
        assert_eq!(*v2, [3, 4]);
        assert_eq!(bytes.as_ref(), &[5, 6]);
        drop(v1);
        drop(v2);
        assert_eq!(bytes.as_ref(), &[5, 6]);
    }
}

// ---------------------------------------------------------------------------
// Complex drop orderings
// ---------------------------------------------------------------------------

/// Create a web of clones and slices, drop in various orders.
#[test]
fn complex_drop_ordering() {
    let original = Bytes::from_source(vec![0u8, 1, 2, 3, 4, 5, 6, 7]);
    let clone1 = original.clone();
    let slice1 = original.slice(2..6);
    let slice2 = slice1.slice(1..3);
    let weak = slice2.downgrade();

    drop(original);
    assert_eq!(clone1.as_ref(), &[0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(slice1.as_ref(), &[2, 3, 4, 5]);
    assert_eq!(slice2.as_ref(), &[3, 4]);

    drop(slice1);
    assert_eq!(slice2.as_ref(), &[3, 4]);

    let upgraded = weak.upgrade().expect("still alive");
    assert_eq!(upgraded.as_ref(), &[3, 4]);

    drop(clone1);
    drop(slice2);
    drop(upgraded);
    assert!(weak.upgrade().is_none());
}

/// try_unwrap_owner interleaved with slicing and cloning.
#[test]
fn try_unwrap_after_clone_drop_sequence() {
    let bytes = Bytes::from_source(vec![1u8, 2, 3, 4]);
    let clone = bytes.clone();
    let slice = bytes.slice(1..3);

    // Three strong refs (bytes, clone, slice) -> unwrap fails
    let err = bytes.try_unwrap_owner::<Vec<u8>>().unwrap_err();
    assert_eq!(err.as_ref(), &[1, 2, 3, 4]);

    drop(err);
    drop(slice);
    // Only clone remains -> unwrap succeeds
    let v = clone.try_unwrap_owner::<Vec<u8>>().expect("unique");
    assert_eq!(v, vec![1, 2, 3, 4]);
}
