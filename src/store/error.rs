//! Error types for KVStore operations.

use std::io;
use thiserror::Error;

/// Errors that can occur during KVStore operations.
#[derive(Error, Debug)]
pub enum StoreError {
    /// The active segment was not found when it should exist.
    #[error("Active segment not found")]
    ActiveSegmentNotFound,

    /// A specific segment was not found.
    #[error("Segment {0} not found")]
    SegmentNotFound(usize),

    /// A segment disappeared during index rebuild.
    #[error("Segment disappeared during rebuild")]
    SegmentDisappeared,

    /// Checksum validation failed during read.
    #[error("Checksum mismatch at offset {offset}: expected {expected:08x}, got {computed:08x}")]
    ChecksumMismatch {
        offset: u64,
        expected: u32,
        computed: u32,
    },

    /// Compaction operation failed.
    #[error("Compaction failed: {0}")]
    CompactionFailed(String),

    /// An I/O error occurred.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Result type alias for KVStore operations.
pub type Result<T> = std::result::Result<T, StoreError>;
