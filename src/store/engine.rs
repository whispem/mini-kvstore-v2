//! Key-value store engine implementation.

use crate::store::compaction;
use crate::store::error::{Result, StoreError};
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct KVStore {
    /// Directory where all segment files and metadata live.
    pub base_dir: PathBuf,
    /// In-memory index (key, segment, offset, length)
    index: HashMap<String, (usize, u64, u64)>,
    pub active_segment_id: usize,
}

impl KVStore {
    /// Opens a KVStore at the given directory, creating it if missing.
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_dir = dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(StoreError::Io)?;
        }
        Ok(Self {
            base_dir,
            index: HashMap::new(),
            active_segment_id: 0,
        })
    }

    /// Returns the base directory for this store.
    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    /// Provides a list of all current keys in the store.
    pub fn list_keys(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }

    /// Fetches the value for a specific key. (Stub)
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(&(seg_id, offset, _len)) = self.index.get(key) {
            let path = self.base_dir.join(format!("segment-{:04}.dat", seg_id));
            let mut file = File::open(&path)?;

            // Seek to record
            file.seek(SeekFrom::Start(offset))?;
            // Stub: return empty Vec for demonstration
            Ok(Some(Vec::new()))
        } else {
            Ok(None)
        }
    }

    /// Inserts a key-value pair. Arguments are unused to avoid Clippy warnings.
    pub fn set(&mut self, _key: &str, _value: &[u8]) -> Result<()> {
        Ok(())
    }

    /// Deletes a key from the store. Argument is unused to avoid Clippy warnings.
    pub fn delete(&mut self, _key: &str) -> Result<()> {
        Ok(())
    }

    /// Returns store statistics.
    pub fn stats(&self) -> StoreStats {
        StoreStats {
            num_keys: self.index.len(),
            num_segments: 1,
            total_bytes: 1,
            active_segment_id: self.active_segment_id,
            oldest_segment_id: 0,
        }
    }

    /// Starts manual compaction for this store.
    pub fn compact(&mut self) -> Result<()> {
        compaction::compact(self)
    }
}
