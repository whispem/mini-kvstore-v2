use std::fs::{File, OpenOptions};
use std::io::{Read, Result, Seek, SeekFrom, Write};
use std::path::PathBuf;

const SEGMENT_SIZE_LIMIT: u64 = 1024 * 1024;

pub struct Segment {
    pub file: File,
    pub len: u64,
}

impl Segment {
    pub fn open(dir: &PathBuf, id: usize) -> Result<Self> {
        let filename = format!("segment-{}.dat", id);
        let path = dir.join(filename);
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;
        let len = file.seek(SeekFrom::End(0))?;
        Ok(Segment { file, len })
    }

    pub fn append(&mut self, key: &[u8], value: &[u8]) -> Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let key_len = key.len() as u64;
        let value_len = value.len() as u64;
        self.file.write_all(&key_len.to_le_bytes())?;
        self.file.write_all(&value_len.to_le_bytes())?;
        self.file.write_all(key)?;
        self.file.write_all(value)?;
        self.file.sync_all()?;
        self.len = self.file.seek(SeekFrom::End(0))?;
        Ok(offset)
    }

    pub fn append_tombstone(&mut self, key: &[u8]) -> Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let key_len = key.len() as u64;
        let value_len = u64::MAX; // Tombstone
        self.file.write_all(&key_len.to_le_bytes())?;
        self.file.write_all(&value_len.to_le_bytes())?;
        self.file.write_all(key)?;
        self.file.sync_all()?;
        self.len = self.file.seek(SeekFrom::End(0))?;
        Ok(offset)
    }

    pub fn is_full(&self) -> bool {
        self.len >= SEGMENT_SIZE_LIMIT
    }

    pub fn read_record_at(&mut self, offset: u64) -> Result<Option<(String, Option<Vec<u8>>)>> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf8 = [0u8; 8];
        if let Err(_) = self.file.read_exact(&mut buf8) {
            return Ok(None);
        }
        let key_len = u64::from_le_bytes(buf8);

        if let Err(_) = self.file.read_exact(&mut buf8) {
            return Ok(None);
        }
        let value_len = u64::from_le_bytes(buf8);

        let mut key_buf = vec![0u8; key_len as usize];
        if let Err(_) = self.file.read_exact(&mut key_buf) {
            return Ok(None);
        }
        let key = String::from_utf8_lossy(&key_buf).to_string();

        if value_len == u64::MAX {
            return Ok(Some((key, None))); // tombstone
        } else {
            let mut val_buf = vec![0u8; value_len as usize];
            if let Err(_) = self.file.read_exact(&mut val_buf) {
                return Ok(None);
            }
            return Ok(Some((key, Some(val_buf))));
        }
    }

    pub fn read_value_at(&mut self, offset: u64) -> Result<Option<Vec<u8>>> {
        self.file.seek(SeekFrom::Start(offset))?;

        let mut buf8 = [0u8; 8];
        self.file.read_exact(&mut buf8)?;
        let key_len = u64::from_le_bytes(buf8);

        self.file.read_exact(&mut buf8)?;
        let value_len = u64::from_le_bytes(buf8);

        let mut key_buf = vec![0u8; key_len as usize];
        self.file.read_exact(&mut key_buf)?;

        if value_len == u64::MAX {
            Ok(None) // Tombstone
        } else {
            let mut val_buf = vec![0u8; value_len as usize];
            self.file.read_exact(&mut val_buf)?;
            Ok(Some(val_buf))
        }
    }
}
