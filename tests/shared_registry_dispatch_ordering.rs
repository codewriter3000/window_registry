use std::sync::{Arc, Mutex};

use window_registry::{Registry, RegistryEvent, SharedRegistry};

mod common;
use common::TestPtrs;

#[test]
fn shared_registry_dispatch_ordering_and_unlocking() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let events: Arc<Mutex<Vec<RegistryEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let events_insert = Arc::clone(&events);
    let reg_insert = reg.clone();
    let id = reg
        .insert_window_with(dk, sk, move |evs| {
            let _snapshot = reg_insert.snapshot_all();
            events_insert.lock().unwrap().extend(evs);
        })
        .expect("insert_window_with should succeed");

    let events_remove = Arc::clone(&events);
    let reg_remove = reg.clone();
    reg.remove_window_with(id, move |evs| {
        let _snapshot = reg_remove.snapshot_all();
        events_remove.lock().unwrap().extend(evs);
    })
    .expect("remove_window_with should succeed");

    let collected = events.lock().unwrap();
    assert!(matches!(
        collected.get(0),
        Some(RegistryEvent::WindowCreated { id: ev_id, .. }) if *ev_id == id
    ));
    assert!(matches!(
        collected.get(1),
        Some(RegistryEvent::WindowDestroyed { id: ev_id }) if *ev_id == id
    ));
}

#[test]
fn shared_registry_event_ordering_multiple_windows() {
    let reg = SharedRegistry::new(Registry::new());
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };

    let events: Arc<Mutex<Vec<RegistryEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let events_insert_a = Arc::clone(&events);
    let reg_insert_a = reg.clone();
    let id_a = reg
        .insert_window_with(dk1, sk1, move |evs| {
            let _snapshot = reg_insert_a.snapshot_all();
            events_insert_a.lock().unwrap().extend(evs);
        })
        .expect("insert_window_with A should succeed");

    let events_insert_b = Arc::clone(&events);
    let reg_insert_b = reg.clone();
    let id_b = reg
        .insert_window_with(dk2, sk2, move |evs| {
            let _snapshot = reg_insert_b.snapshot_all();
            events_insert_b.lock().unwrap().extend(evs);
        })
        .expect("insert_window_with B should succeed");

    let events_remove_b = Arc::clone(&events);
    let reg_remove_b = reg.clone();
    reg.remove_window_with(id_b, move |evs| {
        let _snapshot = reg_remove_b.snapshot_all();
        events_remove_b.lock().unwrap().extend(evs);
    })
    .expect("remove_window_with B should succeed");

    let events_remove_a = Arc::clone(&events);
    let reg_remove_a = reg.clone();
    reg.remove_window_with(id_a, move |evs| {
        let _snapshot = reg_remove_a.snapshot_all();
        events_remove_a.lock().unwrap().extend(evs);
    })
    .expect("remove_window_with A should succeed");

    let collected = events.lock().unwrap();
    assert_eq!(collected.len(), 4);
    assert!(matches!(
        collected.get(0),
        Some(RegistryEvent::WindowCreated { id, .. }) if *id == id_a
    ));
    assert!(matches!(
        collected.get(1),
        Some(RegistryEvent::WindowCreated { id, .. }) if *id == id_b
    ));
    assert!(matches!(
        collected.get(2),
        Some(RegistryEvent::WindowDestroyed { id }) if *id == id_b
    ));
    assert!(matches!(
        collected.get(3),
        Some(RegistryEvent::WindowDestroyed { id }) if *id == id_a
    ));
}
