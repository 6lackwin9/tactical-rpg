//! The user store behind `aegis_save` (WEBREL1-E1b, `docs/specs/WEBREL1.md` 2.2): every user file (saves, slot locks,
//! keyed saves, settings) is read and written through a `Store`, so the save API (`user_dir`, slots, settings) is the
//! same on every platform. Native: `Files`, the user folder on disk (`std::fs`, unchanged). Web: `Keys` over the
//! browser's `localStorage` (`LocalStorage`), one key per file, `aegis/<game id>/<relative path>` (`key_for`); folders
//! are derived from the keys. A write the backend refuses (the browser's quota) is an `io::Error`, which the save API
//! turns into its usual problem strings ("Can't write ..."): never a panic. The bytes written are the same everywhere.
//!
//! `with_store` runs code on another store (tests and tools: an in-memory `Keys<MemKeys>` with a byte quota).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Where user files live, as the save API needs it. Paths are the ones the API builds under `user_dir`.
pub trait Store {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    /// Moves `from` over `to` (replacing it); on failure `to` is as it was.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;
    /// The entries of the folder `path` as full paths (files and folders).
    fn list_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;

    fn exists(&self, path: &Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    /// Copies the file `from` to `to`.
    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        let bytes = self.read(from)?;
        self.write(to, &bytes)
    }

    /// Whether one `write` replaces a file whole or not at all (WEBREL1 6.2: `localStorage.setItem` is atomic), so the
    /// save API writes the file directly instead of a temp file and a rename, and a save needs only its own size in the
    /// quota. False on disk, where a write cut short leaves half a file.
    fn writes_whole(&self) -> bool {
        false
    }

    /// The file at `path` as UTF-8.
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        String::from_utf8(self.read(path)?).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8"))
    }
}

/// The native store: the user folder on disk, `std::fs` as it always was.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct Files;

#[cfg(not(target_arch = "wasm32"))]
impl Store for Files {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        std::fs::write(path, bytes)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn list_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        Ok(std::fs::read_dir(path)?.flatten().map(|e| e.path()).collect())
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::copy(from, to).map(|_| ())
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }
}

/// The first path component of every web store key: the engine's user folder on the web (`crate::app_dir`).
pub const WEB_ROOT: &str = "aegis";

/// The `localStorage` key of the user file at `path`: its components joined by `/` (`.` dropped, `..` popping),
/// which for a path under `user_dir(<game id>)` is `aegis/<game id>/<relative path>`. None for a path outside
/// `WEB_ROOT` (the store refuses it).
pub fn key_for(path: &Path) -> Option<String> {
    let key = aegis_core::vfs::key(path);
    (key.split('/').next() == Some(WEB_ROOT) && key.len() > WEB_ROOT.len()).then_some(key)
}

/// A string key-value backend (the browser's `localStorage`, or `MemKeys`).
pub trait KeyBackend {
    fn get(&self, key: &str) -> Option<String>;
    /// Sets `key`; Err with the backend's reason when it refuses (a full quota).
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
    fn remove(&self, key: &str);
    fn keys(&self) -> Vec<String>;
}

/// The web store: one key per user file (`key_for`), the file's text as the value (user files are UTF-8 JSON).
#[derive(Debug, Default, Clone)]
pub struct Keys<B> {
    pub backend: B,
}

fn outside(path: &Path) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, format!("{}: outside the user store ({WEB_ROOT}/...)", path.display()))
}

fn missing(path: &Path) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("{}: no such user file", path.display()))
}

impl<B: KeyBackend> Keys<B> {
    pub fn new(backend: B) -> Self {
        Keys { backend }
    }

    fn folder_prefix(path: &Path) -> Option<String> {
        Some(format!("{}/", key_for(path).or_else(|| (aegis_core::vfs::key(path) == WEB_ROOT).then(|| WEB_ROOT.to_string()))?))
    }
}

