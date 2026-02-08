use std::num::NonZeroU32;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use window_registry::{Registry, RegistryError, RegistryEvent, SharedRegistry, WindowId};

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
            // Ensure the registry is unlocked by taking a read snapshot inside dispatch.
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
fn shared_registry_allows_concurrent_reads() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    let reg_reader = reg.clone();
    let reader = std::thread::spawn(move || {
        let snap = reg_reader.snapshot(id).expect("snapshot should exist");
        assert_eq!(snap.id, id);
    });

    reader.join().expect("reader thread should finish");

    reg.remove_window_with(id, |_| {})
        .expect("remove_window_with should succeed");
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
    assert_eq!(collected.len(), 2);
    assert!(collected.iter().any(|e| matches!(
        e,
        RegistryEvent::WindowMapped { id: ev_id } if *ev_id == id
    )));
    assert!(collected.iter().any(|e| matches!(
        e,
        RegistryEvent::LifecycleChanged { id: ev_id, old, new }
            if *ev_id == id
                && *old == window_registry::LifecycleState::Created
                && *new == window_registry::LifecycleState::Mapped
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
    assert_eq!(collected.len(), 2);
    assert!(collected.iter().any(|e| matches!(
        e,
        RegistryEvent::WindowUnmapped { id: ev_id } if *ev_id == id
    )));
    assert!(collected.iter().any(|e| matches!(
        e,
        RegistryEvent::LifecycleChanged { id: ev_id, old, new }
            if *ev_id == id
                && *old == window_registry::LifecycleState::Mapped
                && *new == window_registry::LifecycleState::Unmapped
    )));
}

#[test]
fn shared_registry_insert_window_error_propagates() {
    let reg = SharedRegistry::new(Registry::new());
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let _id1 = reg
        .insert_window_with(dk1, sk1, |_| {})
        .expect("insert_window_with should succeed");

    let (dk2, sk2) = unsafe { p2.keys() };
    let err = reg.insert_window_with(dk1, sk2, |_| {});
    assert!(matches!(
        err,
        Err(RegistryError::DesktopKeyAlreadyRegistered { dk, .. }) if dk == dk1
    ));

    let err = reg.insert_window_with(dk2, sk1, |_| {});
    assert!(matches!(
        err,
        Err(RegistryError::SurfaceKeyAlreadyRegistered { sk, .. }) if sk == sk1
    ));
}

#[test]
fn shared_registry_remove_window_error_propagates() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    reg.remove_window_with(id, |_| {})
        .expect("remove_window_with should succeed");

    let err = reg.remove_window_with(id, |_| {});
    assert!(matches!(err, Err(RegistryError::InvalidWindowId(stale)) if stale == id));
}

#[test]
fn shared_registry_insert_window_error_skips_dispatch() {
    let reg = SharedRegistry::new(Registry::new());
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let _id = reg
        .insert_window_with(dk1, sk1, |_| {})
        .expect("insert_window_with should succeed");

    let (dk2, sk2) = unsafe { p2.keys() };
    let called = Arc::new(AtomicBool::new(false));
    let called_flag = Arc::clone(&called);

    let err = reg.insert_window_with(dk1, sk2, move |_| {
        called_flag.store(true, Ordering::SeqCst);
    });

    assert!(matches!(
        err,
        Err(RegistryError::DesktopKeyAlreadyRegistered { dk, .. }) if dk == dk1
    ));
    assert!(!called.load(Ordering::SeqCst));

    let called_flag = Arc::clone(&called);
    let err = reg.insert_window_with(dk2, sk1, move |_| {
        called_flag.store(true, Ordering::SeqCst);
    });

    assert!(matches!(
        err,
        Err(RegistryError::SurfaceKeyAlreadyRegistered { sk, .. }) if sk == sk1
    ));
    assert!(!called.load(Ordering::SeqCst));
}

#[test]
fn shared_registry_remove_window_error_skips_dispatch() {
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
    let err = reg.remove_window_with(id, move |_| {
        called_flag.store(true, Ordering::SeqCst);
    });

    assert!(matches!(err, Err(RegistryError::InvalidWindowId(stale)) if stale == id));
    assert!(!called.load(Ordering::SeqCst));
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

#[cfg(feature = "test-utils")]
#[test]
fn shared_registry_poisoned_write_lock_panics() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };

    reg.poison_for_test();

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = reg.insert_window_with(dk, sk, |_| {});
    }));
    assert!(result.is_err());

    let bogus_id = WindowId::new_for_test(0, NonZeroU32::new(1).unwrap());

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = reg.remove_window_with(bogus_id, |_| {});
    }));
    assert!(result.is_err());

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = reg.on_map_with(bogus_id, |_| {});
    }));
    assert!(result.is_err());

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = reg.on_unmap_with(bogus_id, |_| {});
    }));
    assert!(result.is_err());
}
