// mini-kvstore-v2/src/lib.rs

#[derive(Debug)]
pub struct Stats {
    pub num_keys: usize,
    // Add more fields as needed for your examples/tests, e.g. below.
    // pub num_segments: usize,
    // pub total_bytes: usize,
}

// Implement Display for Stats, fallback to Debug if needed
use std::fmt;
impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Minimal: match your println!("{}", stats)
        write!(f, "{:?}", self)
    }
}

pub struct KVStore {}

impl KVStore {
    /// Open a new or existing key-value store at the given path
    pub fn open(_path: &str) -> Result<Self, std::io::Error> {
        unimplemented!()
    }

    /// Set the value for a given key
    pub fn set(&mut self, _key: &str, _value: &[u8]) -> Result<(), std::io::Error> {
        unimplemented!()
    }

    /// Get the value for a given key
    pub fn get(&self, _key: &str) -> Option<Vec<u8>> {
        unimplemented!()
    }

    /// Delete a key (and its value) from the store
    pub fn delete(&mut self, _key: &str) -> Result<(), std::io::Error> {
        unimplemented!()
    }

    /// List all keys in the store
    pub fn list_keys(&self) -> Vec<String> {
        unimplemented!()
    }

    /// Compact the store to eliminate outdated entries
    pub fn compact(&mut self) -> Result<(), std::io::Error> {
        unimplemented!()
    }

    /// Get statistics about the store
    pub fn stats(&self) -> Stats {
        unimplemented!()
    }
}
