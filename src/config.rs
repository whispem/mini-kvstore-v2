//! Centralized configuration for mini-kvstore-v2

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub volume_id: String,
    pub data_dir: String,
    pub compaction_threshold: usize,
    pub compaction_interval_secs: u64,
    pub max_request_size_mb: usize,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "9002".to_string())
                .parse()
                .unwrap_or(9002),
            volume_id: env::var("VOLUME_ID").unwrap_or_else(|_| "vol-1".to_string()),
            data_dir: env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string()),
            compaction_threshold: env::var("COMPACTION_THRESHOLD")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            compaction_interval_secs: env::var("COMPACTION_INTERVAL_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            max_request_size_mb: env::var("MAX_REQUEST_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 9002,
            volume_id: "vol-1".to_string(),
            data_dir: "data".to_string(),
            compaction_threshold: 5,
            compaction_interval_secs: 60,
            max_request_size_mb: 100,
        }
    }
}
