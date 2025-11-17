use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum FsyncPolicy {

    Always,
    Never,
    Batch,
}

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub data_dir: PathBuf,
    pub segment_size: u64,
    pub fsync: FsyncPolicy,
    pub compaction_threshold: usize,
}

impl StoreConfig {

    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
            ..Self::default()
        }
    }

    pub fn default() -> Self {
        StoreConfig {
            data_dir: PathBuf::from("data"),
            segment_size: 1024 * 1024, 
            fsync: FsyncPolicy::Always,
            compaction_threshold: 3,
        }
    }
}
