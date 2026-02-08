use std::collections::HashSet;

mod common;
use common::TestPtrs;

#[test]
fn ids_debug_format_includes_hex_pointer() {
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };

    let dk_dbg = format!("{dk:?}");
    let sk_dbg = format!("{sk:?}");

    let dk_hex = format!("{:#x}", p.ds as usize);
    let sk_hex = format!("{:#x}", p.s as usize);

    assert!(dk_dbg.starts_with("DesktopKey("));
    assert!(dk_dbg.contains(&dk_hex));

    assert!(sk_dbg.starts_with("SurfaceKey("));
    assert!(sk_dbg.contains(&sk_hex));
}

#[test]
fn ids_hash_and_eq_work_in_hashset() {
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };

    let mut dks = HashSet::new();
    dks.insert(dk1);
    dks.insert(dk2);
    assert_eq!(dks.len(), 2);

    let mut sks = HashSet::new();
    sks.insert(sk1);
    sks.insert(sk2);
    assert_eq!(sks.len(), 2);
}
