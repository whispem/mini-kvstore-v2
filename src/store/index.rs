//! In-memory index for fast key lookup.
//! Maps: key -> (segment_id, offset)

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub segment_id: u64,
    pub offset: u64,
}

#[derive(Debug, Default)]
pub struct Index {
    map: HashMap<String, IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert or update a key -> (segment_id, offset)
    pub fn insert(&mut self, key: String, segment_id: u64, offset: u64) {
        self.map.insert(key, IndexEntry { segment_id, offset });
    }

    /// Get a reference to the index entry for a key
    pub fn get(&self, key: &str) -> Option<&IndexEntry> {
        self.map.get(key)
    }

    /// Remove a key (tombstone or delete)
    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }
}
