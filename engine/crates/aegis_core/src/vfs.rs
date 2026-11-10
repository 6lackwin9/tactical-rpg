//! The engine's one read seam (WEBREL1-E1b, `docs/specs/WEBREL1.md` 2.1): every runtime data, asset and font read
//! goes through `read`, `read_to_string`, `exists`, `is_file`, `is_dir` and `list_dir`. Native: `std::fs` itself
//! (same paths, same errors; `list_dir` returns the entries in `read_dir`'s order, so a caller that sorts still
//! sorts). Web (`wasm32`): the mounted `Table`, an in-memory path -> bytes map with derived folders, which the web
//! start fills from the fetched bundle before the first frame (`mount`, WEBREL1-G1). Writes are not part of the seam:
//! user files go through `aegis_save`'s store, content writers are native tooling, and the dev features that are off
//! on the web (playtest recordings, screenshots, dev tools writing assets) use `dev`. The lint in `tests/fs_lint.rs`
//! refuses `std::fs` and std path probes elsewhere in non-test, non-CLI `src`.

use std::collections::BTreeMap;
use std::io;
use std::path::{Component, Path, PathBuf};

/// An in-memory file tree: normalised path -> bytes. Folders are derived (a folder exists while a file is under it),
/// listings are sorted by name. Paths are normalised by component (`.` dropped, `..` pops, `\` and `/` both split on
/// the web), so `a/./b/../c.json` and `a/c.json` are one file.
#[derive(Debug, Default, Clone)]
pub struct Table {
    files: BTreeMap<String, Vec<u8>>,
}

