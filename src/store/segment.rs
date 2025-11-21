//! Segment file management for append-only log storage.

use crate::store::error::{Result, StoreError};
use crc32fast::Hasher;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Maximum size of a segment file before rotation (1 MB).
const SEGMENT_SIZE_LIMIT: u64 = 1024 * 1024;

/// A single segment file in the append-only log.
///
/// Each segment stores records in the format:
/// `[key_len: u64][value_len: u64][checksum: u32][key bytes][value bytes]`
///
/// Tombstones use `value_len = u64::MAX` and only store the key.
pub struct Segment {
    file: File,
    /// Current length of the segment file in bytes.
    pub len: u64,
    /// Segment identifier.
    pub id: usize,
}

impl Segment {
    /// Opens or creates a segment file.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory where segment files are stored
    /// * `id` - Unique segment identifier
    pub fn open(dir: &Path, id: usize) -> Result<Self> {
        let path = dir.join(format!("segment-{:04}.dat", id));
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;
        let len = file.seek(SeekFrom::End(0))?;
        Ok(Segment { file, len, id })
    }

    /// Appends a key-value record to the segment.
    ///
    /// Returns the offset where the record was written.
    pub fn append(&mut self, key: &[u8], value: &[u8]) -> Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;

        // Compute checksum over key + value
        let mut hasher = Hasher::new();
        hasher.update(key);
        hasher.update(value);
        let checksum = hasher.finalize();

        // Write header: key_len (8) + value_len (8) + checksum (4)
        self.file.write_all(&(key.len() as u64).to_le_bytes())?;
        self.file.write_all(&(value.len() as u64).to_le_bytes())?;
        self.file.write_all(&checksum.to_le_bytes())?;

        // Write data
        self.file.write_all(key)?;
        self.file.write_all(value)?;
        self.file.sync_all()?;

        self.len = self.file.seek(SeekFrom::End(0))?;
        Ok(offset)
    }

    /// Appends a tombstone record (deletion marker) for a key.
    ///
    /// Returns the offset where the tombstone was written.
    pub fn append_tombstone(&mut self, key: &[u8]) -> Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;

        // Tombstone: checksum only covers key
        let checksum = crc32fast::hash(key);

        self.file.write_all(&(key.len() as u64).to_le_bytes())?;
        self.file.write_all(&u64::MAX.to_le_bytes())?; // Tombstone marker
        self.file.write_all(&checksum.to_le_bytes())?;
        self.file.write_all(key)?;
        self.file.sync_all()?;

        self.len = self.file.seek(SeekFrom::End(0))?;
        Ok(offset)
    }

    /// Returns true if the segment has reached its size limit.
    pub fn is_full(&self) -> bool {
        self.len >= SEGMENT_SIZE_LIMIT
    }

    /// Reads a record at the given offset.
    ///
    /// Returns `Ok(Some((key, Some(value))))` for a normal record,
    /// `Ok(Some((key, None)))` for a tombstone, or `Ok(None)` if EOF.
    pub fn read_record_at(&mut self, offset: u64) -> Result<Option<(String, Option<Vec<u8>>)>> {
        self.file.seek(SeekFrom::Start(offset))?;

        // Read header: key_len (8) + value_len (8) + checksum (4) = 20 bytes
        let mut header = [0u8; 20];
        match self.file.read_exact(&mut header) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let key_len = u64::from_le_bytes(header[0..8].try_into().unwrap());
        let value_len = u64::from_le_bytes(header[8..16].try_into().unwrap());
        let stored_checksum = u32::from_le_bytes(header[16..20].try_into().unwrap());

        // Read key
        let mut key_buf = vec![0u8; key_len as usize];
        self.file.read_exact(&mut key_buf)?;
        let key = String::from_utf8_lossy(&key_buf).to_string();

        if value_len == u64::MAX {
            // Tombstone - validate key-only checksum
            let computed = crc32fast::hash(&key_buf);
            if computed != stored_checksum {
                return Err(StoreError::ChecksumMismatch {
                    offset,
                    expected: stored_checksum,
                    computed,
                });
            }
            Ok(Some((key, None)))
        } else {
            // Normal record - read value and validate
            let mut val_buf = vec![0u8; value_len as usize];
            self.file.read_exact(&mut val_buf)?;

            let mut hasher = Hasher::new();
            hasher.update(&key_buf);
            hasher.update(&val_buf);
            let computed = hasher.finalize();

            if computed != stored_checksum {
                return Err(StoreError::ChecksumMismatch {
                    offset,
                    expected: stored_checksum,
                    computed,
                });
            }

            Ok(Some((key, Some(val_buf))))
        }
    }

    /// Reads only the value at the given offset.
    ///
    /// Returns `Ok(Some(value))` for a normal record,
    /// `Ok(None)` for a tombstone.
    pub fn read_value_at(&mut self, offset: u64) -> Result<Option<Vec<u8>>> {
        match self.read_record_at(offset)? {
            Some((_, value)) => Ok(value),
            None => Ok(None),
        }
    }

    /// Returns the size of a record given key and value lengths.
    ///
    /// Used for calculating offsets during index rebuild.
    pub fn record_size(key_len: u64, value_len: u64) -> u64 {
        // header (20) + key + value (or 0 for tombstone)
        20 + key_len + if value_len == u64::MAX { 0 } else { value_len }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all};

    fn setup_test_dir(path: &str) {
        let _ = remove_dir_all(path);
        create_dir_all(path).expect("Failed to create test directory");
    }

    #[test]
    fn test_segment_append_and_read() {
        let test_dir = "tests_data/segment_append_read";
        setup_test_dir(test_dir);

        let mut seg = Segment::open(Path::new(test_dir), 0).unwrap();

        let offset = seg.append(b"hello", b"world").unwrap();
        assert_eq!(offset, 0);

        let record = seg.read_record_at(0).unwrap();
        assert!(record.is_some());
        let (key, value) = record.unwrap();
        assert_eq!(key, "hello");
        assert_eq!(value, Some(b"world".to_vec()));

        let _ = remove_dir_all(test_dir);
    }

    #[test]
    fn test_segment_tombstone() {
        let test_dir = "tests_data/segment_tombstone";
        setup_test_dir(test_dir);

        let mut seg = Segment::open(Path::new(test_dir), 0).unwrap();

        seg.append_tombstone(b"deleted_key").unwrap();

        let record = seg.read_record_at(0).unwrap();
        assert!(record.is_some());
        let (key, value) = record.unwrap();
        assert_eq!(key, "deleted_key");
        assert!(value.is_none());

        let _ = remove_dir_all(test_dir);
    }

    #[test]
    fn test_checksum_validation() {
        let test_dir = "tests_data/segment_checksum";
        setup_test_dir(test_dir);

        let mut seg = Segment::open(Path::new(test_dir), 0).unwrap();
        seg.append(b"key", b"value").unwrap();

        // Reading should succeed with valid checksum
        let result = seg.read_record_at(0);
        assert!(result.is_ok());

        let _ = remove_dir_all(test_dir);
    }
}
