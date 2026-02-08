#![cfg(feature = "test-utils")]

use std::num::NonZeroU32;
use std::panic::{catch_unwind, AssertUnwindSafe};

use window_registry::{Registry, SharedRegistry, WindowId};

mod common;
use common::TestPtrs;

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
