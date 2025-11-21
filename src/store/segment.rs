//! Segment logic for mini-kvstore-v2.
// Clippy warnings silenced (including type complexity).

pub type SegmentReadResult = Result<Option<(String, Option<Vec<u8>>)>, std::io::Error>;

#[allow(dead_code)]
const SEGMENT_SIZE_LIMIT: u64 = 1024 * 1024;

#[allow(dead_code)]
pub struct Segment {
    pub path: std::path::PathBuf,
    pub id: usize, // Unused field allowed for Clippy
                   // TODO: Add other fields if necessary.
}

#[allow(dead_code)]
impl Segment {
    /// Opens a segment (stub implementation).
    pub fn open(_dir: &std::path::Path, _id: usize) -> Result<Self, std::io::Error> {
        Ok(Segment {
            path: _dir.join(format!("segment-{:04}.dat", _id)),
            id: _id,
        })
    }

    /// Appends a key-value pair to the segment (stub).
    pub fn append(&mut self, _key: &[u8], _value: &[u8]) -> Result<u64, std::io::Error> {
        Ok(0)
    }

    /// Appends a tombstone (delete marker) for a key.
    pub fn append_tombstone(&mut self, _key: &[u8]) -> Result<u64, std::io::Error> {
        Ok(0)
    }

    /// Checks if the segment is full (stub).
    pub fn is_full(&self) -> bool {
        false
    }

    /// Reads a key/value record at the given offset (stub).
    pub fn read_record_at(&mut self, _offset: u64) -> SegmentReadResult {
        Ok(None)
    }

    /// Reads a value at a given offset (stub).
    pub fn read_value_at(&mut self, _offset: u64) -> Result<Option<Vec<u8>>, std::io::Error> {
        Ok(None)
    }

    /// Computes the record size for a key/value (stub).
    pub fn record_size(_key_len: u64, _value_len: u64) -> u64 {
        0
    }
}
