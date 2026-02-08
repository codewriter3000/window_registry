#![cfg(feature = "test-utils")]

use std::num::NonZeroU32;

use window_registry::{Registry, RegistryError, WindowId};

fn invalid_index_id() -> WindowId {
    WindowId::new_for_test(9999, NonZeroU32::new(1).unwrap())
}

#[test]
fn get_returns_none_for_out_of_bounds_index() {
    let reg = Registry::new();
    assert!(reg.get(invalid_index_id()).is_none());
}

#[test]
fn get_mut_returns_none_for_out_of_bounds_index() {
    let mut reg = Registry::new();
    assert!(reg.get_mut(invalid_index_id()).is_none());
}

#[test]
fn remove_returns_none_for_out_of_bounds_index() {
    let mut reg = Registry::new();
    assert!(reg.remove(invalid_index_id()).is_none());
}

#[test]
fn remove_window_errors_for_out_of_bounds_index() {
    let mut reg = Registry::new();
    let err = reg.remove_window(invalid_index_id()).expect_err("invalid id should error");
    assert!(matches!(err, RegistryError::InvalidWindowId(_)));
}
