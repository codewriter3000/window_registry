use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use window_registry::{
    LifecycleState,
    Registry,
    RegistryError,
    RegistryEvent,
    SharedRegistry,
    WindowChange,
};

mod common;
use common::TestPtrs;

#[test]
fn shared_registry_lifecycle_invalid_id_propagates() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    reg.remove_window_with(id, |_| {})
        .expect("remove_window_with should succeed");

    let events: Arc<Mutex<Vec<RegistryEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let events_map = Arc::clone(&events);
    let err = reg.on_map_with(id, move |evs| {
        events_map.lock().unwrap().extend(evs);
    });
    assert!(matches!(err, Err(RegistryError::InvalidWindowId(stale)) if stale == id));
    assert!(events.lock().unwrap().is_empty());

    let events_unmap = Arc::clone(&events);
    let err = reg.on_unmap_with(id, move |evs| {
        events_unmap.lock().unwrap().extend(evs);
    });
    assert!(matches!(err, Err(RegistryError::InvalidWindowId(stale)) if stale == id));
    assert!(events.lock().unwrap().is_empty());
}

#[test]
fn shared_registry_on_map_with_dispatches_events() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    let events: Arc<Mutex<Vec<RegistryEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_map = Arc::clone(&events);

    reg.on_map_with(id, move |evs| {
        events_map.lock().unwrap().extend(evs);
    })
    .expect("on_map_with should succeed");

    let collected = events.lock().unwrap();
    assert_eq!(collected.len(), 1);
    assert!(collected.iter().any(|e| matches!(
        e,
        RegistryEvent::WindowChanged { id: ev_id, changes }
            if *ev_id == id
                && changes.lifecycle == Some(WindowChange { old: LifecycleState::Created, new: LifecycleState::Mapped })
    )));
}

#[test]
fn shared_registry_on_unmap_with_dispatches_events() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    reg.on_map_with(id, |_| {})
        .expect("on_map_with should succeed");

    let events: Arc<Mutex<Vec<RegistryEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_unmap = Arc::clone(&events);

    reg.on_unmap_with(id, move |evs| {
        events_unmap.lock().unwrap().extend(evs);
    })
    .expect("on_unmap_with should succeed");

    let collected = events.lock().unwrap();
    assert_eq!(collected.len(), 1);
    assert!(collected.iter().any(|e| matches!(
        e,
        RegistryEvent::WindowChanged { id: ev_id, changes }
            if *ev_id == id
                && changes.lifecycle == Some(WindowChange { old: LifecycleState::Mapped, new: LifecycleState::Unmapped })
    )));
}

#[test]
fn shared_registry_on_map_error_skips_dispatch() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    reg.remove_window_with(id, |_| {})
        .expect("remove_window_with should succeed");

    let called = Arc::new(AtomicBool::new(false));
    let called_flag = Arc::clone(&called);
    let err = reg.on_map_with(id, move |_| {
        called_flag.store(true, Ordering::SeqCst);
    });

    assert!(matches!(err, Err(RegistryError::InvalidWindowId(stale)) if stale == id));
    assert!(!called.load(Ordering::SeqCst));
}

#[test]
fn shared_registry_on_unmap_error_skips_dispatch() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    reg.remove_window_with(id, |_| {})
        .expect("remove_window_with should succeed");

    let called = Arc::new(AtomicBool::new(false));
    let called_flag = Arc::clone(&called);
    let err = reg.on_unmap_with(id, move |_| {
        called_flag.store(true, Ordering::SeqCst);
    });

    assert!(matches!(err, Err(RegistryError::InvalidWindowId(stale)) if stale == id));
    assert!(!called.load(Ordering::SeqCst));
}
