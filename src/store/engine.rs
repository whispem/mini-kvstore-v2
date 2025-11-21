//! Key-value store engine implementation.

use crate::store::error::{Result, StoreError};
use crate::store::compaction;
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct KVStore {
    // Directory where all segment files and metadata live.
    pub base_dir: PathBuf,
    // In-memory index
    index: HashMap<String, (usize, u64, u64)>,
}

impl KVStore {
    /// Open a KVStore at the given directory (creates it if missing)
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_dir = dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)
                .map_err(|e| StoreError::Io(e))?;
        }
        Ok(Self {
            base_dir,
            index: HashMap::new(),
        })
    }

    /// Returns the base directory for the store.
    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    /// Returns all the current keys.
    pub fn list_keys(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }

    /// Gets the value for a key.
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some((_segment, _offset, _len)) = self.index.get(key) {
        
            Ok(Some(Vec::new()))
        } else {
            Ok(None)
        }
    }

    /// Sets a key-value pair.
    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        // ... logique ajout ...
        Ok(())
    }

    /// Deletes a key.
    pub fn delete(&mut self, key: &str) -> Result<()> {
        // ... logique suppression ...
        Ok(())
    }

    /// Returns statistics for the store.
    pub fn stats(&self) -> StoreStats {
        StoreStats {
            num_keys: self.index.len(),
            num_segments: 1, 
            total_bytes: 1,  
            active_segment_id: 0,
            oldest_segment_id: 0,
        }
    }

    /// Manual compaction entrypoint.
    pub fn compact(&mut self) -> Result<()> {
        compaction::compact(self)
    }
}
