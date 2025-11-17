use crate::store::config::StoreConfig;
use crate::store::index::Index;
use crate::store::record::{RecordHeader, TOMBSTONE_MARKER};
use crate::store::segment::Segment;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind, Result, Seek}; 
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
                            "No record found at position {} in segment {}", pos, id
                        );
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
}
