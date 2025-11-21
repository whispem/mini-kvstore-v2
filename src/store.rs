//! Storage engine module.
//!
//! This module contains the core KVStore implementation with:
//! - Segmented append-only log storage
//! - In-memory index for fast lookups
//! - CRC32 checksums for data integrity
//! - Manual compaction for space reclamation

pub mod compaction;
pub mod config;
mod engine;
pub mod error;
mod index;
mod segment;
pub mod stats;

pub use engine::KVStore;
