//! Lint/Clippy/CI-friendly file utilities for mini-kvstore-v2 volumes (key->blob mapping).

use std::fs;
use std::path::{Path, PathBuf};

/// Ensures a directory exists, creating it if needed.
pub fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
    } else {
        Ok(())
    }
}

/// Maps a key to its blob path within the data directory.
/// Example: data/blobs/<sub>/<key>
pub fn key_to_blob_path(data_dir: &Path, key: &str) -> PathBuf {
    let sub = if key.len() >= 2 { &key[0..2] } else { "xx" };
    let blob_dir = data_dir.join("blobs").join(sub);
    let _ = ensure_dir_exists(&blob_dir); // Ignore error for demo simplicity
    blob_dir.join(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blob_path() {
        let path = key_to_blob_path(Path::new("/tmp/data"), "abcdef");
        assert!(path.to_str().unwrap().contains("ab"));
        assert!(path.to_str().unwrap().contains("abcdef"));
    }
}
