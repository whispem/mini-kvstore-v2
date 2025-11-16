use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub struct Segment {
    pub id: usize,
    pub path: PathBuf,
    file: File,
    pub len: u64,
}

impl Segment {
    pub fn open(dir: &PathBuf, id: usize) -> std::io::Result<Self> {
        let filename = format!("segment-{}.dat", id);
        let path = dir.join(filename);
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;
        let len = file.seek(SeekFrom::End(0))?;
        Ok(Segment { id, path, file, len })
    }

    /// Append a record: [key_len:u64][value_len:u64][key bytes][value bytes]
    /// Returns the offset at which the record was written (before the header).
    pub fn append(&mut self, key: &[u8], value: &[u8]) -> std::io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        let key_len = key.len() as u64;
        let value_len = value.len() as u64;
        self.file.write_all(&key_len.to_le_bytes())?;
        self.file.write_all(&value_len.to_le_bytes())?;
        self.file.write_all(key)?;
        self.file.write_all(value)?;
        self.file.flush()?;
        self.len = self.file.seek(SeekFrom::End(0))?;
        Ok(offset)
    }

    /// Read a value given an offset (must point to the start of a record)
    pub fn read_value_at(&mut self, offset: u64) -> std::io::Result<Option<Vec<u8>>> {
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf8 = [0u8; 8];
        // read key_len
        self.file.read_exact(&mut buf8)?;
        let key_len = u64::from_le_bytes(buf8);
        // read value_len
        self.file.read_exact(&mut buf8)?;
        let value_len = u64::from_le_bytes(buf8);
        // skip key
        let mut key_buf = vec![0u8; key_len as usize];
        self.file.read_exact(&mut key_buf)?;
        // read value
        if value_len == u64::MAX {
            // tombstone marker
            return Ok(None);
        } else {
            let mut val_buf = vec![0u8; value_len as usize];
            self.file.read_exact(&mut val_buf)?;
            return Ok(Some(val_buf));
        }
    }
}
