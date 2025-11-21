use crate::store::error::Result as StoreResult;
use crate::store::stats::StoreStats;
use crate::KVStore;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMeta {
    pub key: String,
    pub etag: String,
    pub size: u64,
    pub volume_id: String,
}

pub struct BlobStorage {
    store: KVStore,
    volume_id: String,
}

impl BlobStorage {
    pub fn new(data_dir: impl AsRef<Path>, volume_id: String) -> StoreResult<Self> {
        let store = KVStore::open(data_dir)?;
        Ok(BlobStorage { store, volume_id })
    }

    pub fn put(&mut self, key: &str, data: &[u8]) -> StoreResult<BlobMeta> {
        let etag = format!("{:08x}", crc32fast::hash(data));
        self.store.set(key, data)?;
        Ok(BlobMeta {
            key: key.to_string(),
            etag,
            size: data.len() as u64,
            volume_id: self.volume_id.clone(),
        })
    }

    pub fn get(&self, key: &str) -> StoreResult<Option<Vec<u8>>> {
        self.store.get(key)
    }

    pub fn delete(&mut self, key: &str) -> StoreResult<()> {
        self.store.delete(key)
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.store.list_keys()
    }

    pub fn volume_id(&self) -> &str {
        &self.volume_id
    }

    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }
}
