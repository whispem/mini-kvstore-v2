#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub data_dir: std::path::PathBuf,
    pub segment_size: u64,
    pub fsync_policy: FsyncPolicy,
}

#[derive(Debug, Clone, Copy)]
pub enum FsyncPolicy {
    Always,
    Never,
}

impl StoreConfig {
    pub fn new(data_dir: &std::path::Path) -> Self {
        StoreConfig {
            data_dir: data_dir.to_path_buf(),
            segment_size: 1024 * 1024, 
            fsync_policy: FsyncPolicy::Always,
        }
    }
}
