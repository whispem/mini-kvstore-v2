//! Blob storage wrapper around KVStore with metadata tracking

use crate::store::error::Result as StoreResult;
use crate::store::KVStore;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata for a stored blob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMeta {
    /// Unique key identifier
    pub key: String,
    /// Content hash (etag) for integrity checks
    pub etag: String,
    /// Size in bytes
    pub size: u64,
    /// Volume ID where this blob is stored
    pub volume_id: String,
}

/// Blob storage engine wrapping KVStore
pub struct BlobStorage {
    store: KVStore,
    volume_id: String,
}

impl BlobStorage {
    /// Creates a new BlobStorage instance
    ///
    /// # Arguments
    ///
    /// * `data_dir` - Directory for storing blob data
    /// * `volume_id` - Unique identifier for this volume
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::volume::BlobStorage;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let storage = BlobStorage::new("volume_data", "vol-1".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(data_dir: impl AsRef<Path>, volume_id: String) -> StoreResult<Self> {
        let store = KVStore::open(data_dir)?;
        Ok(BlobStorage { store, volume_id })
    }

    /// Stores a blob and returns its metadata
    ///
    /// # Arguments
    ///
    /// * `key` - Unique key for the blob
    /// * `data` - Blob data to store
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use mini_kvstore_v2::volume::BlobStorage;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut storage = BlobStorage::new("volume_data", "vol-1".to_string())?;
    /// let meta = storage.put("my-key", b"my data")?;
    /// println!("Stored blob with etag: {}", meta.etag);
    /// # Ok(())
    /// # }
    /// ```
    pub fn put(&mut self, key: &str, data: &[u8]) -> StoreResult<BlobMeta> {
        // Calculate etag (CRC32 hash of content)
        let etag = format!("{:08x}", crc32fast::hash(data));

        // Store in underlying KVStore
        self.store.set(key, data)?;

        Ok(BlobMeta {
            key: key.to_string(),
            etag,
            size: data.len() as u64,
            volume_id: self.volume_id.clone(),
        })
    }

    /// Retrieves a blob by key
    ///
    /// Returns `Ok(Some(data))` if found, `Ok(None)` if not found
    pub fn get(&mut self, key: &str) -> StoreResult<Option<Vec<u8>>> {
        self.store.get(key)
    }

    /// Deletes a blob by key
    ///
    /// This operation is idempotent - deleting a non-existent key succeeds
    pub fn delete(&mut self, key: &str) -> StoreResult<()> {
        self.store.delete(key)
    }

    /// Lists all blob keys in storage
    pub fn list_keys(&self) -> Vec<String> {
        self.store.list_keys()
    }

    /// Returns the volume ID
    pub fn volume_id(&self) -> &str {
        &self.volume_id
    }

    /// Returns storage statistics
    pub fn stats(&self) -> crate::store::stats::StoreStats {
        self.store.stats()
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
    fn test_blob_put_and_get() {
        let test_dir = "tests_data/volume_put_get";
        setup_test_dir(test_dir);

        let mut storage = BlobStorage::new(test_dir, "test-vol".to_string()).unwrap();

        let meta = storage.put("blob1", b"hello world").unwrap();
        assert_eq!(meta.key, "blob1");
        assert_eq!(meta.size, 11);
        assert_eq!(meta.volume_id, "test-vol");
        assert!(!meta.etag.is_empty());

        let data = storage.get("blob1").unwrap();
        assert_eq!(data, Some(b"hello world".to_vec()));

        let _ = remove_dir_all(test_dir);
    }

    #[test]
    fn test_blob_delete() {
        let test_dir = "tests_data/volume_delete";
        setup_test_dir(test_dir);

        let mut storage = BlobStorage::new(test_dir, "test-vol".to_string()).unwrap();

        storage.put("blob1", b"data").unwrap();
        assert!(storage.get("blob1").unwrap().is_some());

        storage.delete("blob1").unwrap();
        assert!(storage.get("blob1").unwrap().is_none());

        let _ = remove_dir_all(test_dir);
    }

    #[test]
    fn test_etag_consistency() {
        let test_dir = "tests_data/volume_etag";
        setup_test_dir(test_dir);

        let mut storage = BlobStorage::new(test_dir, "test-vol".to_string()).unwrap();

        let meta1 = storage.put("blob1", b"same data").unwrap();
        let meta2 = storage.put("blob2", b"same data").unwrap();

        // Same content should produce same etag
        assert_eq!(meta1.etag, meta2.etag);

        let _ = remove_dir_all(test_dir);
    }
}
