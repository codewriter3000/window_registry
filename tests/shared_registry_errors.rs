use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use window_registry::{Registry, RegistryError, SharedRegistry};

mod common;
use common::TestPtrs;

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
