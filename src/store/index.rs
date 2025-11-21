//! In-memory index for KVStore.
// All unused code annotated for Clippy compliance.

#[allow(dead_code)]
pub struct Index {
    /// Map: key -> (segment_id, offset, length)
    map: std::collections::HashMap<String, (usize, u64, u64)>,
}

#[allow(dead_code)]
impl Index {
    /// Creates a new empty index.
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }

    /// Inserts a key and its segment location.
    pub fn insert(&mut self, key: String, seg_id: usize, offset: u64, len: u64) {
        self.map.insert(key, (seg_id, offset, len));
    }

    /// Gets the segment info for a key.
    pub fn get(&self, key: &str) -> Option<&(usize, u64, u64)> {
        self.map.get(key)
    }

    /// Removes a key and returns its segment info.
    pub fn remove(&mut self, key: &str) -> Option<(usize, u64, u64)> {
        self.map.remove(key)
    }

    /// Returns the number of keys stored.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Checks if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns an iterator over the index's keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Checks if the index contains a key.
    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Clears all entries from the index.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}
