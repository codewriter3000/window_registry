use window_registry::{
    Registry,
    DesktopKey,
    SurfaceKey,
    weston_desktop_surface,
    weston_surface
};

#[test]
fn generation_prevents_stale_ids() {
    let mut r = Registry::new();

    // Create fake but stable heap allocations to simulate C pointers
    let ds_a = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s_a  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    let a = unsafe {
        r.insert_window(
            DesktopKey::from_ptr(ds_a),
            SurfaceKey::from_ptr(s_a),
            None, // view
        )
    };

    // Remove window A
    r.remove_window(a);

    // Create a second, different "window"
    let ds_b = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s_b  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;


    let b = unsafe {
        r.insert_window(
            DesktopKey::from_ptr(ds_b),
            SurfaceKey::from_ptr(s_b),
            None,
        )
    };

    // Stale ID must not resolve
    assert!(r.get(a).is_none());

    // New ID must resolve
    assert!(r.get(b).is_some());

    // Cleanup heap allocations
    unsafe {
        drop(Box::from_raw(ds_a));
        drop(Box::from_raw(s_a));
        drop(Box::from_raw(ds_b));
        drop(Box::from_raw(s_b));
    }
}

