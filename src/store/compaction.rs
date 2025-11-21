//! Manual log compaction logic.

use super::error::{Result, StoreError};
use crate::store::KVStore;
use std::fs;

/// Performs manual compaction.
/// Collects all live keys, clears the store, and rewrites them.
pub fn compact(store: &mut KVStore) -> Result<()> {
    // Collect all live key-value pairs
    let keys: Vec<String> = store.list_keys();
    let mut live_data: Vec<(String, Vec<u8>)> = Vec::new();

    for key in keys {
        if let Some(value) = store.get(&key)? {
            live_data.push((key, value));
        }
    }

    // Clear all segments
    let volume_dir = store.base_dir();
    let segments = find_all_segments(&volume_dir)?;
    
    for seg_path in segments {
        fs::remove_file(seg_path).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to remove old segment: {}", e))
        })?;
    }

    // Reopen the store (this will create a fresh segment-0000.dat)
    *store = KVStore::open(&volume_dir)?;

    // Rewrite all live data
    for (key, value) in live_data {
        store.set(&key, &value)?;
    }

    Ok(())
}

/// Finds all segment file paths in a directory
fn find_all_segments(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut segments = Vec::new();
    
    for entry in fs::read_dir(dir).map_err(|e| {
        StoreError::CompactionFailed(format!("Failed to read directory: {}", e))
    })? {
        let entry = entry.map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to read entry: {}", e))
        })?;
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
