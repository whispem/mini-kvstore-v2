use crate::store::index::Index;
use crate::store::segment::Segment;
use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

pub struct KvStore {
    pub dir: PathBuf,
    pub segments: HashMap<usize, Segment>,
    pub index: Index,
    pub active_id: usize,
}

impl KvStore {
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        let mut ids: Vec<usize> = fs::read_dir(&dir)?
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("segment-") && fname.ends_with(".dat") {
                    fname["segment-".len()..fname.len()-4].parse().ok()
                } else {
                    None
                }
            })
            .collect();
        ids.sort_unstable();
        let active_id = if ids.is_empty() { 0 } else { *ids.last().unwrap() };
        let mut segments = HashMap::new();
        for &id in &ids {
            let seg = Segment::open(&dir, id)?;
            segments.insert(id, seg);
        }
        if !segments.contains_key(&active_id) {
            let seg = Segment::open(&dir, active_id)?;
            segments.insert(active_id, seg);
        }
        let mut store = KvStore {
            dir,
            segments,
            index: Index::new(),
            active_id,
        };
        let mut ordered_ids: Vec<usize> = store.segments.keys().cloned().collect();
        ordered_ids.sort_unstable();
        for id in ordered_ids {
            let seg = store.segments.get_mut(&id).unwrap();
            let mut pos = 0u64;
            while pos < seg.len {
                seg.file.seek(std::io::SeekFrom::Start(pos))?;
                let mut buf8 = [0u8; 8];
                if let Err(_) = seg.file.read_exact(&mut buf8) {
                    break;
                }
                let key_len = u64::from_le_bytes(buf8);
                seg.file.read_exact(&mut buf8)?;
                let value_len = u64::from_le_bytes(buf8);
                let mut key_buf = vec![0u8; key_len as usize];
                seg.file.read_exact(&mut key_buf)?;
                let key = String::from_utf8_lossy(&key_buf).to_string();
                let record_header_size = 8 + 8;
                let record_size = record_header_size + key_len + value_len;
                if value_len == u64::MAX {
                    store.index.remove(&key);
                } else {
                    store.index.insert(key, id, pos, value_len);
                }
                pos += record_size;
            }
        }
        Ok(store)
    }

    fn active_segment_mut(&mut self) -> &mut Segment {
        self.segments.get_mut(&self.active_id).unwrap()
    }

    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let key_b = key.as_bytes();
        let offset = self.active_segment_mut().append(key_b, value)?;
        self.index
            .insert(key.to_string(), self.active_id, offset, value.len() as u64);
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some((seg_id, offset, value_len)) = self.index.get(key) {
            if let Some(seg) = self.segments.get_mut(seg_id) {
                return seg.read_value_at(*offset);
            }
        }
        Ok(None)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        let key_b = key.as_bytes();
        let tombstone_value_len = u64::MAX;
        let mut seg = self.active_segment_mut();
        let offset = seg.file.seek(std::io::SeekFrom::End(0))?;
        let key_len = key_b.len() as u64;
        seg.file.write_all(&key_len.to_le_bytes())?;
        seg.file.write_all(&tombstone_value_len.to_le_bytes())?;
        seg.file.write_all(key_b)?;
        seg.file.flush()?;
        seg.len = seg.file.seek(std::io::SeekFrom::End(0))?;
        self.index.remove(key);
        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        use crate::store::compaction::compact_segments;
        compact_segments(self)
    }
}
