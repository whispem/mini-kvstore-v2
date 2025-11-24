use std::fmt;

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Corrupted(String),
    CorruptedData(String),
    CompactionFailed(String),
    NotFound,
}

pub type Result<T> = std::result::Result<T, StoreError>;

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "IO error: {}", e),
            StoreError::Corrupted(msg) => write!(f, "Corrupted data: {}", msg),
            StoreError::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
            StoreError::CompactionFailed(msg) => write!(f, "Compaction failed: {}", msg),
            StoreError::NotFound => write!(f, "Not found"),
        }
    }
}

impl std::error::Error for StoreError {}
