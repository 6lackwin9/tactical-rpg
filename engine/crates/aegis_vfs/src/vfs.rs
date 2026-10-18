use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid virtual path: {0}")]
    InvalidPath(String),

    #[error("File not found: {0}")]
    NotFound(String),
}

/// Virtual File System mapping `res://` and `user://` paths.
#[derive(Debug, Clone)]
pub struct Vfs {
    res_root: PathBuf,
    user_root: PathBuf,
}

impl Vfs {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(res_root: P1, user_root: P2) -> Self {
        Self {
            res_root: res_root.as_ref().to_path_buf(),
            user_root: user_root.as_ref().to_path_buf(),
        }
    }

    /// Resolves a virtual path like `res://data/catalog/skills.json` or `user://saves/slot_1.json`
    /// to an absolute physical filesystem path.
    pub fn resolve(&self, vpath: &str) -> Result<PathBuf, VfsError> {
        if let Some(stripped) = vpath.strip_prefix("res://") {
            Ok(self.res_root.join(stripped.replace('/', std::path::MAIN_SEPARATOR_STR)))
        } else if let Some(stripped) = vpath.strip_prefix("user://") {
            Ok(self.user_root.join(stripped.replace('/', std::path::MAIN_SEPARATOR_STR)))
        } else {
            // Treat as relative to res_root if no scheme
            Ok(self.res_root.join(vpath.replace('/', std::path::MAIN_SEPARATOR_STR)))
        }
    }

    pub fn file_exists(&self, vpath: &str) -> bool {
        if let Ok(path) = self.resolve(vpath) {
            path.is_file()
        } else {
            false
        }
    }

    pub fn dir_exists(&self, vpath: &str) -> bool {
        if let Ok(path) = self.resolve(vpath) {
            path.is_dir()
        } else {
            false
        }
    }

    pub fn read_to_string(&self, vpath: &str) -> Result<String, VfsError> {
        let path = self.resolve(vpath)?;
        if !path.exists() {
            return Err(VfsError::NotFound(vpath.to_string()));
        }
        Ok(fs::read_to_string(path)?)
    }

    pub fn read_bytes(&self, vpath: &str) -> Result<Vec<u8>, VfsError> {
        let path = self.resolve(vpath)?;
        if !path.exists() {
            return Err(VfsError::NotFound(vpath.to_string()));
        }
        Ok(fs::read(path)?)
    }

    pub fn write_string(&self, vpath: &str, content: &str) -> Result<(), VfsError> {
        let path = self.resolve(vpath)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn create_dir_all(&self, vpath: &str) -> Result<(), VfsError> {
        let path = self.resolve(vpath)?;
        fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn remove_file(&self, vpath: &str) -> Result<(), VfsError> {
        let path = self.resolve(vpath)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn list_dir(&self, vpath: &str) -> Result<Vec<String>, VfsError> {
        let path = self.resolve(vpath)?;
        if !path.exists() {
            return Err(VfsError::NotFound(vpath.to_string()));
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        entries.sort();
        Ok(entries)
    }

    /// Read and parse a JSON file directly into serde_json::Value
    pub fn read_json(&self, vpath: &str) -> Result<serde_json::Value, VfsError> {
        let content = self.read_to_string(vpath)?;
        let val: serde_json::Value = serde_json::from_str(&content)?;
        Ok(val)
    }

    /// Serialize and write a JSON value directly to a file
    pub fn write_json<T: serde::Serialize>(&self, vpath: &str, value: &T) -> Result<(), VfsError> {
        let json_str = serde_json::to_string_pretty(value)?;
        self.write_string(vpath, &json_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_path_resolution() {
        let vfs = Vfs::new("C:/Project", "C:/Project/saves");
        assert_eq!(
            vfs.resolve("res://data/items.json").unwrap(),
            PathBuf::from("C:/Project/data/items.json")
        );
        assert_eq!(
            vfs.resolve("user://slot_1.json").unwrap(),
            PathBuf::from("C:/Project/saves/slot_1.json")
        );
    }
}
