use crate::store::config::StoreConfig;
use crate::store::error::{Result, StoreError};
use crate::store::index::Index;
use crate::store::segment::Segment;
use crate::store::stats::StoreStats;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A key-value store with segmented append-only log storage.
///
/// `KVStore` provides a simple persistent key-value store that uses an append-only
/// log structure with multiple segment files. It maintains an in-memory index for
/// fast lookups and supports compaction to reclaim disk space.
///
/// # Examples
///
/// ```no_run
/// use mini_kvstore_v2::KVStore;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut store = KVStore::open("my_data")?;
///
/// // Set a key-value pair
/// store.set("hello", b"world")?;
///
/// // Get a value
/// assert_eq!(store.get("hello")?, Some(b"world".to_vec()));
///
/// // Delete a key
/// store.delete("hello")?;
/// assert_eq!(store.get("hello")?, None);
/// # Ok(())
/// # }
/// ```
pub struct KVStore {
    pub config: StoreConfig,
    pub segments: HashMap<usize, Segment>,
    pub index: Index,
    pub active_id: usize,
}

impl KVStore {
    /// Opens a key-value store at the specified directory with default configuration.
    ///
    /// If the directory doesn't exist, it will be created. If segment files exist
    /// in the directory, they will be loaded and the in-memory index will be rebuilt.
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the directory where segment files will be stored
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut store = KVStore::open("data")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory cannot be created
    /// - Existing segment files cannot be read
    /// - The index cannot be rebuilt
    pub fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let config = StoreConfig::new(dir.as_ref());
        Self::open_with_config(config)
    }

    /// Opens a key-value store with a custom configuration.
    ///
    /// This allows fine-grained control over store behavior including segment size,
    /// fsync policy, and compaction thresholds.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom store configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    /// # use mini_kvstore_v2::store::config::StoreConfig;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = StoreConfig::new("custom_data");
    /// let mut store = KVStore::open_with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_config(config: StoreConfig) -> Result<Self> {
        fs::create_dir_all(&config.data_dir)?;

        let mut ids: Vec<usize> = fs::read_dir(&config.data_dir)?
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("segment-") && fname.ends_with(".dat") {
                    fname["segment-".len()..fname.len() - 4].parse().ok()
                } else {
                    None
                }
            })
            .collect();
        ids.sort_unstable();

        let active_id = ids.last().copied().unwrap_or(0);

        let mut segments = HashMap::new();
        for &id in &ids {
            let seg = Segment::open(&config.data_dir, id)?;
            segments.insert(id, seg);
        }

        if let std::collections::hash_map::Entry::Vacant(e) = segments.entry(active_id) {
            let seg = Segment::open(&config.data_dir, active_id)?;
            e.insert(seg);
        }

        let mut store = KVStore {
            config,
            segments,
            index: Index::new(),
            active_id,
        };

        store.rebuild_index()?;

        Ok(store)
    }

    /// Rebuild in-memory index from all segments
    fn rebuild_index(&mut self) -> Result<()> {
        let mut ordered_ids: Vec<usize> = self.segments.keys().copied().collect();
        ordered_ids.sort_unstable();

        for id in ordered_ids {
            let seg = self
                .segments
                .get_mut(&id)
                .ok_or(StoreError::SegmentDisappeared)?;

            let mut pos = 0u64;
            while pos < seg.len {
                match seg.read_record_at(pos) {
                    Ok(Some((key, value_opt))) => {
                        if let Some(ref value) = value_opt {
                            self.index.insert(key.clone(), id, pos, value.len() as u64);
                        } else {
                            self.index.remove(&key);
                        }

                        let key_bytes = key.as_bytes();
                        let record_size = 8
                            + 8
                            + key_bytes.len() as u64
                            + value_opt.as_ref().map(|v| v.len() as u64).unwrap_or(0);
                        pos += record_size;
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to read record at position {} in segment {}: {}",
                            pos, id, e
                        );
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Sets a key-value pair in the store.
    ///
    /// If the key already exists, its value will be updated. The operation is
    /// immediately persisted to disk (with fsync) before returning.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    /// * `value` - The value to associate with the key
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = KVStore::open("example_data")?;
    /// store.set("user:1", b"Alice")?;
    /// store.set("user:2", b"Bob")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The active segment cannot be written to
    /// - A new segment cannot be created when rotation is needed
    /// - The fsync operation fails
    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let active_seg = self
            .segments
            .get_mut(&self.active_id)
            .ok_or(StoreError::ActiveSegmentNotFound)?;

        if active_seg.is_full() {
            self.active_id += 1;
            let new_seg = Segment::open(&self.config.data_dir, self.active_id)?;
            self.segments.insert(self.active_id, new_seg);
        }

        let active_seg = self
            .segments
            .get_mut(&self.active_id)
            .ok_or(StoreError::ActiveSegmentNotFound)?;

        let offset = active_seg.append(key.as_bytes(), value)?;
        self.index
            .insert(key.to_string(), self.active_id, offset, value.len() as u64);

        Ok(())
    }

    /// Retrieves the value associated with a key.
    ///
    /// Returns `Ok(Some(value))` if the key exists, `Ok(None)` if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = KVStore::open("example_data")?;
    /// store.set("greeting", b"Hello, World!")?;
    ///
    /// match store.get("greeting")? {
    ///     Some(value) => println!("Found: {}", String::from_utf8_lossy(&value)),
    ///     None => println!("Key not found"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the segment file cannot be read.
    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(&(seg_id, offset, _len)) = self.index.get(key) {
            let seg = self
                .segments
                .get_mut(&seg_id)
                .ok_or(StoreError::SegmentNotFound(seg_id))?;

            seg.read_value_at(offset).map_err(StoreError::from)
        } else {
            Ok(None)
        }
    }

    /// Deletes a key from the store.
    ///
    /// This operation appends a tombstone record to the log and removes the key
    /// from the in-memory index. The space is not immediately reclaimed; use
    /// `compact()` to reclaim space from deleted keys.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = KVStore::open("example_data")?;
    /// store.set("temp", b"data")?;
    /// store.delete("temp")?;
    /// assert_eq!(store.get("temp")?, None);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the tombstone cannot be written to the active segment.
    pub fn delete(&mut self, key: &str) -> Result<()> {
        if !self.index.map.contains_key(key) {
            return Ok(());
        }

        let active_seg = self
            .segments
            .get_mut(&self.active_id)
            .ok_or(StoreError::ActiveSegmentNotFound)?;

        if active_seg.is_full() {
            self.active_id += 1;
            let new_seg = Segment::open(&self.config.data_dir, self.active_id)?;
            self.segments.insert(self.active_id, new_seg);
        }

        let active_seg = self
            .segments
            .get_mut(&self.active_id)
            .ok_or(StoreError::ActiveSegmentNotFound)?;

        active_seg.append_tombstone(key.as_bytes())?;
        self.index.remove(key);

        Ok(())
    }

    /// Returns a list of all keys currently in the store.
    ///
    /// This operation is fast as it only reads from the in-memory index.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = KVStore::open("example_data")?;
    /// store.set("a", b"1")?;
    /// store.set("b", b"2")?;
    ///
    /// let keys = store.list_keys();
    /// assert_eq!(keys.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_keys(&self) -> Vec<String> {
        self.index.map.keys().cloned().collect()
    }

    /// Returns statistics about the store.
    ///
    /// Provides information about the number of keys, segments, total disk usage,
    /// and segment IDs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = KVStore::open("example_data")?;
    /// let stats = store.stats();
    /// println!("Keys: {}", stats.num_keys);
    /// println!("Segments: {}", stats.num_segments);
    /// println!("Total size: {:.2} MB", stats.total_mb());
    /// # Ok(())
    /// # }
    /// ```
    pub fn stats(&self) -> StoreStats {
        let num_keys = self.index.len();
        let num_segments = self.segments.len();

        let total_bytes: u64 = self.segments.values().map(|s| s.len).sum();

        let oldest_segment_id = self.segments.keys().copied().min().unwrap_or(0);

        StoreStats {
            num_keys,
            num_segments,
            total_bytes,
            active_segment_id: self.active_id,
            oldest_segment_id,
        }
    }

    /// Performs compaction on the store.
    ///
    /// Compaction rewrites all live keys into new segments and removes old segments,
    /// reclaiming space from deleted keys and old values. This operation may take
    /// significant time for large stores.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mini_kvstore_v2::KVStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut store = KVStore::open("example_data")?;
    /// // Write and overwrite many keys
    /// for i in 0..1000 {
    ///     store.set("key", format!("value_{}", i).as_bytes())?;
    /// }
    ///
    /// // Reclaim space
    /// store.compact()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Segments cannot be read
    /// - New segments cannot be created
    /// - Old segments cannot be deleted
    pub fn compact(&mut self) -> Result<()> {
        crate::store::compaction::compact_segments(self)
    }
}
