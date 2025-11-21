//! Key-value store engine implementation with simple file persistence for unit tests.

use crate::store::compaction;
use crate::store::error::{Result, StoreError};
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
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
        let mut values = HashMap::new();
        let file_path = base_dir.join("kv_store.bin");
        if file_path.exists() {
            let file = File::open(&file_path).map_err(StoreError::Io)?;
            let mut reader = BufReader::new(file);
            // Simple binary format: key_len, key_bytes, val_len, val_bytes for each entry
            loop {
                let mut buf4 = [0u8; 4];
                if reader.read_exact(&mut buf4).is_err() {
                    break;
                }
                let key_len = u32::from_le_bytes(buf4) as usize;
                let mut key_bytes = vec![0u8; key_len];
                reader.read_exact(&mut key_bytes)?;
                reader.read_exact(&mut buf4)?;
                let val_len = u32::from_le_bytes(buf4) as usize;
                let mut val_bytes = vec![0u8; val_len];
                reader.read_exact(&mut val_bytes)?;
                let key = String::from_utf8(key_bytes).map_err(|e| {
                    StoreError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })?;
                values.insert(key, val_bytes);
            }
        }
        Ok(Self { base_dir, values })
    }

    fn persist(&self) -> Result<()> {
        let file_path = self.base_dir.join("kv_store.bin");
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(StoreError::Io)?;
        let mut writer = BufWriter::new(file);
        for (key, value) in &self.values {
            let key_bytes = key.as_bytes();
            let val_bytes = value.as_slice();
            writer.write_all(&(key_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(key_bytes)?;
            writer.write_all(&(val_bytes.len() as u32).to_le_bytes())?;
            writer.write_all(val_bytes)?;
        }
        writer.flush()?;
        Ok(())
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
        self.persist()
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.values.remove(key);
        self.persist()
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
