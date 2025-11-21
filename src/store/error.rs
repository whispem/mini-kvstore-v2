use std::io;

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Corrupted data: {0}")]
    CorruptedData(String),

    #[error("Compaction failed: {0}")]
    CompactionFailed(String),
}

pub type Result<T> = std::result::Result<T, StoreError>;
