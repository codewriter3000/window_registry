use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use window_registry::{DesktopKey, SurfaceKey, weston_desktop_surface, weston_surface};

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

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn ids_debug_format_includes_hex_pointer() {
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };

    let dk_fmt = format!("{:?}", dk);
    let sk_fmt = format!("{:?}", sk);

    assert!(dk_fmt.starts_with("DesktopKey(0x"));
    assert!(dk_fmt.ends_with(")"));

    assert!(sk_fmt.starts_with("SurfaceKey(0x"));
    assert!(sk_fmt.ends_with(")"));
}

#[test]
fn ids_hash_is_stable_for_same_pointer() {
    let p = TestPtrs::new();

    let (dk1, sk1) = unsafe { p.keys() };
    let (dk2, sk2) = unsafe { p.keys() };

    assert_eq!(dk1, dk2);
    assert_eq!(sk1, sk2);

    assert_eq!(hash_of(&dk1), hash_of(&dk2));
    assert_eq!(hash_of(&sk1), hash_of(&sk2));
}

#[test]
fn ids_as_ptr_round_trip() {
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };

    let dk_ptr = dk.as_ptr();
    let sk_ptr = sk.as_ptr();

    assert_eq!(dk_ptr as usize, p.ds as usize);
    assert_eq!(sk_ptr as usize, p.s as usize);
}