/// `path` as a table key: its components joined by `/` (a leading `/` kept for an absolute path), `.` dropped, `..`
/// popping the previous component.
pub fn key(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut absolute = false;
    for c in path.components() {
        match c {
            Component::RootDir => absolute = true,
            Component::Prefix(p) => parts.push(p.as_os_str().to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(s) => parts.extend(s.to_string_lossy().split('\\').filter(|p| !p.is_empty()).map(str::to_owned)),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, format!("{}: not in the mounted bundle", path.display()))
}

impl Table {
    pub const fn new() -> Table {
        Table { files: BTreeMap::new() }
    }

    /// Adds or replaces the file at `path`.
    pub fn insert(&mut self, path: impl AsRef<Path>, bytes: Vec<u8>) {
        self.files.insert(key(path.as_ref()), bytes);
    }

    /// Adds every file of `other` (a later bundle part); a path in both takes `other`'s bytes.
    pub fn extend(&mut self, other: Table) {
        self.files.extend(other.files);
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Every file as (key, bytes), in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.files.iter().map(|(k, v)| (k.as_str(), v.as_slice()))
    }

    /// The table as one bundle part in the pack format (`PACK_MAGIC`): the magic, the file count, then per file (in key
    /// order) the key's length, the key (UTF-8, `/`-separated), the data's length and the data; every number a
    /// little-endian u32.
    pub fn to_pack(&self) -> Vec<u8> {
        let size: usize = self.files.iter().map(|(k, v)| 8 + k.len() + v.len()).sum();
        let mut out = Vec::with_capacity(PACK_MAGIC.len() + 4 + size);
        out.extend_from_slice(PACK_MAGIC);
        out.extend_from_slice(&(self.files.len() as u32).to_le_bytes());
        for (k, v) in &self.files {
            out.extend_from_slice(&(k.len() as u32).to_le_bytes());
            out.extend_from_slice(k.as_bytes());
            out.extend_from_slice(&(v.len() as u32).to_le_bytes());
            out.extend_from_slice(v);
        }
        out
    }

    /// A bundle part in the pack format (`to_pack`) as a table; the problem when the bytes are not one (another magic,
    /// a cut-off file, a key that is not UTF-8, bytes after the last file).
    pub fn from_pack(bytes: &[u8]) -> Result<Table, String> {
        let mut at = 0usize;
        let mut take = |n: usize, what: &str| -> Result<&[u8], String> {
            let end = at.checked_add(n).filter(|e| *e <= bytes.len()).ok_or_else(|| format!("the pack is cut off in {what} at byte {at}"))?;
            let s = &bytes[at..end];
            at = end;
            Ok(s)
        };
        if take(PACK_MAGIC.len(), "the magic")? != PACK_MAGIC {
            return Err("not a bundle pack (no AEGISPK1 magic)".into());
        }
        let num = |b: &[u8]| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize;
        let count = num(take(4, "the file count")?);
        let mut t = Table::new();
        for i in 0..count {
            let klen = num(take(4, "a key length")?);
            let k = std::str::from_utf8(take(klen, "a key")?).map_err(|_| format!("file {i}: the key is not UTF-8"))?.to_string();
            let dlen = num(take(4, "a data length")?);
            let data = take(dlen, &k)?.to_vec();
            t.insert(&k, data);
        }
        if at != bytes.len() {
            return Err(format!("{} byte(s) after the last of {count} files", bytes.len() - at));
        }
        Ok(t)
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// The bytes of the file at `path`, or `NotFound`.
    pub fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files.get(&key(path)).cloned().ok_or_else(|| not_found(path))
    }

    /// The file at `path` as UTF-8, or `NotFound` / `InvalidData` (like `std::fs::read_to_string`).
    pub fn read_to_string(&self, path: &Path) -> io::Result<String> {
        String::from_utf8(self.read(path)?).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8"))
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&key(path))
    }

    /// Whether a file lies under `path` (the root, an empty key, is a folder while the table has any file).
    pub fn is_dir(&self, path: &Path) -> bool {
        let k = key(path);
        if k.is_empty() {
            return !self.files.is_empty();
        }
        let prefix = if k.ends_with('/') { k } else { format!("{k}/") };
        self.files.range(prefix.clone()..).next().is_some_and(|(f, _)| f.starts_with(&prefix))
    }

    pub fn exists(&self, path: &Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    /// The direct entries (files and folders) of the folder `path`, as `path.join(name)`, sorted by name; `NotFound`
    /// when no file lies under it.
    pub fn list_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let k = key(path);
        let prefix = if k.is_empty() || k.ends_with('/') { k } else { format!("{k}/") };
        let mut names: Vec<&str> = Vec::new();
        for f in self.files.range(prefix.clone()..).map(|(f, _)| f).take_while(|f| f.starts_with(&prefix)) {
            let name = f[prefix.len()..].split('/').next().unwrap_or_default();
            if !name.is_empty() && names.last() != Some(&name) {
                names.push(name);
            }
        }
        if names.is_empty() {
            return Err(not_found(path));
        }
        Ok(names.into_iter().map(|n| path.join(n)).collect())
    }
}

/// The first bytes of a bundle part (`Table::to_pack`, WEBREL1-G1): the format's name and version.
pub const PACK_MAGIC: &[u8; 8] = b"AEGISPK1";

#[cfg(target_arch = "wasm32")]
static MOUNTED: std::sync::RwLock<Table> = std::sync::RwLock::new(Table::new());

/// How many tables were mounted so far (`generation`).
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Mounts `table` as the web file tree (adding to what is mounted: a later bundle part extends it) and counts one
/// generation. Native reads never look at it (they read the disk), so on native this only counts; the web start calls
/// it once per fetched bundle part (the first before the first frame, a later one, such as the audio, while the game
/// runs).
pub fn mount(table: Table) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut m = MOUNTED.write().unwrap_or_else(|e| e.into_inner());
        m.extend(table);
    }
    #[cfg(not(target_arch = "wasm32"))]
    drop(table);
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// The number of `mount` calls so far: a reader that remembers a missing file (a cache) forgets it when this changes,
/// since a later bundle part may have brought the file.
pub fn generation() -> u64 {
    GENERATION.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(target_arch = "wasm32")]
fn with<R>(f: impl FnOnce(&Table) -> R) -> R {
    f(&MOUNTED.read().unwrap_or_else(|e| e.into_inner()))
}

