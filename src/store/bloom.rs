use bloomfilter::Bloom;
use sha2::{Digest, Sha256};

pub struct BloomIndex {
    pub bloom: Bloom<[u8; 32]>,
}

impl BloomIndex {
    pub fn new(expected_items: usize) -> Self {
        // Bitmap size = expected_items * 10 (rule of thumb)
        let bitmap_size = expected_items * 10;
        let bloom = Bloom::new(bitmap_size, expected_items).expect("failed to create bloom filter");

        Self { bloom }
    }

    pub fn insert(&mut self, key: &str) {
        self.bloom.set(&self.hash_key(key));
    }

    pub fn might_contain(&self, key: &str) -> bool {
        self.bloom.check(&self.hash_key(key))
    }

    fn hash_key(&self, key: &str) -> [u8; 32] {
        let hash = Sha256::digest(key.as_bytes());
        let mut out = [0u8; 32];
        out.copy_from_slice(&hash);
        out
    }
}
