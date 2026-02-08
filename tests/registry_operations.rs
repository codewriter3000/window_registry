use window_registry::{
    LifecycleState,
    Registry,
    RegistryError,
    RegistryEvent,
    WindowChange,
    WindowState,
};

mod common;
use common::TestPtrs;

#[test]
fn registry_insert_lookup_and_snapshot() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, events) = reg.insert_window(dk, sk).expect("insert_window should succeed");

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowCreated { id: ev_id, dk: ev_dk, sk: ev_sk }
            if ev_id == id && ev_dk == dk && ev_sk == sk
    ));

    assert_eq!(reg.from_desktop(dk), Some(id));
    assert_eq!(reg.from_surface(sk), Some(id));

    let snapshot = reg.snapshot(id).expect("snapshot should be available");
    assert_eq!(snapshot.id, id);
    assert_eq!(snapshot.dk, dk);
    assert_eq!(snapshot.sk, sk);
    assert_eq!(snapshot.lifecycle, LifecycleState::Created);
    assert_eq!(snapshot.geometry, None);
    assert_eq!(snapshot.state, WindowState::default());
    assert_eq!(snapshot.is_focused, false);
    assert_eq!(snapshot.workspace, None);
    assert_eq!(snapshot.output, None);
    assert_eq!(snapshot.stack_index, 0);
    assert_eq!(snapshot.parent_id, None);
    assert!(snapshot.children.is_empty());
    assert_eq!(snapshot.title, None);
    assert_eq!(snapshot.app_id, None);

    let all = reg.snapshot_all();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, id);
}

#[test]
fn registry_duplicate_keys_error() {
    let mut reg = Registry::new();

    let p1 = TestPtrs::new();
    let (dk1, sk1) = unsafe { p1.keys() };
    let (id1, _events) = reg.insert_window(dk1, sk1).expect("insert_window A");

    let p2 = TestPtrs::new();
    let (dk2, sk2) = unsafe { p2.keys() };

    let err = reg.insert_window(dk1, sk2).expect_err("duplicate dk should fail");
    assert!(matches!(
        err,
        RegistryError::DesktopKeyAlreadyRegistered { dk, existing }
            if dk == dk1 && existing == id1
    ));

    let err = reg.insert_window(dk2, sk1).expect_err("duplicate sk should fail");
    assert!(matches!(
        err,
        RegistryError::SurfaceKeyAlreadyRegistered { sk, existing }
            if sk == sk1 && existing == id1
    ));
}

#[test]
fn registry_set_title_and_invalid_id() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");

    assert!(reg.set_title(id, "App Title".to_string()));
    let snapshot = reg.snapshot(id).expect("snapshot should exist");
    assert_eq!(snapshot.title.as_deref(), Some("App Title"));

    let (_record, _events) = reg.remove_window(id).expect("remove_window should succeed");
    assert!(!reg.set_title(id, "After Remove".to_string()));
}

#[test]
fn registry_lifecycle_transitions() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");

    let events = reg.on_map(id).expect("on_map should succeed");
    assert_eq!(events.len(), 1);
    assert!(events.iter().any(|e| matches!(
        e,
        RegistryEvent::WindowChanged { id: ev_id, changes }
            if *ev_id == id
                && changes.lifecycle == Some(WindowChange { old: LifecycleState::Created, new: LifecycleState::Mapped })
    )));

    let events = reg.on_map(id).expect("on_map idempotent");
    assert!(events.is_empty());

    let events = reg.on_unmap(id).expect("on_unmap should succeed");
    assert_eq!(events.len(), 1);
    assert!(events.iter().any(|e| matches!(
        e,
        RegistryEvent::WindowChanged { id: ev_id, changes }
            if *ev_id == id
                && changes.lifecycle == Some(WindowChange { old: LifecycleState::Mapped, new: LifecycleState::Unmapped })
    )));

    let events = reg.on_unmap(id).expect("on_unmap idempotent");
    assert!(events.is_empty());
}

#[test]
fn registry_invalid_id_errors() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");
    let (_record, _events) = reg.remove_window(id).expect("remove_window should succeed");

    let err = reg.on_map(id).expect_err("stale id should fail");
    assert!(matches!(err, RegistryError::InvalidWindowId(stale) if stale == id));

    let err = reg.on_unmap(id).expect_err("stale id should fail");
    assert!(matches!(err, RegistryError::InvalidWindowId(stale) if stale == id));

    let err = reg.remove_window(id).expect_err("stale id should fail");
    assert!(matches!(err, RegistryError::InvalidWindowId(stale) if stale == id));

    // Stale ids are treated as invalid.
}
