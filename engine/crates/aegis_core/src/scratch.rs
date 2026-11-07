//! Test scratch folders (TESTCLEAN1): `Scratch` makes a unique folder in the temp dir and removes it when dropped, a
//! failing (panicking) test included, so test runs leave nothing behind. Every test that needs a temp folder uses it;
//! a test that keeps its output on purpose writes to a fixed path instead and says so.

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A scratch folder `<temp>/aegis_<tag>_<pid>_<n>`, unique per process and per call; removed with everything in it
/// on drop. Derefs to its `Path`, so `scratch.join("a.json")` works; `path_buf()` for an owned copy.
#[derive(Debug)]
pub struct Scratch {
    path: PathBuf,
}

impl Scratch {
    /// A new, empty folder (created).
    pub fn new(tag: &str) -> Scratch {
        let s = Scratch::vacant(tag);
        std::fs::create_dir_all(&s.path).unwrap_or_else(|e| panic!("scratch folder {}: {e}", s.path.display()));
        s
    }

    /// A unique path not yet created, for code under test that makes the folder itself; still removed on drop.
    pub fn vacant(tag: &str) -> Scratch {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("aegis_{tag}_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Scratch { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_buf(&self) -> PathBuf {
        self.path.clone()
    }

    /// Hands the guard to the current thread and returns the path: the folder is removed when the thread ends (a
    /// panic included). For helpers whose result outlives them in a form that cannot carry the guard (a context
    /// struct, a path handed on); each test runs on its own thread, so that is the end of the test.
    pub fn until_thread_end(self) -> PathBuf {
        let path = self.path.clone();
        HELD.with(|h| h.borrow_mut().push(self));
        path
    }
}

thread_local! {
    static HELD: std::cell::RefCell<Vec<Scratch>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Deref for Scratch {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Windows can hold a just-closed file for a moment; a few short retries, then give up quietly.
        for _ in 0..5 {
            if std::fs::remove_dir_all(&self.path).is_ok() || !self.path.exists() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}
