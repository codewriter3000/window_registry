use window_registry::{Registry, SharedRegistry};

mod common;
use common::TestPtrs;

#[test]
fn shared_registry_allows_concurrent_reads() {
    let reg = SharedRegistry::new(Registry::new());
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_with(dk, sk, |_| {})
        .expect("insert_window_with should succeed");

    let reg_reader = reg.clone();
    let reader = std::thread::spawn(move || {
        let snap = reg_reader.snapshot(id).expect("snapshot should exist");
        assert_eq!(snap.id, id);
    });

    reader.join().expect("reader thread should finish");

    reg.remove_window_with(id, |_| {})
        .expect("remove_window_with should succeed");
}
