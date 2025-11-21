//! Manual log compaction logic.

use super::error::{Result, StoreError};
use crate::KVStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

/// Performs manual compaction:
/// - Rewrites only live key-value pairs to a new segment
/// - Updates index
/// - Deletes obsolete segments
pub fn compact(store: &mut KVStore) -> Result<()> {
    let volume_dir = store.volume_dir();
    let temp_dir = volume_dir.join("tmp_compaction");

    // Remove any previous temp directory
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to clean temp dir: {}", e))
        })?;
    }
    fs::create_dir_all(&temp_dir)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to create temp dir: {}", e)))?;

    // Collect all live data
    let mut live_data: Vec<(String, Vec<u8>)> = Vec::new();
    {
        let keys = store.list_keys();
        for key in keys {
            if let Some(value) = store.get(&key).unwrap_or(None) {
                live_data.push((key, value));
            }
        }
    }

    // Write live data to a new segment
    let seg_path = temp_dir.join("seg_compacted.dat");
    let mut seg_file = fs::File::create(&seg_path)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to create compacted segment: {}", e)))?;

    for (key, value) in &live_data {
        // Use KVStore's volume logic to serialize
        store.write_record(&mut seg_file, key, value)?;
    }

    // Prepare to swap: find all old segments
    let segment_files = segment_file_paths(&volume_dir)?;

    // Move compacted segment back to volume directory
    let new_segment_id = next_segment_id(&volume_dir)?;
    let new_segment_path = volume_dir.join(format!("segment-{:04}.dat", new_segment_id));
    fs::rename(&seg_path, &new_segment_path)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to move compacted segment: {}", e)))?;

    // Delete old segments
    for seg in &segment_files {
        fs::remove_file(seg)
            .map_err(|e| StoreError::CompactionFailed(format!("Failed to remove old segment: {}", e)))?;
    }

    // Clean up temp dir
    fs::remove_dir_all(&temp_dir)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to clean temp dir: {}", e)))?;

    // Rebuild index by reloading new segment
    store.reload_index()?;

    Ok(())
}

/// Returns paths to all segment files.
fn segment_file_paths(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut segments = Vec::new();
    for entry in fs::read_dir(dir)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to list segments: {}", e)))?
    {
        let entry = entry
            .map_err(|e| StoreError::CompactionFailed(format!("Failed to read segment entry: {}", e)))?;
        let path = entry.path();
        if path
            .file_name()
            .map(|n| n.to_string_lossy().starts_with("segment-") && n.to_string_lossy().ends_with(".dat"))
            .unwrap_or(false)
        {
            segments.push(path);
        }
    }
    Ok(segments)
}

/// Finds the next available segment ID in the volume dir.
fn next_segment_id(dir: &Path) -> Result<usize> {
    let mut ids = HashSet::new();
    for entry in fs::read_dir(dir)
        .map_err(|e| StoreError::CompactionFailed(format!("Failed to list segments: {}", e)))?
    {
        let entry = entry
            .map_err(|e| StoreError::CompactionFailed(format!("Failed to read segment entry: {}", e)))?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if name.starts_with("segment-") && name.ends_with(".dat") {
                if let Ok(id) = name["segment-".len()..name.len() - ".dat".len()].parse::<usize>() {
                    ids.insert(id);
                }
            }
        }
    }
    let next = ids.into_iter().max().map(|x| x + 1).unwrap_or(0);
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KVStore;

    #[test]
    fn test_compaction_removes_obsolete_segments() {
        let test_dir = PathBuf::from("tests_data/unit_compaction_remove");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");
        let mut store = KVStore::open(test_dir.to_str().unwrap()).unwrap();

        // Write multiple versions of keys
        for i in 0..3 {
            store.set("k", format!("v{}", i).as_bytes()).unwrap();
        }
        compact(&mut store).unwrap();

        let seg_files = segment_file_paths(&test_dir).unwrap();
        assert_eq!(seg_files.len(), 1);

        let _ = fs::remove_dir_all(&test_dir);
    }
}
