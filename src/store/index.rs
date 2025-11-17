pub struct Index {
    pub map: std::collections::HashMap<String, (usize, u64, u64)>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            map: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, seg_id: usize, offset: u64, len: u64) {
        self.map.insert(key, (seg_id, offset, len));
    }

    pub fn get(&self, key: &str) -> Option<&(usize, u64, u64)> {
        self.map.get(key)
    }

    pub fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}
