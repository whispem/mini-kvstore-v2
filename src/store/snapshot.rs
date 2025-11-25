//! Index snapshot persistence for faster restarts.

use crate::store::error::{Result, StoreError};
use crate::store::index::Index;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const SNAPSHOT_MAGIC: &[u8; 8] = b"KVINDEX1";

/// Save index to snapshot file
pub fn save_snapshot(index: &Index, path: &Path) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write magic header
    writer.write_all(SNAPSHOT_MAGIC)?;

    // Write number of entries
    let num_entries = index.len() as u64;
    writer.write_all(&num_entries.to_le_bytes())?;

    // Write each entry
    for key in index.keys() {
        if let Some(entry) = index.get(key) {
            // Key length + key
            let key_bytes = key.as_bytes();
            let key_len = key_bytes.len() as u32;
            writer.write_all(&key_len.to_le_bytes())?;
            writer.write_all(key_bytes)?;

            // Entry data
            writer.write_all(&entry.segment_id.to_le_bytes())?;
            writer.write_all(&entry.offset.to_le_bytes())?;
        }
    }

    writer.flush()?;
    Ok(())
}

/// Load index from snapshot file
pub fn load_snapshot(path: &Path) -> Result<Index> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and verify magic
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != SNAPSHOT_MAGIC {
        return Err(StoreError::Corrupted("Invalid snapshot magic".into()));
    }

    // Read number of entries
    let mut num_entries_bytes = [0u8; 8];
    reader.read_exact(&mut num_entries_bytes)?;
    let num_entries = u64::from_le_bytes(num_entries_bytes);

    let mut index = Index::new();

    // Read each entry
    for _ in 0..num_entries {
        // Read key
        let mut key_len_bytes = [0u8; 4];
        reader.read_exact(&mut key_len_bytes)?;
        let key_len = u32::from_le_bytes(key_len_bytes) as usize;

        let mut key_bytes = vec![0u8; key_len];
        reader.read_exact(&mut key_bytes)?;
        let key = String::from_utf8(key_bytes)
            .map_err(|_| StoreError::Corrupted("Invalid UTF-8 in snapshot".into()))?;

        // Read entry data
        let mut segment_id_bytes = [0u8; 8];
        reader.read_exact(&mut segment_id_bytes)?;
        let segment_id = u64::from_le_bytes(segment_id_bytes);

        let mut offset_bytes = [0u8; 8];
        reader.read_exact(&mut offset_bytes)?;
        let offset = u64::from_le_bytes(offset_bytes);

        index.insert(key, segment_id, offset);
    }

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_roundtrip() {
        let mut index = Index::new();
        index.insert("key1".to_string(), 0, 100);
        index.insert("key2".to_string(), 1, 200);
        index.insert("key3".to_string(), 1, 300);

        let snapshot_path = "tests_data/test_snapshot.idx";
        let _ = std::fs::create_dir_all("tests_data");

        // Save
        save_snapshot(&index, Path::new(snapshot_path)).unwrap();

        // Load
        let loaded = load_snapshot(Path::new(snapshot_path)).unwrap();

        assert_eq!(loaded.len(), 3);
        assert!(loaded.contains("key1"));
        assert!(loaded.contains("key2"));
        assert!(loaded.contains("key3"));

        let entry1 = loaded.get("key1").unwrap();
        assert_eq!(entry1.segment_id, 0);
        assert_eq!(entry1.offset, 100);

        let _ = std::fs::remove_file(snapshot_path);
    }

    #[test]
    fn test_empty_snapshot() {
        let index = Index::new();
        let snapshot_path = "tests_data/test_empty_snapshot.idx";
        let _ = std::fs::create_dir_all("tests_data");

        save_snapshot(&index, Path::new(snapshot_path)).unwrap();
        let loaded = load_snapshot(Path::new(snapshot_path)).unwrap();

        assert_eq!(loaded.len(), 0);
        assert!(loaded.is_empty());

        let _ = std::fs::remove_file(snapshot_path);
    }
}
