pub mod compaction;
pub mod config;
mod engine;
pub mod error;
mod index;
mod record;
mod segment;
mod stats;

pub use engine::KVStore;
pub use error::{Result, StoreError};