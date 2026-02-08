use std::sync::{Arc, Mutex};

use window_registry::{
    SharedRegistry,
    Registry,
    DesktopKey,
    SurfaceKey,
    weston_desktop_surface,
    weston_surface,
    LifecycleState,
    RegistryEvent,
};

mod common;
use common::{assert_shared_registry_invariants};

#[test]
fn registry_ids_deliverable_c_generation_lifecycle_and_events() {
    let reg = SharedRegistry::new(Registry::new());

    // We'll capture events emitted by insert/remove without needing a full event bus.
    let collected: Arc<Mutex<Vec<RegistryEvent>>> = Arc::new(Mutex::new(Vec::new()));

    // Create fake but stable heap allocations to simulate C pointers
    let ds_a = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s_a  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    // Insert window A
    let events_a = Arc::clone(&collected);
    let a = unsafe {
        reg.insert_window_with(
            DesktopKey::from_ptr(ds_a),
            SurfaceKey::from_ptr(s_a),
            move |evs| events_a.lock().unwrap().extend(evs),
        )
    }
    .expect("insert_window_with(A) should succeed");
    
    assert_shared_registry_invariants(&reg);

    // Deliverable C: lifecycle starts at Created
    let info_a = reg.snapshot(a).expect("A should be readable after insert");
    assert_eq!(info_a.lifecycle, LifecycleState::Created);

    // Deliverable C: WindowCreated event emitted for A
    {
        let evs = collected.lock().unwrap();
        assert!(
            evs.iter().any(|e| matches!(e, RegistryEvent::WindowCreated { id, .. } if *id == a)),
            "expected WindowCreated event for A"
        );
    }

    // Remove window A
    let events_rm_a = Arc::clone(&collected);
    reg.remove_window_with(a, move |evs| events_rm_a.lock().unwrap().extend(evs))
        .expect("remove_window_with(A) should succeed");

    assert_shared_registry_invariants(&reg); 

    // Create a second, different "window"
    let ds_b = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s_b  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    // Insert window B
    let events_b = Arc::clone(&collected);
    let b = unsafe {
        reg.insert_window_with(
            DesktopKey::from_ptr(ds_b),
            SurfaceKey::from_ptr(s_b),
            move |evs| events_b.lock().unwrap().extend(evs),
        )
    }
    .expect("insert_window_with(B) should succeed");

    assert_shared_registry_invariants(&reg); 

    // Stale ID must not resolve (generation check)
    assert!(reg.snapshot(a).is_none(), "stale WindowId A must not resolve");

    // New ID must resolve
    assert!(reg.snapshot(b).is_some(), "new WindowId B must resolve");

    // Deliverable C: WindowDestroyed emitted for A, WindowCreated for B
    {
        let evs = collected.lock().unwrap();
        assert!(
            evs.iter().any(|e| matches!(e, RegistryEvent::WindowDestroyed { id } if *id == a)),
            "expected WindowDestroyed event for A"
        );
        assert!(
            evs.iter().any(|e| matches!(e, RegistryEvent::WindowCreated { id, .. } if *id == b)),
            "expected WindowCreated event for B"
        );
    }

    // Exercise cross-thread reads (Option B shape)
    let reg2 = reg.clone();
    let t = std::thread::spawn(move || {
        assert!(reg2.snapshot(a).is_none());
        assert!(reg2.snapshot(b).is_some());
    });
    t.join().expect("reader thread panicked");

    // Cleanup heap allocations
    unsafe {
        drop(Box::from_raw(ds_a));
        drop(Box::from_raw(s_a));
        drop(Box::from_raw(ds_b));
        drop(Box::from_raw(s_b));
    }
}

