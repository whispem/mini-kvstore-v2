//! In-memory index for KVStore.
// Unused code annotated for Clippy compliance.

#[allow(dead_code)]
pub struct Index {
    /// Map: key -> (segment_id, offset, length)
    map: std::collections::HashMap<String, (usize, u64, u64)>,
}

#[allow(dead_code)]
impl Index {
    pub fn new() -> Self {
        Self {
            map: std::collections::HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: String, seg_id: usize, offset: u64, len: u64) {
        self.map.insert(key, (seg_id, offset, len));
    }
    pub fn get(&self, key: &str) -> Option<&(usize, u64, u64)> {
        self.map.get(key)
    }
    pub fn remove(&mut self, key: &str) -> Option<(usize, u64, u64)> {
        self.map.remove(key)
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
    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    pub fn clear(&mut self) {
        self.map.clear();
    }
}


impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}
