//! Manual log compaction logic.

use super::error::{Result, StoreError};
use crate::store::KVStore;
use std::fs;

/// Performs manual compaction.
pub fn compact(store: &mut KVStore) -> Result<()> {
    // Clear all segments
    let volume_dir = store.base_dir();
    let segments = find_all_segments(&volume_dir)?;

    for seg_path in segments {
        fs::remove_file(seg_path).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to remove old segment: {}", e))
        })?;
    }

    Ok(())
}

/// Finds all segment file paths in a directory
fn find_all_segments(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut segments = Vec::new();

    for entry in fs::read_dir(dir)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to read directory: {}", e)))?
    {
        let entry =
            entry.map_err(|e| StoreError::CompactionFailed(format!("Failed to read entry: {}", e)))?;
        let path = entry.path();

        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if name.starts_with("segment-") && name.ends_with(".dat") {
                segments.push(path);
            }
        }
    }

    Ok(segments)
}
