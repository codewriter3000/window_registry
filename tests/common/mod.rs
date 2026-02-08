use std::collections::{HashMap, HashSet};

use window_registry::{
    DesktopKey,
    LifecycleState,
    SharedRegistry,
    SurfaceKey,
    WindowId,
    weston_desktop_surface,
    weston_surface,
};

pub struct TestPtrs {
    pub ds: *mut weston_desktop_surface,
    pub s: *mut weston_surface,
}

impl TestPtrs {
    pub fn new() -> Self {
        let ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
        let s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;
        Self { ds, s }
    }

    pub unsafe fn keys(&self) -> (DesktopKey, SurfaceKey) {
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

/// Invariants that should hold no matter what your higher-level policy is.
#[allow(dead_code)]
pub fn assert_shared_registry_hard_invariants(reg: &SharedRegistry) {
    let all = reg.snapshot_all();

    // H1: snapshot_all() WindowIds are unique
    let mut ids = HashSet::<WindowId>::with_capacity(all.len());
    for w in &all {
        assert!(
            ids.insert(w.id),
            "duplicate WindowId in snapshot_all: {:?}",
            w.id
        );
    }

    // H2: round-trip: every id in snapshot_all must resolve via snapshot(id)
    for w in &all {
        let rt = reg.snapshot(w.id).unwrap_or_else(|| {
            panic!(
                "snapshot({:?}) returned None but id exists in snapshot_all()",
                w.id
            )
        });
        assert_eq!(rt.id, w.id, "snapshot(id) returned mismatched id");
        assert_eq!(rt.dk, w.dk, "snapshot(id) dk mismatch for {:?}", w.id);
        assert_eq!(rt.sk, w.sk, "snapshot(id) sk mismatch for {:?}", w.id);
        assert_eq!(rt.lifecycle, w.lifecycle, "snapshot(id) lifecycle mismatch for {:?}", w.id);
        assert_eq!(rt.title, w.title, "snapshot(id) title mismatch for {:?}", w.id);
        assert_eq!(rt.app_id, w.app_id, "snapshot(id) app_id mismatch for {:?}", w.id);
    }

    // H3: No live window should be Destroyed if your registry removes records on destroy.
    // If you *do* intend to keep destroyed records, delete this or move it to policy invariants.
    for w in &all {
        assert_ne!(
            w.lifecycle,
            LifecycleState::Destroyed,
            "snapshot_all contains Destroyed window (unexpected for remove-on-destroy model): {:?}",
            w.id
        );
    }
}

/// Optional invariants: enable these only if your model requires uniqueness of pointers.
///
/// Many window registries enforce: one (dk, sk) pair == one window.
#[allow(dead_code)]
pub fn assert_shared_registry_pointer_uniqueness(reg: &SharedRegistry) {
    let all = reg.snapshot_all();

    // P1: SurfaceKey is unique among live windows
    let mut seen_sk = HashMap::<SurfaceKey, WindowId>::new();
    for w in &all {
        if let Some(prev) = seen_sk.insert(w.sk, w.id) {
            panic!(
                "SurfaceKey reused by two live windows: sk={:?} prev_id={:?} new_id={:?}",
                w.sk, prev, w.id
            );
        }
    }

    // P2: DesktopKey is unique among live windows (if that’s your intended rule)
    let mut seen_dk = HashMap::<DesktopKey, WindowId>::new();
    for w in &all {
        if let Some(prev) = seen_dk.insert(w.dk, w.id) {
            panic!(
                "DesktopKey reused by two live windows: dk={:?} prev_id={:?} new_id={:?}",
                w.dk, prev, w.id
            );
        }
    }

    // P3: (dk, sk) pair unique (often redundant if sk unique)
    let mut seen_pair = HashMap::<(DesktopKey, SurfaceKey), WindowId>::new();
    for w in &all {
        if let Some(prev) = seen_pair.insert((w.dk, w.sk), w.id) {
            panic!(
                "(dk, sk) pair reused by two live windows: dk={:?} sk={:?} prev_id={:?} new_id={:?}",
                w.dk, w.sk, prev, w.id
            );
        }
    }
}

#[allow(dead_code)]
pub fn assert_shared_registry_invariants(reg: &SharedRegistry) {
    assert_shared_registry_hard_invariants(&reg);
    assert_shared_registry_pointer_uniqueness(&reg);
}