impl<B: KeyBackend> Store for Keys<B> {
    /// One `set` replaces the value whole or refuses and changes nothing.
    fn writes_whole(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let key = key_for(path).ok_or_else(|| outside(path))?;
        self.backend.get(&key).map(String::into_bytes).ok_or_else(|| missing(path))
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let key = key_for(path).ok_or_else(|| outside(path))?;
        let text = std::str::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("{}: the web store keeps text only", path.display())))?;
        self.backend.set(&key, text).map_err(|why| io::Error::new(io::ErrorKind::StorageFull, format!("{}: {why}", path.display())))
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let bytes = self.read(from)?;
        self.write(to, &bytes)?;
        self.remove_file(from)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let key = key_for(path).ok_or_else(|| outside(path))?;
        if self.backend.get(&key).is_none() {
            return Err(missing(path));
        }
        self.backend.remove(&key);
        Ok(())
    }

    /// Folders are derived from the keys: nothing to make (a path outside the store is still refused).
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        Self::folder_prefix(path).map(|_| ()).ok_or_else(|| outside(path))
    }

    fn is_file(&self, path: &Path) -> bool {
        key_for(path).is_some_and(|k| self.backend.get(&k).is_some())
    }

    fn is_dir(&self, path: &Path) -> bool {
        Self::folder_prefix(path).is_some_and(|p| self.backend.keys().iter().any(|k| k.starts_with(&p)))
    }

    /// Sorted by name.
    fn list_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let prefix = Self::folder_prefix(path).ok_or_else(|| outside(path))?;
        let mut names: Vec<String> = self.backend.keys().iter().filter_map(|k| k.strip_prefix(&prefix)).filter_map(|rest| rest.split('/').next()).filter(|n| !n.is_empty()).map(str::to_owned).collect();
        names.sort();
        names.dedup();
        if names.is_empty() {
            return Err(missing(path));
        }
        Ok(names.into_iter().map(|n| path.join(n)).collect())
    }
}

/// An in-memory `KeyBackend` with a byte quota (the sum of every key's and value's length, as `localStorage` counts
/// its quota over keys and values): the web store's stand-in for native tests and tools. A `set` past the quota is
/// refused and changes nothing.
#[derive(Debug, Default)]
pub struct MemKeys {
    pub map: RefCell<BTreeMap<String, String>>,
    /// The most bytes the keys and values may hold (0: no limit).
    pub quota: usize,
}

impl MemKeys {
    pub fn with_quota(quota: usize) -> Self {
        MemKeys { map: RefCell::default(), quota }
    }

    /// The bytes the keys and values hold now.
    pub fn used(&self) -> usize {
        self.map.borrow().iter().map(|(k, v)| k.len() + v.len()).sum()
    }
}

impl KeyBackend for MemKeys {
    fn get(&self, key: &str) -> Option<String> {
        self.map.borrow().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let old = self.map.borrow().get(key).map_or(0, |v| key.len() + v.len());
        let after = self.used() - old + key.len() + value.len();
        if self.quota > 0 && after > self.quota {
            return Err(format!("the storage quota ({} bytes) is full", self.quota));
        }
        self.map.borrow_mut().insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove(&self, key: &str) {
        self.map.borrow_mut().remove(key);
    }

    fn keys(&self) -> Vec<String> {
        self.map.borrow().keys().cloned().collect()
    }
}

/// The browser's `localStorage` (the web build's backend). A missing `localStorage` (a sandboxed page) reads nothing
/// and refuses every write.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalStorage;

#[cfg(target_arch = "wasm32")]
impl LocalStorage {
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }
}

#[cfg(target_arch = "wasm32")]
impl KeyBackend for LocalStorage {
    fn get(&self, key: &str) -> Option<String> {
        Self::storage()?.get_item(key).ok()?
    }

    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        let storage = Self::storage().ok_or("this page has no localStorage")?;
        // A full quota is a QuotaExceededError (a JS exception, returned as Err), never a panic.
        storage.set_item(key, value).map_err(|e| e.as_string().unwrap_or_else(|| "the browser refused the write (storage quota)".into()))
    }

    fn remove(&self, key: &str) {
        if let Some(s) = Self::storage() {
            let _ = s.remove_item(key);
        }
    }

    fn keys(&self) -> Vec<String> {
        let Some(s) = Self::storage() else { return Vec::new() };
        (0..s.length().unwrap_or(0)).filter_map(|i| s.key(i).ok().flatten()).collect()
    }
}

thread_local! {
    static OVERRIDE: RefCell<Option<Rc<dyn Store>>> = const { RefCell::new(None) };
}

/// Runs `f` with `store` as this thread's user store (tests and tools); the platform store is back afterwards, also
/// when `f` panics.
pub fn with_store<R>(store: Rc<dyn Store>, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<Rc<dyn Store>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let prev = self.0.take();
            OVERRIDE.with(|o| *o.borrow_mut() = prev);
        }
    }
    let _restore = Restore(OVERRIDE.with(|o| o.borrow_mut().replace(store)));
    f()
}

/// The user store in use: `with_store`'s, else the platform's (`Files` on native, `Keys<LocalStorage>` on the web).
pub fn current() -> Rc<dyn Store> {
    if let Some(s) = OVERRIDE.with(|o| o.borrow().clone()) {
        return s;
    }
    platform()
}

#[cfg(not(target_arch = "wasm32"))]
fn platform() -> Rc<dyn Store> {
    Rc::new(Files)
}

#[cfg(target_arch = "wasm32")]
fn platform() -> Rc<dyn Store> {
    Rc::new(Keys::new(LocalStorage))
}
