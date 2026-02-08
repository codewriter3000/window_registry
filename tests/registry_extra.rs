use std::sync::Arc;

use window_registry::{LifecycleState, Registry, RegistryError, SharedRegistry};

mod common;
use common::{assert_shared_registry_invariants, TestPtrs};

#[test]
fn reverse_lookup_cleared_on_remove_window() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");

    assert_eq!(reg.from_desktop(dk), Some(id));
    assert_eq!(reg.from_surface(sk), Some(id));

    let (_record, _events) = reg.remove_window(id).expect("remove_window should succeed");

    assert_eq!(reg.from_desktop(dk), None);
    assert_eq!(reg.from_surface(sk), None);
}

#[test]
fn reverse_lookup_persists_on_remove() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let removed = reg.remove(id).expect("remove should succeed");
    assert_eq!(removed.id, id);

    assert_eq!(reg.from_desktop(dk), Some(id));
    assert_eq!(reg.from_surface(sk), Some(id));
    assert!(reg.snapshot(id).is_none(), "stale id should not resolve");

    let err = reg.insert_window(dk, sk).expect_err("maps still reserve dk/sk");
    assert!(matches!(
        err,
        RegistryError::DesktopKeyAlreadyRegistered { dk: err_dk, existing }
            if err_dk == dk && existing == id
    ));
}

#[test]
fn slot_reuse_invalidates_old_id() {
    let mut reg = Registry::new();

    let p1 = TestPtrs::new();
    let (dk1, sk1) = unsafe { p1.keys() };
    let (id1, _events) = reg.insert_window(dk1, sk1).expect("insert_window A");
    let (_record, _events) = reg.remove_window(id1).expect("remove_window A");

    let p2 = TestPtrs::new();
    let (dk2, sk2) = unsafe { p2.keys() };
    let (id2, _events) = reg.insert_window(dk2, sk2).expect("insert_window B");

    assert!(reg.snapshot(id1).is_none(), "old id should be stale after reuse");
    assert!(reg.snapshot(id2).is_some(), "new id should resolve");
    assert_ne!(id1, id2, "ids should differ after reuse");
}

#[test]
fn invariants_hold_across_multiple_windows() {
    let reg = SharedRegistry::new(Registry::new());

    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let p3 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let (dk3, sk3) = unsafe { p3.keys() };

    let id1 = reg.insert_window_with(dk1, sk1, |_| {}).expect("insert A");
    let id2 = reg.insert_window_with(dk2, sk2, |_| {}).expect("insert B");
    let id3 = reg.insert_window_with(dk3, sk3, |_| {}).expect("insert C");

    assert_shared_registry_invariants(&reg);

    reg.remove_window_with(id2, |_| {}).expect("remove B");
    reg.remove_window_with(id1, |_| {}).expect("remove A");
    reg.remove_window_with(id3, |_| {}).expect("remove C");

    assert_shared_registry_invariants(&reg);
}

#[test]
fn concurrent_insert_remove_is_safe() {
    let reg = Arc::new(SharedRegistry::new(Registry::new()));

    let mut handles = Vec::new();
    for _ in 0..8 {
        let reg_thread = Arc::clone(&reg);
        handles.push(std::thread::spawn(move || {
            let p = TestPtrs::new();
            let (dk, sk) = unsafe { p.keys() };
            let id = reg_thread
                .insert_window_with(dk, sk, |_| {})
                .expect("insert_window_with should succeed");

            let snap = reg_thread.snapshot(id).expect("snapshot should exist");
            assert_eq!(snap.id, id);

            reg_thread.remove_window_with(id, |_| {}).expect("remove_window_with should succeed");
        }));
    }

    for handle in handles {
        handle.join().expect("thread should finish");
    }

    assert!(reg.snapshot_all().is_empty(), "registry should be empty after all threads");
}

#[test]
fn snapshot_all_consistent_after_mixed_operations() {
    let mut reg = Registry::new();

    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let p3 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let (dk3, sk3) = unsafe { p3.keys() };

    let (id1, _events) = reg.insert_window(dk1, sk1).expect("insert A");
    let (id2, _events) = reg.insert_window(dk2, sk2).expect("insert B");
    let (id3, _events) = reg.insert_window(dk3, sk3).expect("insert C");

    reg.on_map(id1).expect("map A");
    reg.on_unmap(id1).expect("unmap A");
    reg.on_map(id2).expect("map B");
    reg.remove_window(id2).expect("remove B");

    let all = reg.snapshot_all();
    assert_eq!(all.len(), 2);

    let mut by_id = std::collections::HashMap::new();
    for w in all {
        by_id.insert(w.id, w.lifecycle);
    }

    assert_eq!(by_id.get(&id1), Some(&LifecycleState::Unmapped));
    assert_eq!(by_id.get(&id3), Some(&LifecycleState::Created));
    assert!(!by_id.contains_key(&id2));
}

#[test]
fn remove_twice_returns_none() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");

    assert!(reg.remove(id).is_some(), "first remove should succeed");
    assert!(reg.remove(id).is_none(), "second remove should return None");
    assert!(reg.snapshot(id).is_none(), "removed id should not resolve");
}

#[test]
fn reverse_lookups_unchanged_after_failed_insert() {
    let mut reg = Registry::new();

    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };

    let (id1, _events) = reg.insert_window(dk1, sk1).expect("insert A");

    let err = reg.insert_window(dk1, sk2).expect_err("duplicate dk should fail");
    assert!(matches!(
        err,
        RegistryError::DesktopKeyAlreadyRegistered { dk, existing }
            if dk == dk1 && existing == id1
    ));

    assert_eq!(reg.from_desktop(dk1), Some(id1));
    assert_eq!(reg.from_surface(sk1), Some(id1));

    let err = reg.insert_window(dk2, sk1).expect_err("duplicate sk should fail");
    assert!(matches!(
        err,
        RegistryError::SurfaceKeyAlreadyRegistered { sk, existing }
            if sk == sk1 && existing == id1
    ));

    assert_eq!(reg.from_desktop(dk1), Some(id1));
    assert_eq!(reg.from_surface(sk1), Some(id1));
}

#[test]
fn insert_low_level_allocates_new_id() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");
    let (record, _events) = reg.remove_window(id).expect("remove_window should succeed");

    let new_id = reg.insert(record);
    let stored = reg.get(new_id).expect("record should be stored");
    assert_eq!(stored.id, id, "low-level insert preserves record contents");
    assert_ne!(new_id, id, "new allocation should yield a fresh id");
}

#[test]
fn generation_mismatch_paths_return_none_or_error() {
    let mut reg = Registry::new();

    let p1 = TestPtrs::new();
    let (dk1, sk1) = unsafe { p1.keys() };
    let (old_id, _events) = reg.insert_window(dk1, sk1).expect("insert_window A");
    reg.remove_window(old_id).expect("remove_window A");

    let p2 = TestPtrs::new();
    let (dk2, sk2) = unsafe { p2.keys() };
    let (new_id, _events) = reg.insert_window(dk2, sk2).expect("insert_window B");

    assert!(reg.get_mut(old_id).is_none(), "stale id should not allow mutable access");
    assert!(reg.remove(old_id).is_none(), "stale id should not remove");

    let err = reg.remove_window(old_id).expect_err("stale id should error");
    assert!(matches!(err, RegistryError::InvalidWindowId(id) if id == old_id));

    assert!(reg.snapshot(new_id).is_some(), "new id should remain valid");
}
