//! Configuration for the KVStore.

use std::path::PathBuf;

/// Fsync policy for durability guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsyncPolicy {
    /// Fsync after every write (safest, slowest).
    Always,
    /// Never fsync (fastest, risk of data loss).
    Never,
    /// Fsync periodically in batches.
    Batch,
}

impl Default for FsyncPolicy {
    fn default() -> Self {
        Self::Always
    }
}

/// Configuration options for the KVStore.
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Directory where segment files are stored.
    pub data_dir: PathBuf,
    /// Maximum size of each segment file in bytes.
    pub segment_size: u64,
    /// Fsync policy for writes.
    pub fsync: FsyncPolicy,
    /// Number of segments that trigger automatic compaction suggestion.
    pub compaction_threshold: usize,
}

impl StoreConfig {
    /// Creates a new configuration with the given data directory.
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        StoreConfig {
            data_dir: data_dir.into(),
            segment_size: 1024 * 1024, // 1 MB
            fsync: FsyncPolicy::Always,
            compaction_threshold: 3,
        }
    }

    /// Sets the segment size limit.
    pub fn with_segment_size(mut self, size: u64) -> Self {
        self.segment_size = size;
        self
    }

    /// Sets the fsync policy.
    pub fn with_fsync(mut self, policy: FsyncPolicy) -> Self {
        self.fsync = policy;
        self
    }

    /// Sets the compaction threshold.
    pub fn with_compaction_threshold(mut self, threshold: usize) -> Self {
        self.compaction_threshold = threshold;
        self
    }
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self::new("data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = StoreConfig::default();
        assert_eq!(config.data_dir, PathBuf::from("data"));
        assert_eq!(config.segment_size, 1024 * 1024);
        assert_eq!(config.fsync, FsyncPolicy::Always);
        assert_eq!(config.compaction_threshold, 3);
    }

    #[test]
    fn test_config_builder() {
        let config = StoreConfig::new("/tmp/test")
            .with_segment_size(512 * 1024)
            .with_fsync(FsyncPolicy::Batch)
            .with_compaction_threshold(5);

        assert_eq!(config.data_dir, PathBuf::from("/tmp/test"));
        assert_eq!(config.segment_size, 512 * 1024);
        assert_eq!(config.fsync, FsyncPolicy::Batch);
        assert_eq!(config.compaction_threshold, 5);
    }
}
