//! Key-value store engine implementation.

use crate::store::compaction;
use crate::store::error::{Result, StoreError};
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct KVStore {
    pub base_dir: PathBuf,
    values: HashMap<String, Vec<u8>>,
}

impl KVStore {
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_dir = dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(StoreError::Io)?;
        }
        Ok(Self {
            base_dir,
            values: HashMap::new(),
        })
    }

    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.values.get(key).cloned())
    }

    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.values.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.values.remove(key);
        Ok(())
    }

    pub fn stats(&self) -> StoreStats {
        StoreStats {
            num_keys: self.values.len(),
            num_segments: 1,
            total_bytes: self.values.values().map(|v| v.len()).sum::<usize>() as u64,
            active_segment_id: 0,
            oldest_segment_id: 0,
        }
    }

    pub fn compact(&mut self) -> Result<()> {
        compaction::compact(self)
    }
}
