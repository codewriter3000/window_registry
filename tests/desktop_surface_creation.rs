use window_registry::{DesktopKey, SurfaceKey, weston_desktop_surface, weston_surface};

#[test]
fn keys_hash_and_eq_by_address() {
    // Two distinct allocations => two distinct addresses
    let ds1 = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let ds2 = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s1  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;
    let s2  = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    unsafe {
        // DesktopKey: equality by address
        let dk1 = DesktopKey::from_ptr(ds1);
        let dk2 = DesktopKey::from_ptr(ds2);
        assert_ne!(dk1, dk2);

        let dk1_again = DesktopKey::from_ptr(ds1);
        assert_eq!(dk1, dk1_again);

        // SurfaceKey: equality by address
        let sk1 = SurfaceKey::from_ptr(s1);
        let sk2 = SurfaceKey::from_ptr(s2);
        assert_ne!(sk1, sk2);

        let sk1_again = SurfaceKey::from_ptr(s1);
        assert_eq!(sk1, sk1_again);
    }

    // Cleanup heap allocations
    unsafe {
        drop(Box::from_raw(ds1));
        drop(Box::from_raw(ds2));
        drop(Box::from_raw(s1));
        drop(Box::from_raw(s2));
    }
}

