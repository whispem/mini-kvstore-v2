use crate::store::config::StoreConfig;
use crate::store::index::Index;
use crate::store::segment::Segment;
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

pub struct KVStore {
    pub config: StoreConfig,
    pub segments: HashMap<usize, Segment>,
    pub index: Index,
    pub active_id: usize,
}

impl KVStore {
    /// Open store with default config
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let config = StoreConfig::new(dir.as_ref());
        Self::open_with_config(config)
    }

    /// Open store with custom config
    pub fn open_with_config(config: StoreConfig) -> Result<Self> {
        fs::create_dir_all(&config.data_dir)?;

        let mut ids: Vec<usize> = fs::read_dir(&config.data_dir)?
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("segment-") && fname.ends_with(".dat") {
                    fname["segment-".len()..fname.len() - 4].parse().ok()
                } else {
                    None
                }
            })
            .collect();
        ids.sort_unstable();

        let active_id = ids.last().copied().unwrap_or(0);

        let mut segments = HashMap::new();
        for &id in &ids {
            let seg = Segment::open(&config.data_dir, id)?;
            segments.insert(id, seg);
        }

        if !segments.contains_key(&active_id) {
            let seg = Segment::open(&config.data_dir, active_id)?;
            segments.insert(active_id, seg);
        }

        let mut store = KVStore {
            config,
            segments,
            index: Index::new(),
            active_id,
        };

        store.rebuild_index()?;

        Ok(store)
    }

    /// Rebuild in-memory index from all segments
    fn rebuild_index(&mut self) -> Result<()> {
        let mut ordered_ids: Vec<usize> = self.segments.keys().copied().collect();
        ordered_ids.sort_unstable();

        for id in ordered_ids {
            let seg = self.segments.get_mut(&id).ok_or_else(|| {
                Error::new(ErrorKind::NotFound, "Segment disappeared during rebuild")
            })?;

            let mut pos = 0u64;
            while pos < seg.len {
                match seg.read_record_at(pos) {
                    Ok(Some((key, value_opt))) => {
                        if let Some(ref value) = value_opt {
                            self.index.insert(key.clone(), id, pos, value.len() as u64);
                        } else {
                            // Tombstone
                            self.index.remove(&key);
                        }

                        // Calculate next position based on simple format
                        // Format: key_len (8) + value_len (8) + key + value
                        let key_bytes = key.as_bytes();
                        let record_size = 8 + 8 + key_bytes.len() as u64 + 
                            value_opt.as_ref().map(|v| v.len() as u64).unwrap_or(0);
                        pos += record_size;
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to read record at position {} in segment {}: {}", 
                            pos, id, e
                        );
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Set a key-value pair
    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        // Check if active segment is full, create new one if needed
        let active_seg = self.segments.get_mut(&self.active_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Active segment not found"))?;

        if active_seg.is_full() {
            self.active_id += 1;
            let new_seg = Segment::open(&self.config.data_dir, self.active_id)?;
            self.segments.insert(self.active_id, new_seg);
        }

        let active_seg = self.segments.get_mut(&self.active_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Active segment not found"))?;

        let offset = active_seg.append(key.as_bytes(), value)?;
        self.index.insert(key.to_string(), self.active_id, offset, value.len() as u64);

        Ok(())
    }

    /// Get a value by key
    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(&(seg_id, offset, _len)) = self.index.get(key) {
            let seg = self.segments.get_mut(&seg_id)
                .ok_or_else(|| Error::new(ErrorKind::NotFound, "Segment not found"))?;

            seg.read_value_at(offset)
        } else {
            Ok(None)
        }
    }

    /// Delete a key
    pub fn delete(&mut self, key: &str) -> Result<()> {
        // Check if key exists
        if !self.index.map.contains_key(key) {
            return Ok(()); // Deleting non-existent key is a no-op
        }

        // Check if active segment is full
        let active_seg = self.segments.get_mut(&self.active_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Active segment not found"))?;

        if active_seg.is_full() {
            self.active_id += 1;
            let new_seg = Segment::open(&self.config.data_dir, self.active_id)?;
            self.segments.insert(self.active_id, new_seg);
        }

        let active_seg = self.segments.get_mut(&self.active_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Active segment not found"))?;

        active_seg.append_tombstone(key.as_bytes())?;
        self.index.remove(key);

        Ok(())
    }

    /// List all keys in the store
    pub fn list_keys(&self) -> Vec<String> {
        self.index.map.keys().cloned().collect()
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        let num_keys = self.index.len();
        let num_segments = self.segments.len();
        
        let total_bytes: u64 = self.segments.values().map(|s| s.len).sum();
        
        let oldest_segment_id = self.segments.keys().copied().min().unwrap_or(0);

        StoreStats {
            num_keys,
            num_segments,
            total_bytes,
            active_segment_id: self.active_id,
            oldest_segment_id,
        }
    }

    /// Run manual compaction
    pub fn compact(&mut self) -> Result<()> {
        crate::store::compaction::compact_segments(self)
    }
}
