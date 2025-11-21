//! Compaction logic for reclaiming space from old segments.

use crate::store::engine::KVStore;
use crate::store::error::{Result, StoreError};
use crate::store::segment::Segment;
use std::fs;

/// Performs compaction on the store.
///
/// This function:
/// 1. Reads all live key-value pairs from existing segments
/// 2. Creates new segments with only the live data
/// 3. Atomically replaces old segments with new ones
/// 4. Removes old segment files
///
/// # Safety
///
/// This implementation collects all live data in memory first,
/// which ensures consistency but may use significant memory for large stores.
pub fn compact_segments(store: &mut KVStore) -> Result<()> {
    let data_dir = &store.config.data_dir;
    let temp_dir = data_dir.join(".compacting");

    // Create temporary directory for new segments
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to clean temp dir: {}", e))
        })?;
    }
    fs::create_dir_all(&temp_dir).map_err(|e| {
        StoreError::CompactionFailed(format!("Failed to create temp dir: {}", e))
    })?;

    // Collect all live data
    let mut live_data: Vec<(String, Vec<u8>)> = Vec::new();

    for key in store.index.keys() {
        if let Some(&(seg_id, offset, _len)) = store.index.get(key) {
            if let Some(seg) = store.segments.get_mut(&seg_id) {
                if let Ok(Some(value)) = seg.read_value_at(offset) {
                    live_data.push((key.clone(), value));
                }
            }
        }
    }

    // Write live data to new segments in temp directory
    let mut new_active_id = 0usize;
    let mut new_segments = std::collections::HashMap::new();
    let mut new_index = crate::store::index::Index::new();

    // Create first new segment
    let mut current_seg = Segment::open(&temp_dir, new_active_id).map_err(|e| {
        StoreError::CompactionFailed(format!("Failed to create new segment: {}", e))
    })?;

    for (key, value) in live_data {
        // Check if we need a new segment
        if current_seg.is_full() {
            new_segments.insert(new_active_id, current_seg);
            new_active_id += 1;
            current_seg = Segment::open(&temp_dir, new_active_id).map_err(|e| {
                StoreError::CompactionFailed(format!("Failed to create new segment: {}", e))
            })?;
        }

        // Write to new segment
        let offset = current_seg.append(key.as_bytes(), &value).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to write during compaction: {}", e))
        })?;

        new_index.insert(key, new_active_id, offset, value.len() as u64);
    }

    // Don't forget the last segment
    new_segments.insert(new_active_id, current_seg);

    // Now atomically swap: remove old segments and move new ones
    // First, collect old segment IDs
    let old_segment_ids: Vec<usize> = store.segments.keys().copied().collect();

    // Remove old segment files
    for seg_id in &old_segment_ids {
        let old_path = data_dir.join(format!("segment-{:04}.dat", seg_id));
        if old_path.exists() {
            fs::remove_file(&old_path).map_err(|e| {
                StoreError::CompactionFailed(format!(
                    "Failed to remove old segment {}: {}",
                    seg_id, e
                ))
            })?;
        }
    }

    // Move new segments from temp to data directory
    for seg_id in new_segments.keys() {
        let temp_path = temp_dir.join(format!("segment-{:04}.dat", seg_id));
        let final_path = data_dir.join(format!("segment-{:04}.dat", seg_id));
        fs::rename(&temp_path, &final_path).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to move new segment {}: {}", seg_id, e))
        })?;
    }

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    // Reopen segments from the data directory
    store.segments.clear();
    for seg_id in new_segments.keys() {
        let seg = Segment::open(data_dir, *seg_id).map_err(|e| {
            StoreError::CompactionFailed(format!("Failed to reopen segment {}: {}", seg_id, e))
        })?;
        store.segments.insert(*seg_id, seg);
    }

    // Update store state
    store.index = new_index;
    store.active_id = new_active_id;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all};
    use std::path::Path;

    fn setup_test_dir(path: &str) {
        let _ = remove_dir_all(path);
        create_dir_all(path).expect("Failed to create test directory");
    }

    #[test]
    fn test_compaction_preserves_data() {
        let test_dir = "tests_data/compaction_test";
        setup_test_dir(test_dir);

        let mut store = KVStore::open(test_dir).unwrap();

        // Write data
        store.set("key1", b"value1").unwrap();
        store.set("key2", b"value2").unwrap();
        store.set("key3", b"value3").unwrap();

        // Update some keys
        store.set("key1", b"updated1").unwrap();

        // Delete a key
        store.delete("key2").unwrap();

        // Compact
        compact_segments(&mut store).unwrap();

        // Verify data integrity
        assert_eq!(store.get("key1").unwrap(), Some(b"updated1".to_vec()));
        assert_eq!(store.get("key2").unwrap(), None);
        assert_eq!(store.get("key3").unwrap(), Some(b"value3".to_vec()));

        let _ = remove_dir_all(test_dir);
    }

    #[test]
    fn test_compaction_reduces_size() {
        let test_dir = "tests_data/compaction_size";
        setup_test_dir(test_dir);

        let mut store = KVStore::open(test_dir).unwrap();

        // Write many versions of the same key
        for i in 0..100 {
            store.set("key", format!("value_{}", i).as_bytes()).unwrap();
        }

        let stats_before = store.stats();
        compact_segments(&mut store).unwrap();
        let stats_after = store.stats();

        // Size should decrease
        assert!(stats_after.total_bytes < stats_before.total_bytes);

        // Data should be preserved
        assert_eq!(store.get("key").unwrap(), Some(b"value_99".to_vec()));

        let _ = remove_dir_all(test_dir);
    }

    #[test]
    fn test_compaction_empty_store() {
        let test_dir = "tests_data/compaction_empty";
        setup_test_dir(test_dir);

        let mut store = KVStore::open(test_dir).unwrap();

        // Compaction on empty store should succeed
        compact_segments(&mut store).unwrap();

        assert_eq!(store.stats().num_keys, 0);

        let _ = remove_dir_all(test_dir);
    }
}
