// Volume module: HTTP server for blob storage
pub mod handlers;
pub mod server;
pub mod storage;

pub use storage::BlobStorage;
