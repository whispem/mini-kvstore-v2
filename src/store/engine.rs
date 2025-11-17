use crate::store::config::StoreConfig;
use crate::store::index::Index;
use crate::store::record::{RecordHeader, TOMBSTONE_MARKER};
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
                let start_pos = pos;

                match seg.read_record_at(pos) {
                    Ok(Some((key, value_opt))) => {
                        if let Some(value) = value_opt {
                            self.index.insert(key, id, pos, value.len() as u64);
                        } else {
                            // Tombstone
                            self.index.remove(&key);
                        }

                        // Calculate next position
                        seg.file.seek(std::io::SeekFrom::Start(pos))?;
                        if let Some(header) = RecordHeader::read_from(&mut seg.file)? {
                            let record_size = RecordHeader::SIZE as u64
                                + header.key_len as u64
                                + if header.value_len == TOMBSTONE_MARKER {
                                    0
                                } else {
                                    header.value_len as u64
                                };
                            pos = start_pos + record_size;
                        } else {
                            break;
                        }
                    }
                    Ok(None) => {
                        eprintln!(
                            "Warning: Corrupt record at offset {} in segment {}, stopping rebuild for this segment",
                            pos, id
                        );
                        break;
                    }
                    Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                        eprintln!(
                            "Warning: Incomplete record at offset {} in segment {}, likely due to crash",
                            pos, id
                        );
                        break;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Error reading record at offset {} in segment {}: {}",
                            pos, id, e
                        );
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn active_segment_mut(&mut self) -> Result<&mut Segment> {
        self.segments
            .get_mut(&self.active_id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Active segment not found"))
    }

    /// Rotate to a new segment if active is full
    fn maybe_rotate_segment(&mut self) -> Result<()> {
        if self.active_segment_mut()?.is_full() {
            let new_id = self.active_id + 1;
            let new_seg = Segment::open(&self.config.data_dir, new_id)?;
            self.segments.insert(new_id, new_seg);
            self.active_id = new_id;
            println!("Rotated to new segment: {}", new_id);
        }
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        self.maybe_rotate_segment()?;

        let key_b = key.as_bytes();
        let offset = self.active_segment_mut()?.append(key_b, value)?;
        self.index
            .insert(key.to_string(), self.active_id, offset, value.len() as u64);
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some((seg_id, offset, _value_len)) = self.index.get(key) {
            if let Some(seg) = self.segments.get_mut(seg_id) {
                return seg.read_value_at(*offset);
            }
        }
        Ok(None)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.maybe_rotate_segment()?;

        let key_b = key.as_bytes();
        self.active_segment_mut()?.append_tombstone(key_b)?;
        self.index.remove(key);
        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        crate::store::compaction::compact_segments(self)
    }

    pub fn stats(&self) -> StoreStats {
        let total_bytes: u64 = self.segments.values().map(|s| s.len).sum();
        let oldest = self.segments.keys().copied().min().unwrap_or(0);

        StoreStats {
            num_keys: self.index.len(),
            num_segments: self.segments.len(),
            total_bytes,
            active_segment_id: self.active_id,
            oldest_segment_id: oldest,
        }
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.index.map.keys().cloned().collect()
    }
}
