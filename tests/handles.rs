use std::ptr::NonNull;

use window_registry::{CompositorHandles, Registry, weston_view};

mod common;
use common::TestPtrs;

#[test]
fn compositor_handles_set_get_remove() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let (id, _events) = reg.insert_window(dk, sk).expect("insert_window should succeed");

    let view = Box::into_raw(Box::new(0u8)) as *mut weston_view;

    let mut handles = CompositorHandles::new();
    handles.set_view(id, view);

    let stored = handles.get_view(id).expect("view should be set");
    let expected = NonNull::new(view).unwrap();
    assert_eq!(stored, expected);

    handles.remove_view(id);
    assert!(handles.get_view(id).is_none());

    unsafe {
        drop(Box::from_raw(view));
    }
}
