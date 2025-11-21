use std::io;

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Corrupted data")]
    CorruptedData,
}

pub type Result<T> = std::result::Result<T, StoreError>;
