//! In-memory index for fast key lookups.

use std::collections::HashMap;

/// In-memory index mapping keys to their location in segments.
///
/// Each entry stores: `(segment_id, offset, value_length)`
#[derive(Debug, Default)]
pub struct Index {
    pub(crate) map: HashMap<String, (usize, u64, u64)>,
}

impl Index {
    /// Creates a new empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or updates an entry in the index.
    #[inline]
    pub fn insert(&mut self, key: String, seg_id: usize, offset: u64, len: u64) {
        self.map.insert(key, (seg_id, offset, len));
    }

    /// Gets the location of a key.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&(usize, u64, u64)> {
        self.map.get(key)
    }

    /// Removes a key from the index.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<(usize, u64, u64)> {
        self.map.remove(key)
    }

    /// Returns the number of keys in the index.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the index is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns an iterator over the keys.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Returns true if the index contains the key.
    #[inline]
    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Clears all entries from the index.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_basic_operations() {
        let mut index = Index::new();

        assert!(index.is_empty());

        index.insert("key1".to_string(), 0, 100, 50);
        index.insert("key2".to_string(), 1, 200, 75);

        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());

        assert_eq!(index.get("key1"), Some(&(0, 100, 50)));
        assert_eq!(index.get("key2"), Some(&(1, 200, 75)));
        assert_eq!(index.get("key3"), None);

        assert!(index.contains("key1"));
        assert!(!index.contains("key3"));
    }

    #[test]
    fn test_index_update() {
        let mut index = Index::new();

        index.insert("key".to_string(), 0, 100, 50);
        index.insert("key".to_string(), 1, 200, 75);

        assert_eq!(index.len(), 1);
        assert_eq!(index.get("key"), Some(&(1, 200, 75)));
    }

    #[test]
    fn test_index_remove() {
        let mut index = Index::new();

        index.insert("key".to_string(), 0, 100, 50);
        let removed = index.remove("key");

        assert_eq!(removed, Some((0, 100, 50)));
        assert!(index.is_empty());
        assert_eq!(index.remove("key"), None);
    }

    #[test]
    fn test_index_clear() {
        let mut index = Index::new();

        index.insert("key1".to_string(), 0, 100, 50);
        index.insert("key2".to_string(), 1, 200, 75);

        index.clear();

        assert!(index.is_empty());
    }
}
