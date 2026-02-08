use std::sync::{Arc, RwLock};

use window_registry::{
    Registry,
    DesktopKey,
    SurfaceKey,
    weston_desktop_surface,
    weston_surface,
};

#[test]
fn generation_prevents_stale_ids_multithreaded() {
    // Shared registry (Option B)
    let reg = Arc::new(RwLock::new(Registry::new()));

    // Fake but stable heap allocations to simulate C pointers
    let ds_a = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s_a  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    // Insert window A (write lock)
    let a = {
        let mut r = reg.write().expect("registry lock poisoned");
        unsafe {
            r.insert_window(
                DesktopKey::from_ptr(ds_a),
                SurfaceKey::from_ptr(s_a),
            )
        }
    };

    // Remove window A (write lock)
    {
        let mut r = reg.write().expect("registry lock poisoned");
        r.remove_window(a);
    }

    // Create a second, different "window"
    let ds_b = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s_b  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    // Insert window B (write lock)
    let b = {
        let mut r = reg.write().expect("registry lock poisoned");
        unsafe {
            r.insert_window(
                DesktopKey::from_ptr(ds_b),
                SurfaceKey::from_ptr(s_b),
            )
        }
    };

    // Reads should work from any thread. We'll check on a separate thread to
    // exercise the Send/Sync requirements of the shared registry.
    let reg2 = Arc::clone(&reg);
    let handle = std::thread::spawn(move || {
        let r = reg2.read().expect("registry lock poisoned");

        // Stale ID must not resolve
        assert!(r.get(a).is_none());

        // New ID must resolve
        assert!(r.get(b).is_some());
    });

    handle.join().expect("reader thread panicked");

    // Cleanup heap allocations
    unsafe {
        drop(Box::from_raw(ds_a));
        drop(Box::from_raw(s_a));
        drop(Box::from_raw(ds_b));
        drop(Box::from_raw(s_b));
    }
}

