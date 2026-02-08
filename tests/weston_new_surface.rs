use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

use window_registry::{
    on_new_desktop_surface,
    on_new_desktop_surface_with_keys,
    DesktopKey,
    Registry,
    RegistryError,
    SurfaceKey,
    weston_desktop_surface,
    weston_surface,
};

static SURFACE_PTR: AtomicPtr<weston_surface> = AtomicPtr::new(ptr::null_mut());

#[no_mangle]
pub extern "C" fn weston_desktop_surface_get_surface(
    _ds: *mut weston_desktop_surface,
) -> *mut weston_surface {
    SURFACE_PTR.load(Ordering::SeqCst)
}

struct TestPtrs {
    ds: *mut weston_desktop_surface,
    s: *mut weston_surface,
}

impl TestPtrs {
    fn new() -> Self {
        let ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
        let s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;
        Self { ds, s }
    }

    unsafe fn keys(&self) -> (DesktopKey, SurfaceKey) {
        (DesktopKey::from_ptr(self.ds), SurfaceKey::from_ptr(self.s))
    }
}

impl Drop for TestPtrs {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ds));
            drop(Box::from_raw(self.s));
        }
    }
}

#[test]
fn on_new_desktop_surface_with_keys_errors_on_duplicate() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };

    on_new_desktop_surface_with_keys(dk1, sk1, &mut reg)
        .expect("first insert should succeed");

    let err = on_new_desktop_surface_with_keys(dk1, sk2, &mut reg)
        .expect_err("duplicate desktop key should error");
    assert!(matches!(
        err,
        RegistryError::DesktopKeyAlreadyRegistered { dk, .. } if dk == dk1
    ));

    let err = on_new_desktop_surface_with_keys(dk2, sk1, &mut reg)
        .expect_err("duplicate surface key should error");
    assert!(matches!(
        err,
        RegistryError::SurfaceKeyAlreadyRegistered { sk, .. } if sk == sk1
    ));
}

#[test]
fn on_new_desktop_surface_propagates_error() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    SURFACE_PTR.store(p.s, Ordering::SeqCst);

    unsafe {
        on_new_desktop_surface(p.ds, &mut reg).expect("first insert should succeed");
        let err = on_new_desktop_surface(p.ds, &mut reg)
            .expect_err("duplicate desktop key should error");
        assert!(matches!(
            err,
            RegistryError::DesktopKeyAlreadyRegistered { dk, .. }
                if dk == DesktopKey::from_ptr(p.ds)
        ));
    }
}