/// The bytes of the file at `path` (`std::fs::read` on native).
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    #[cfg(not(target_arch = "wasm32"))]
    return std::fs::read(path);
    #[cfg(target_arch = "wasm32")]
    with(|t| t.read(path.as_ref()))
}

/// The file at `path` as UTF-8 (`std::fs::read_to_string` on native).
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    #[cfg(not(target_arch = "wasm32"))]
    return std::fs::read_to_string(path);
    #[cfg(target_arch = "wasm32")]
    with(|t| t.read_to_string(path.as_ref()))
}

/// `Path::exists` on native.
pub fn exists(path: impl AsRef<Path>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return path.as_ref().exists();
    #[cfg(target_arch = "wasm32")]
    with(|t| t.exists(path.as_ref()))
}

/// `Path::is_file` on native.
pub fn is_file(path: impl AsRef<Path>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return path.as_ref().is_file();
    #[cfg(target_arch = "wasm32")]
    with(|t| t.is_file(path.as_ref()))
}

/// `Path::is_dir` on native.
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return path.as_ref().is_dir();
    #[cfg(target_arch = "wasm32")]
    with(|t| t.is_dir(path.as_ref()))
}

/// The entries of the folder `path` as full paths (`DirEntry::path`). Native: `std::fs::read_dir` in its own order,
/// unreadable entries skipped (as `read_dir(..).flatten()`); sort when the order matters. Web: sorted by name.
pub fn list_dir(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    #[cfg(not(target_arch = "wasm32"))]
    return Ok(std::fs::read_dir(path)?.flatten().map(|e| e.path()).collect());
    #[cfg(target_arch = "wasm32")]
    with(|t| t.list_dir(path.as_ref()))
}

/// Dev files (WEBREL1 6.1): the file work of features that are off in the web build: playtest recordings, F12 and note
/// screenshots, dev tools writing assets back. Native: `std::fs` itself. Web: every call fails with `Unsupported`
/// (`OFF_ON_WEB`), probes are false and listings fail with `Unsupported` (callers default to empty), so a dev
/// feature that slips through reports instead of panicking. Runtime reads of data and assets never come here
/// (`vfs::read`), nor do saves (`aegis_save`'s store).
pub mod dev {
    use std::io;
    use std::path::{Path, PathBuf};

    /// Whether dev files work in this build (false on the web).
    pub const ON: bool = !cfg!(target_arch = "wasm32");

    /// The reason a dev file call fails on the web.
    pub const OFF_ON_WEB: &str = "off in the web build (no dev files)";

    #[cfg(target_arch = "wasm32")]
    fn off<T>() -> io::Result<T> {
        Err(io::Error::new(io::ErrorKind::Unsupported, OFF_ON_WEB))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
        std::fs::write(path, bytes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<u64> {
        std::fs::copy(from, to)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    /// The entries of `path` as full paths, in `read_dir`'s order (unreadable entries skipped).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_dir(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
        Ok(std::fs::read_dir(path)?.flatten().map(|e| e.path()).collect())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn exists(path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_file(path: impl AsRef<Path>) -> bool {
        path.as_ref().is_file()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_dir(path: impl AsRef<Path>) -> bool {
        path.as_ref().is_dir()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn read_to_string(_: impl AsRef<Path>) -> io::Result<String> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write(_: impl AsRef<Path>, _: impl AsRef<[u8]>) -> io::Result<()> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn rename(_: impl AsRef<Path>, _: impl AsRef<Path>) -> io::Result<()> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn copy(_: impl AsRef<Path>, _: impl AsRef<Path>) -> io::Result<u64> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_dir_all(_: impl AsRef<Path>) -> io::Result<()> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn remove_file(_: impl AsRef<Path>) -> io::Result<()> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn list_dir(_: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
        off()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn exists(_: impl AsRef<Path>) -> bool {
        false
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_file(_: impl AsRef<Path>) -> bool {
        false
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_dir(_: impl AsRef<Path>) -> bool {
        false
    }
}
