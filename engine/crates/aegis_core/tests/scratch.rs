//! TESTCLEAN1: a `Scratch` folder is unique per call and gone once dropped, a panicking test included.

use aegis_core::scratch::Scratch;

#[test]
fn a_scratch_folder_is_made_unique_and_removed_on_drop() {
    let (a, b) = (Scratch::new("scratch_self"), Scratch::new("scratch_self"));
    assert_ne!(a.path(), b.path(), "two calls with one tag: two folders");
    assert!(a.is_dir() && b.is_dir());
    std::fs::create_dir_all(a.join("deep/er")).unwrap();
    std::fs::write(a.join("deep/er/f.txt"), "x").unwrap();
    let (pa, pb) = (a.path_buf(), b.path_buf());
    drop(a);
    drop(b);
    assert!(!pa.exists() && !pb.exists(), "dropped: removed with its contents");
}

#[test]
fn a_vacant_scratch_is_not_made_but_still_removed() {
    let v = Scratch::vacant("scratch_vacant");
    assert!(!v.exists(), "the code under test makes it");
    std::fs::create_dir_all(v.join("sub")).unwrap();
    let p = v.path_buf();
    drop(v);
    assert!(!p.exists());
}

#[test]
fn a_folder_held_until_the_thread_ends_goes_with_the_thread() {
    let p = std::thread::spawn(|| {
        let p = Scratch::new("scratch_thread").until_thread_end();
        std::fs::write(p.join("f.txt"), "x").unwrap();
        assert!(p.is_dir(), "held while the thread runs");
        p
    })
    .join()
    .unwrap();
    assert!(!p.exists(), "the thread ended: removed");
}

#[test]
fn a_panicking_test_still_removes_its_folder() {
    let made = std::sync::Mutex::new(None);
    let r = std::panic::catch_unwind(|| {
        let s = Scratch::new("scratch_panic");
        std::fs::write(s.join("f.txt"), "x").unwrap();
        *made.lock().unwrap() = Some(s.path_buf());
        panic!("on purpose");
    });
    assert!(r.is_err());
    let p = made.lock().unwrap().clone().expect("the folder was made");
    assert!(!p.exists(), "unwinding dropped the guard");
}
