//! Volume module: HTTP server for blob storage.
//!
//! This module provides an HTTP API on top of the KVStore,
//! allowing blob storage operations via REST endpoints.

pub mod handlers;
pub mod server;
pub mod storage;

pub use server::{start_volume_server, VolumeConfig};
pub use storage::BlobStorage;
