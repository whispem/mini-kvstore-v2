#![allow(dead_code)]
//! Store configuration options for mini-kvstore-v2.

/// Policy for how fsync is handled. Controls data durability.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum FsyncPolicy {
    /// Fsync after every write for maximum safety.
    #[default]
    Always,
    /// Fsync periodically at intervals.
    Interval,
    /// Never fsync (fast, not durable).
    Never,
}

impl FsyncPolicy {
    /// Returns a human-readable description.
    pub fn as_str(&self) -> &'static str {
        match self {
            FsyncPolicy::Always => "fsync after every write",
            FsyncPolicy::Interval => "fsync at intervals",
            FsyncPolicy::Never => "never fsync",
        }
    }
}

/// Complete store configuration with typical options.
#[allow(dead_code)]
#[derive(Debug)]
pub struct StoreConfig {
    pub fsync_policy: FsyncPolicy,
    pub max_segment_size: u64,
    pub enable_checksums: bool,
    pub data_path: String,
    pub cache_segments: usize,
    pub verbose_logging: bool,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            fsync_policy: FsyncPolicy::default(),
            max_segment_size: 16 * 1024 * 1024, // 16 MB
            enable_checksums: true,
            data_path: "data".to_string(),
            cache_segments: 4,
            verbose_logging: false,
        }
    }
}

impl StoreConfig {
    /// Returns a config suitable for tests.
    #[allow(dead_code)]
    pub fn test_config() -> Self {
        Self {
            fsync_policy: FsyncPolicy::Never,
            max_segment_size: 512 * 1024,
            enable_checksums: false,
            data_path: "tests_data/temp".to_string(),
            cache_segments: 1,
            verbose_logging: false,
        }
    }

    /// Display summary for debugging/logging.
    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        format!(
            "StoreConfig: fsync_policy={}, max_segment_size={} bytes, checksums={}, data_path={}, cache_segments={}, verbose_logging={}",
            self.fsync_policy.as_str(),
            self.max_segment_size,
            self.enable_checksums,
            self.data_path,
            self.cache_segments,
            self.verbose_logging
        )
    }
}
