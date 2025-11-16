use crc32fast::Hasher;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const SEGMENTS_DIR: &str = "segments";
const SEGMENT_PREFIX: &str = "segment_";
const SEGMENT_EXT: &str = "log";

#[derive(Debug)]
pub struct KvStore {
    data_dir: PathBuf,
    segments_dir: PathBuf,
    /// in-memory index: key -> Option(value). None means deleted (tombstone).
    index: HashMap<String, Option<Vec<u8>>>,
    /// next segment id (for new segment files)
    next_segment_id: u64,
}

impl KvStore {
    /// open (or create) store at path
    pub fn open<P: AsRef<Path>>(root: P) -> io::Result<Self> {
        let data_dir = root.as_ref().to_path_buf();
        let segments_dir = data_dir.join(SEGMENTS_DIR);
        fs::create_dir_all(&segments_dir)?;

        // find existing segments and rebuild index
        let mut segment_files: Vec<PathBuf> = fs::read_dir(&segments_dir)?
            .filter_map(|e| e.ok().map(|d| d.path()))
            .filter(|p| p.is_file())
            .collect();

        // sort by name (segment_0.log, segment_1.log, ...)
        segment_files.sort();

        let mut index: HashMap<String, Option<Vec<u8>>> = HashMap::new();
        for seg in &segment_files {
            KvStore::replay_segment(seg, &mut index)?;
        }

        // next id = highest existing + 1
        let next_segment_id = segment_files
            .iter()
            .filter_map(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|s| {
                        s.strip_prefix(SEGMENT_PREFIX)
                            .and_then(|after| after.strip_suffix(&format!(".{}", SEGMENT_EXT)))
                    })
                    .and_then(|num| num.parse::<u64>().ok())
            })
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);

        Ok(Self {
            data_dir,
            segments_dir,
            index,
            next_segment_id,
        })
    }

    fn replay_segment(path: &Path, index: &mut HashMap<String, Option<Vec<u8>>>) -> io::Result<()> {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let mut cursor = 0usize;
        while cursor + 13 <= buf.len() {
            // header: key_len(u32) value_len(u32) tombstone(u8)
            let key_len = u32::from_le_bytes(buf[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;
            let value_len = u32::from_le_bytes(buf[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;
            let tomb = buf[cursor];
            cursor += 1;
            if cursor + key_len + value_len + 4 > buf.len() {
                // truncated or corrupted -> stop
                break;
            }
            let key = String::from_utf8_lossy(&buf[cursor..cursor + key_len]).to_string();
            cursor += key_len;
            let value = buf[cursor..cursor + value_len].to_vec();
            cursor += value_len;
            let crc_stored = u32::from_le_bytes(buf[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;

            // verify crc
            let mut hasher = Hasher::new();
            hasher.update(&key.as_bytes());
            hasher.update(&[tomb]);
            hasher.update(&value);
            let crc = hasher.finalize();
            if crc != crc_stored {
                // corrupted record — skip / stop replay
                break;
            }

            if tomb == 1 {
                index.insert(key, None);
            } else {
                index.insert(key, Some(value));
            }
        }
        Ok(())
    }

    fn current_segment_path(&self) -> PathBuf {
        self.segments_dir
            .join(format!("{}{}.{}", SEGMENT_PREFIX, self.next_segment_id, SEGMENT_EXT))
    }

    fn append_record(&mut self, key: &str, value: Option<&[u8]>) -> io::Result<()> {
        let path = self.current_segment_path();
        // ensure file exists
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let key_bytes = key.as_bytes();
        let key_len = key_bytes.len() as u32;
        let val_bytes = value.map(|v| v.to_vec()).unwrap_or_default();
        let val_len = val_bytes.len() as u32;
        let tomb = if value.is_none() { 1u8 } else { 0u8 };

        // checksum over key + tomb + value
        let mut hasher = Hasher::new();
        hasher.update(&key_bytes);
        hasher.update(&[tomb]);
        hasher.update(&val_bytes);
        let crc = hasher.finalize();

        file.write_all(&key_len.to_le_bytes())?;
        file.write_all(&val_len.to_le_bytes())?;
        file.write_all(&[tomb])?;
        file.write_all(key_bytes)?;
        file.write_all(&val_bytes)?;
        file.write_all(&crc.to_le_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// set key -> value
    pub fn set(&mut self, key: String, value: Vec<u8>) -> io::Result<()> {
        self.append_record(&key, Some(&value))?;
        self.index.insert(key, Some(value));
        // rotate segment if gets too large? simple heuristic
        let path = self.current_segment_path();
        if let Ok(metadata) = fs::metadata(&path) {
            // rotate at 2 MB for this simple demo
            if metadata.len() > 2 * 1024 * 1024 {
                self.next_segment_id += 1;
            }
        }
        Ok(())
    }

    /// get key
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.index.get(key).and_then(|opt| opt.clone())
    }

    /// delete key (tombstone)
    pub fn delete(&mut self, key: &str) -> io::Result<bool> {
        if !self.index.contains_key(key) || self.index.get(key).and_then(|v| v.as_ref()).is_none() {
            return Ok(false);
        }
        self.append_record(key, None)?;
        self.index.insert(key.to_string(), None);
        // rotate if large (same heuristic)
        let path = self.current_segment_path();
        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.len() > 2 * 1024 * 1024 {
                self.next_segment_id += 1;
            }
        }
        Ok(true)
    }

    /// list keys (including deleted ones? we filter deleted)
    pub fn keys(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .index
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|_| k.clone()))
            .collect();
        v.sort();
        v
    }

    /// quick stats
    pub fn stats(&self) -> (usize, usize) {
        let segments = fs::read_dir(&self.segments_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        let keys = self.index.iter().filter(|(_, v)| v.is_some()).count();
        (segments, keys)
    }

    /// manual compaction: write current live keys into a new segment, then remove old segments
    pub fn compact(&mut self) -> io::Result<()> {
        // gather live keys
        let mut live: Vec<(String, Vec<u8>)> = self
            .index
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|val| (k.clone(), val.clone())))
            .collect();
        live.sort_by(|a, b| a.0.cmp(&b.0));

        // create new segment file with next_segment_id
        let new_seg_id = self.next_segment_id;
        let new_path = self.segments_dir.join(format!("{}{}.{}", SEGMENT_PREFIX, new_seg_id, SEGMENT_EXT));
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&new_path)?;
            for (k, v) in &live {
                let key_bytes = k.as_bytes();
                let key_len = key_bytes.len() as u32;
                let val_len = v.len() as u32;
                let tomb = 0u8;
                let mut hasher = Hasher::new();
                hasher.update(key_bytes);
                hasher.update(&[tomb]);
                hasher.update(v);
                let crc = hasher.finalize();

                file.write_all(&key_len.to_le_bytes())?;
                file.write_all(&val_len.to_le_bytes())?;
                file.write_all(&[tomb])?;
                file.write_all(key_bytes)?;
                file.write_all(v)?;
                file.write_all(&crc.to_le_bytes())?;
            }
            file.flush()?;
        }

        // remove all old segments (except the one we just wrote) and set next_segment_id = new_seg_id + 1
        for entry in fs::read_dir(&self.segments_dir)? {
            let p = entry?.path();
            if p == new_path {
                continue;
            }
            if p.is_file() {
                let _ = fs::remove_file(p);
            }
        }
        self.next_segment_id = new_seg_id + 1;

        // rebuild index from the new single segment
        self.index.clear();
        KvStore::replay_segment(&new_path, &mut self.index)?;
        Ok(())
    }
}
