//! Volume server startup and configuration

use crate::volume::{handlers, storage::BlobStorage};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Volume server configuration
#[derive(Debug, Clone)]
pub struct VolumeConfig {
    /// Unique volume identifier
    pub volume_id: String,
    /// Data directory for blob storage
    pub data_dir: PathBuf,
    /// Server bind address
    pub bind_addr: SocketAddr,
}

impl VolumeConfig {
    /// Creates a new volume configuration with defaults
    pub fn new(volume_id: String) -> Self {
        VolumeConfig {
            volume_id: volume_id.clone(),
            data_dir: PathBuf::from(format!("volume_data_{}", volume_id)),
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 9002)),
        }
    }

    /// Sets the data directory
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    /// Sets the bind address
    pub fn with_bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }
}

/// Starts the volume server with the given configuration
///
/// # Arguments
///
/// * `config` - Volume server configuration
///
/// # Examples
///
/// ```no_run
/// use mini_kvstore_v2::volume::server::{VolumeConfig, start_volume_server};
/// use std::net::SocketAddr;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = VolumeConfig::new("vol-1".to_string())
///         .with_bind_addr(SocketAddr::from(([127, 0, 0, 1], 9002)));
///     
///     start_volume_server(config).await?;
///     Ok(())
/// }
/// ```
pub async fn start_volume_server(config: VolumeConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;

    println!("=== Mini KVStore v2 - Volume Server ===");
    println!("Volume ID: {}", config.volume_id);
    println!("Data directory: {}", config.data_dir.display());
    println!("Listening on: http://{}", config.bind_addr);
    println!();

    // Initialize blob storage
    let storage = BlobStorage::new(&config.data_dir, config.volume_id.clone())?;
    let storage = Arc::new(Mutex::new(storage));

    // Create router
    let app = handlers::create_router(storage);

    // Start server
    println!("Volume server ready!");
    println!("Try:");
    println!(
        "  curl -X POST http://{}/blobs/test -d 'hello world'",
        config.bind_addr
    );
    println!("  curl http://{}/blobs/test", config.bind_addr);
    println!("  curl http://{}/health", config.bind_addr);
    println!();

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
