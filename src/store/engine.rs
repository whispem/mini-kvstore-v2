#![allow(clippy::while_let_loop)]
use crate::store::bloom::BloomIndex;
use crate::store::error::{Result, StoreError};
use crate::store::index::Index;
use crate::store::record::{self, OP_DEL, OP_SET};
use crate::store::snapshot;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const SEGMENT_PREFIX: &str = "segment-";
const SEGMENT_SUFFIX: &str = ".dat";

pub struct KVStore {
    pub base_dir: PathBuf,

    values: HashMap<String, Vec<u8>>,
    index: Index,
    bloom: BloomIndex,

    active_segment_id: u64,
    active_writer: Option<BufWriter<File>>,

    max_segment_size: u64,
}

impl KVStore {
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let base_dir = dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).map_err(StoreError::Io)?;
        }

        let mut segment_paths: Vec<(u64, PathBuf)> = Vec::new();
        for entry in fs::read_dir(&base_dir).map_err(StoreError::Io)? {
            let entry = entry.map_err(StoreError::Io)?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|x| x.to_str()) {
                if name.starts_with(SEGMENT_PREFIX) && name.ends_with(SEGMENT_SUFFIX) {
                    let id_str = &name[SEGMENT_PREFIX.len()..name.len() - SEGMENT_SUFFIX.len()];
                    if let Ok(id) = id_str.parse::<u64>() {
                        segment_paths.push((id, path));
                    }
                }
            }
        }

        segment_paths.sort_by_key(|(id, _)| *id);

        let mut values = HashMap::new();
        let mut bloom = BloomIndex::new(50_000);

        // Try to load index from snapshot first
        let mut index = Self::try_load_snapshot(&base_dir).unwrap_or_else(Index::new);
        let snapshot_loaded = !index.is_empty();

        // Replay segments
        let replay_start = std::time::Instant::now();
        for (id, path) in &segment_paths {
            Self::replay_segment(path, *id, &mut values, &mut index, &mut bloom)?;
        }

        if !snapshot_loaded && !segment_paths.is_empty() {
            println!(
                "✓ Rebuilt index from segments in {:.2}s",
                replay_start.elapsed().as_secs_f64()
            );
        }

        let last_id = segment_paths.last().map(|(id, _)| *id).unwrap_or(0);
        let new_id = last_id + 1;

        let active_path = base_dir.join(format!("{}{}{}", SEGMENT_PREFIX, new_id, SEGMENT_SUFFIX));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&active_path)?;
        let writer = BufWriter::new(file);

        Ok(Self {
            base_dir,
            values,
            index,
            bloom,
            active_segment_id: new_id,
            active_writer: Some(writer),
            max_segment_size: 16 * 1024 * 1024,
        })
    }

    /// Try to load index from snapshot, fallback to empty if missing
    fn try_load_snapshot(base_dir: &Path) -> Option<Index> {
        let snapshot_path = base_dir.join("index.snapshot");
        if snapshot_path.exists() {
            match snapshot::load_snapshot(&snapshot_path) {
                Ok(idx) => {
                    println!("✓ Loaded index from snapshot ({} keys)", idx.len());
                    return Some(idx);
                }
                Err(e) => {
                    eprintln!(
                        "⚠ Failed to load snapshot: {}, rebuilding from segments",
                        e
                    );
                }
            }
        }
        None
    }

    fn replay_segment(
        path: &Path,
        seg_id: u64,
        values: &mut HashMap<String, Vec<u8>>,
        index: &mut Index,
        bloom: &mut BloomIndex,
    ) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        loop {
            match record::read_record(&mut reader)? {
                Some((op, key, value)) => match op {
                    OP_SET => {
                        if let Some(v) = value {
                            values.insert(key.clone(), v.clone());
                            index.insert(key.clone(), seg_id, 0);
                            bloom.insert(&key);
                        }
                    }
                    OP_DEL => {
                        values.remove(&key);
                        index.remove(&key);
                    }
                    _ => {
                        return Err(StoreError::CorruptedData(format!(
                            "invalid opcode in {}",
                            path.display()
                        )));
                    }
                },
                None => break,
            }
        }

        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let writer = self
            .active_writer
            .as_mut()
            .ok_or_else(|| StoreError::Io(std::io::Error::other("no active writer")))?;

        record::write_record(&mut *writer, OP_SET, key, Some(value))?;
        writer.flush().map_err(StoreError::Io)?;

        self.values.insert(key.to_string(), value.to_vec());
        self.index
            .insert(key.to_string(), self.active_segment_id, 0);
        self.bloom.insert(key);

        if let Ok(meta) = writer.get_ref().metadata() {
            if meta.len() >= self.max_segment_size {
                self.reset_active_segment()?;
            }
        }

        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        let writer = self
            .active_writer
            .as_mut()
            .ok_or_else(|| StoreError::Io(std::io::Error::other("no active writer")))?;

        record::write_record(&mut *writer, OP_DEL, key, None)?;
        writer.flush().map_err(StoreError::Io)?;

        self.values.remove(key);
        self.index.remove(key);

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Fast in-memory path
        if let Some(v) = self.values.get(key) {
            return Ok(Some(v.clone()));
        }

        // Bloom check (if bloom says "no", definitely not present)
        if !self.bloom.might_contain(key) {
            return Ok(None);
        }

        Ok(None)
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    pub fn reset_active_segment(&mut self) -> Result<()> {
        self.active_writer = None;

        self.active_segment_id = self
            .active_segment_id
            .checked_add(1)
            .ok_or_else(|| StoreError::Io(std::io::Error::other("segment id overflow")))?;
        let path = self.base_dir.join(format!(
            "{}{}{}",
            SEGMENT_PREFIX, self.active_segment_id, SEGMENT_SUFFIX
        ));
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        self.active_writer = Some(BufWriter::new(file));
        Ok(())
    }

    pub fn base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    pub fn stats(&self) -> crate::store::stats::StoreStats {
        let num_segments = fs::read_dir(&self.base_dir)
            .ok()
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .map(|n| n.starts_with(SEGMENT_PREFIX) && n.ends_with(SEGMENT_SUFFIX))
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);

        crate::store::stats::StoreStats {
            num_keys: self.values.len(),
            num_segments,
            total_bytes: self.values.values().map(|v| v.len() as u64).sum(),
            active_segment_id: self.active_segment_id as usize,
            oldest_segment_id: 0,
        }
    }

    /// Save index snapshot for faster restarts
    pub fn save_snapshot(&self) -> Result<()> {
        let snapshot_path = self.base_dir.join("index.snapshot");
        snapshot::save_snapshot(&self.index, &snapshot_path)?;
        Ok(())
    }

    pub fn compact(&mut self) -> Result<()> {
        super::compaction::compact(self)
    }
}
