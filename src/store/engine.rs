// mini-kvstore-v2/src/store/engine.rs
use crate::store::error::{Result, StoreError};
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

const SEGMENT_PREFIX: &str = "segment-";
const SEGMENT_SUFFIX: &str = ".dat";

#[derive(Debug)]
pub struct KVStore {
    pub base_dir: PathBuf,
    values: HashMap<String, Vec<u8>>,

    // segment bookkeeping
    active_segment_id: u64,
    active_writer: Option<BufWriter<File>>,
}

impl KVStore {
    /// Open the store and replay all segment files to rebuild in-memory index.
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_dir = dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(StoreError::Io)?;
        }

        // 1) find existing segment files
        let mut segment_paths: Vec<(u64, PathBuf)> = Vec::new();
        for entry in fs::read_dir(&base_dir)
            .map_err(|e| StoreError::Io(std::io::Error::other(format!("read_dir: {}", e))))?
        {
            let entry = entry.map_err(|e| {
                StoreError::Io(std::io::Error::other(format!("read_dir entry: {}", e)))
            })?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(SEGMENT_PREFIX) && name.ends_with(SEGMENT_SUFFIX) {
                    // parse id
                    let id_str = &name[SEGMENT_PREFIX.len()..name.len() - SEGMENT_SUFFIX.len()];
                    if let Ok(id) = id_str.parse::<u64>() {
                        segment_paths.push((id, path));
                    }
                }
            }
        }

        // sort ascending by id
        segment_paths.sort_by_key(|(id, _)| *id);

        // 2) replay segments
        let mut values: HashMap<String, Vec<u8>> = HashMap::new();
        for (_id, path) in &segment_paths {
            Self::replay_segment(path, &mut values)?;
        }

        // 3) determine next segment id and open active segment for append
        let active_segment_id = segment_paths.last().map(|(id, _)| *id).unwrap_or(0);
        let next_id = active_segment_id + 1;
        let active_path = base_dir.join(format!("{}{}{}", SEGMENT_PREFIX, next_id, SEGMENT_SUFFIX));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&active_path)
            .map_err(StoreError::Io)?;
        let writer = BufWriter::new(file);

        Ok(Self {
            base_dir,
            values,
            active_segment_id: next_id,
            active_writer: Some(writer),
        })
    }

    /// Replay a single segment file into the provided values map.
    fn replay_segment(path: &Path, values: &mut HashMap<String, Vec<u8>>) -> Result<()> {
        let file = File::open(path).map_err(|e| {
            StoreError::CorruptedData(format!("Failed to open segment {}: {}", path.display(), e))
        })?;
        let mut reader = BufReader::new(file);

        loop {
            // Read opcode (1 byte)
            let mut op_buf = [0u8; 1];
            if reader.read_exact(&mut op_buf).is_err() {
                // EOF -> done
                break;
            }
            let op = op_buf[0];

            // Read key length (u32 LE)
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf).map_err(|e| {
                StoreError::CorruptedData(format!(
                    "Failed to read key length in {}: {}",
                    path.display(),
                    e
                ))
            })?;
            let key_len = u32::from_le_bytes(len_buf) as usize;

            // Read key bytes
            let mut key_bytes = vec![0u8; key_len];
            reader.read_exact(&mut key_bytes).map_err(|e| {
                StoreError::CorruptedData(format!(
                    "Failed to read key in {}: {}",
                    path.display(),
                    e
                ))
            })?;
            let key = String::from_utf8(key_bytes).map_err(|e| {
                StoreError::CorruptedData(format!("Invalid UTF-8 key in {}: {}", path.display(), e))
            })?;

            match op {
                0 => {
                    // set: read value length and bytes
                    reader.read_exact(&mut len_buf).map_err(|e| {
                        StoreError::CorruptedData(format!(
                            "Failed to read val len in {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    let val_len = u32::from_le_bytes(len_buf) as usize;
                    let mut val_bytes = vec![0u8; val_len];
                    reader.read_exact(&mut val_bytes).map_err(|e| {
                        StoreError::CorruptedData(format!(
                            "Failed to read val in {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    values.insert(key, val_bytes);
                },
                1 => {
                    // delete
                    values.remove(&key);
                },
                other => {
                    return Err(StoreError::CorruptedData(format!(
                        "Unknown opcode {} in segment {}",
                        other,
                        path.display()
                    )));
                },
            }
        }

        Ok(())
    }

    /// Append a set operation to the active segment and update in-memory index.
    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        // write entry: op(1) = 0, key_len(u32), key, val_len(u32), val
        let writer = self
            .active_writer
            .as_mut()
            .ok_or_else(|| StoreError::Io(std::io::Error::other("Active writer missing")))?;

        // Build buffers
        let key_bytes = key.as_bytes();
        let key_len = (key_bytes.len() as u32).to_le_bytes();
        let val_len = (value.len() as u32).to_le_bytes();

        writer.write_all(&[0u8]).map_err(StoreError::Io)?;
        writer.write_all(&key_len).map_err(StoreError::Io)?;
        writer.write_all(key_bytes).map_err(StoreError::Io)?;
        writer.write_all(&val_len).map_err(StoreError::Io)?;
        writer.write_all(value).map_err(StoreError::Io)?;
        writer.flush().map_err(StoreError::Io)?;

        // update in-memory
        self.values.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    /// Append a delete operation to the active segment and update in-memory index.
    pub fn delete(&mut self, key: &str) -> Result<()> {
        let writer = self
            .active_writer
            .as_mut()
            .ok_or_else(|| StoreError::Io(std::io::Error::other("Active writer missing")))?;

        let key_bytes = key.as_bytes();
        let key_len = (key_bytes.len() as u32).to_le_bytes();

        writer.write_all(&[1u8]).map_err(StoreError::Io)?;
        writer.write_all(&key_len).map_err(StoreError::Io)?;
        writer.write_all(key_bytes).map_err(StoreError::Io)?;
        writer.flush().map_err(StoreError::Io)?;

        self.values.remove(key);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.values.get(key).cloned())
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    /// Create a fresh active segment. Used after compaction to start a new file.
    pub fn reset_active_segment(&mut self) -> Result<()> {
        // Close current writer by dropping it
        self.active_writer = None;

        // increment id and create new file
        self.active_segment_id = self
            .active_segment_id
            .checked_add(1)
            .ok_or_else(|| StoreError::Io(std::io::Error::other("segment id overflow")))?;
        let path = self.base_dir.join(format!(
            "{}{}{}",
            SEGMENT_PREFIX, self.active_segment_id, SEGMENT_SUFFIX
        ));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(StoreError::Io)?;
        self.active_writer = Some(BufWriter::new(file));
        Ok(())
    }

    /// Returns base dir (clone)
    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    /// Simple stats view
    pub fn stats(&self) -> StoreStats {
        // Count segments by scanning dir (cheap)
        let num_segments = match fs::read_dir(&self.base_dir) {
            Ok(rd) => rd
                .filter_map(|r| r.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.starts_with(SEGMENT_PREFIX) && n.ends_with(SEGMENT_SUFFIX))
                        .unwrap_or(false)
                })
                .count(),
            Err(_) => 0,
        };

        StoreStats {
            num_keys: self.values.len(),
            num_segments,
            total_bytes: self.values.values().map(|v| v.len() as u64).sum::<u64>(),
            active_segment_id: self.active_segment_id as usize,
            oldest_segment_id: 0, // could be improved by reading min id
        }
    }

    /// High-level convenience to trigger compaction using compaction.rs
    pub fn compact(&mut self) -> Result<()> {
        // Delegates to compaction module which will remove old segments and then
        // call reset_active_segment() to prepare a fresh one.
        super::compaction::compact(self)
    }
}
