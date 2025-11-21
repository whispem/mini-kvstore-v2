//! Key-value store engine implementation.

use crate::store::compaction;
use crate::store::error::{Result, StoreError};
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const SEGMENT_SIZE_LIMIT: u64 = 16 * 1024 * 1024; // 16 MB
const TOMBSTONE: u64 = u64::MAX;

#[derive(Debug)]
pub struct KVStore {
    /// Directory where all segment files and metadata live.
    pub base_dir: PathBuf,
    /// In-memory index (key, segment_id, offset, length)
    index: HashMap<String, (usize, u64, u64)>,
    /// Active segment for writing
    active_segment_id: usize,
    /// Active segment file
    active_segment: BufWriter<File>,
    /// Current offset in active segment
    active_offset: u64,
}

impl KVStore {
    /// Opens a KVStore at the given directory, creating it if missing.
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_dir = dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(StoreError::Io)?;
        }

        // Find existing segments or create first one
        let segments = Self::find_segments(&base_dir)?;
        let active_segment_id = segments.iter().max().copied().unwrap_or(0);

        // Open or create active segment
        let segment_path = base_dir.join(format!("segment-{:04}.dat", active_segment_id));
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&segment_path)?;

        let active_offset = file.metadata()?.len();
        let active_segment = BufWriter::new(file);

        let mut store = Self {
            base_dir,
            index: HashMap::new(),
            active_segment_id,
            active_segment,
            active_offset,
        };

        // Rebuild index from existing segments
        store.rebuild_index()?;

        Ok(store)
    }

    /// Finds all segment IDs in the directory
    fn find_segments(dir: &Path) -> Result<Vec<usize>> {
        let mut segments = Vec::new();
        
        if !dir.exists() {
            return Ok(segments);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                if name.starts_with("segment-") && name.ends_with(".dat") {
                    if let Ok(id) = name["segment-".len()..name.len() - ".dat".len()].parse::<usize>() {
                        segments.push(id);
                    }
                }
            }
        }
        
        segments.sort_unstable();
        Ok(segments)
    }

    /// Rebuilds the index by scanning all segments
    fn rebuild_index(&mut self) -> Result<()> {
        self.index.clear();
        let segments = Self::find_segments(&self.base_dir)?;

        for seg_id in segments {
            let path = self.base_dir.join(format!("segment-{:04}.dat", seg_id));
            let mut file = File::open(&path)?;
            let mut offset = 0u64;

            loop {
                // Read key length
                let mut len_buf = [0u8; 8];
                if file.read_exact(&mut len_buf).is_err() {
                    break; // End of file
                }
                let key_len = u64::from_le_bytes(len_buf);

                // Read value length
                if file.read_exact(&mut len_buf).is_err() {
                    break;
                }
                let value_len = u64::from_le_bytes(len_buf);

                // Read key
                let mut key_buf = vec![0u8; key_len as usize];
                file.read_exact(&mut key_buf)?;
                let key = String::from_utf8_lossy(&key_buf).to_string();

                let record_offset = offset;
                let record_len = 8 + 8 + key_len + value_len;

                // Check if tombstone
                if value_len == TOMBSTONE {
                    self.index.remove(&key);
                    offset += record_len;
                    // Skip past tombstone marker
                    file.seek(SeekFrom::Current(0))?;
                } else {
                    // Skip value
                    file.seek(SeekFrom::Current(value_len as i64))?;
                    self.index.insert(key, (seg_id, record_offset, record_len));
                    offset += record_len;
                }
            }
        }

        Ok(())
    }

    /// Returns the base directory for this store.
    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    /// Provides a list of all current keys in the store.
    pub fn list_keys(&self) -> Vec<String> {
        self.index.keys().cloned().collect()
    }

    /// Fetches the value for a specific key.
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(&(seg_id, offset, _len)) = self.index.get(key) {
            let path = self.base_dir.join(format!("segment-{:04}.dat", seg_id));
            let mut file = File::open(&path)?;
            
            // Seek to record
            file.seek(SeekFrom::Start(offset))?;

            // Read key length
            let mut len_buf = [0u8; 8];
            file.read_exact(&mut len_buf)?;
            let key_len = u64::from_le_bytes(len_buf);

            // Read value length
            file.read_exact(&mut len_buf)?;
            let value_len = u64::from_le_bytes(len_buf);

            // Skip key
            file.seek(SeekFrom::Current(key_len as i64))?;

            // Read value
            let mut value_buf = vec![0u8; value_len as usize];
            file.read_exact(&mut value_buf)?;

            Ok(Some(value_buf))
        } else {
            Ok(None)
        }
    }

    /// Inserts a key-value pair.
    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len() as u64;
        let value_len = value.len() as u64;
        let record_size = 8 + 8 + key_len + value_len;

        // Check if we need to rotate segment
        if self.active_offset + record_size > SEGMENT_SIZE_LIMIT {
            self.rotate_segment()?;
        }

        let offset = self.active_offset;

        // Write record: [key_len][value_len][key][value]
        self.active_segment.write_all(&key_len.to_le_bytes())?;
        self.active_segment.write_all(&value_len.to_le_bytes())?;
        self.active_segment.write_all(key_bytes)?;
        self.active_segment.write_all(value)?;
        self.active_segment.flush()?;

        self.active_offset += record_size;

        // Update index
        self.index.insert(
            key.to_string(),
            (self.active_segment_id, offset, record_size),
        );

        Ok(())
    }

    /// Deletes a key from the store by writing a tombstone.
    pub fn delete(&mut self, key: &str) -> Result<()> {
        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len() as u64;
        let record_size = 8 + 8 + key_len;

        // Check if we need to rotate segment
        if self.active_offset + record_size > SEGMENT_SIZE_LIMIT {
            self.rotate_segment()?;
        }

        // Write tombstone: [key_len][TOMBSTONE][key]
        self.active_segment.write_all(&key_len.to_le_bytes())?;
        self.active_segment.write_all(&TOMBSTONE.to_le_bytes())?;
        self.active_segment.write_all(key_bytes)?;
        self.active_segment.flush()?;

        self.active_offset += record_size;

        // Remove from index
        self.index.remove(key);

        Ok(())
    }

    /// Rotates to a new segment file
    fn rotate_segment(&mut self) -> Result<()> {
        // Flush current segment
        self.active_segment.flush()?;

        // Create new segment
        self.active_segment_id += 1;
        let new_path = self.base_dir.join(format!("segment-{:04}.dat", self.active_segment_id));
        
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&new_path)?;

        self.active_segment = BufWriter::new(file);
        self.active_offset = 0;

        Ok(())
    }

    /// Returns store statistics.
    pub fn stats(&self) -> StoreStats {
        let segments = Self::find_segments(&self.base_dir).unwrap_or_default();
        let mut total_bytes = 0u64;

        for seg_id in &segments {
            let path = self.base_dir.join(format!("segment-{:04}.dat", seg_id));
            if let Ok(metadata) = fs::metadata(&path) {
                total_bytes += metadata.len();
            }
        }

        StoreStats {
            num_keys: self.index.len(),
            num_segments: segments.len(),
            total_bytes,
            active_segment_id: self.active_segment_id,
            oldest_segment_id: segments.first().copied().unwrap_or(0),
        }
    }

    /// Starts manual compaction for this store.
    pub fn compact(&mut self) -> Result<()> {
        compaction::compact(self)
    }
}
