use std::ptr;

use window_registry::{
    on_new_desktop_surface,
    DesktopKey,
    Registry,
    SurfaceKey,
    weston_desktop_surface,
    weston_surface,
};

static mut NEXT_SURFACE: *mut weston_surface = ptr::null_mut();

#[no_mangle]
pub extern "C" fn weston_desktop_surface_get_surface(
    _ds: *mut weston_desktop_surface,
) -> *mut weston_surface {
    unsafe { NEXT_SURFACE }
}

#[test]
fn on_new_desktop_surface_inserts_window() {
    let mut reg = Registry::new();

    let ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    unsafe {
        NEXT_SURFACE = s;
        let _ = on_new_desktop_surface(ds, &mut reg);
    }

    let dk = unsafe { DesktopKey::from_ptr(ds) };
    let sk = unsafe { SurfaceKey::from_ptr(s) };

    let id = reg.from_desktop(dk).expect("desktop key should be registered");
    assert_eq!(reg.from_surface(sk), Some(id));

    let snap = reg.snapshot(id).expect("snapshot should exist");
    assert_eq!(snap.dk, dk);
    assert_eq!(snap.sk, sk);

    unsafe {
        drop(Box::from_raw(ds));
        drop(Box::from_raw(s));
    }
}
