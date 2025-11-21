// mini-kvstore-v2/src/lib.rs

use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Stats {
    pub num_keys: usize,
    pub num_segments: usize,
    pub total_bytes: usize,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stats {{ keys: {}, segments: {}, bytes: {} }}",
            self.num_keys, self.num_segments, self.total_bytes
        )
    }
}

impl Stats {
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }
}

pub struct KVStore {
    path: PathBuf,
    data: HashMap<String, Vec<u8>>,
    current_segment: usize,
}

impl KVStore {
    /// Open a new or existing key-value store at the given path
    pub fn open(path: &str) -> Result<Self, std::io::Error> {
        let path = PathBuf::from(path);
        
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        let mut store = KVStore {
            path,
            data: HashMap::new(),
            current_segment: 0,
        };

        // Load existing data (simplified implementation)
        store.load_existing_data()?;
        
        Ok(store)
    }

    fn load_existing_data(&mut self) -> Result<(), std::io::Error> {
        // Simplified: scan for segment files and load them
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "seg") {
                self.load_segment(&path)?;
                self.current_segment += 1;
            }
        }
        Ok(())
    }

    fn load_segment(&mut self, path: &Path) -> Result<(), std::io::Error> {
        // Simplified segment loading
        if let Ok(mut file) = File::open(path) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            
            // Parse key-value pairs (simplified format)
            for line in content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    self.data.insert(key.to_string(), value.as_bytes().to_vec());
                }
            }
        }
        Ok(())
    }

    /// Set the value for a given key
    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<(), std::io::Error> {
        self.data.insert(key.to_string(), value.to_vec());
        
        // Write to current segment file
        let segment_path = self.path.join(format!("segment_{}.seg", self.current_segment));
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(segment_path)?;
            
        writeln!(file, "{}:{}", key, String::from_utf8_lossy(value))?;
        
        Ok(())
    }

    /// Get the value for a given key
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    /// Delete a key (and its value) from the store
    pub fn delete(&mut self, key: &str) -> Result<(), std::io::Error> {
        self.data.remove(key);
        
        // Mark as deleted in segment (simplified)
        let segment_path = self.path.join(format!("segment_{}.seg", self.current_segment));
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(segment_path)?;
            
        writeln!(file, "{}:<deleted>", key)?;
        
        Ok(())
    }

    /// List all keys in the store
    pub fn list_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Compact the store to eliminate outdated entries
    pub fn compact(&mut self) -> Result<(), std::io::Error> {
        // Create new segment with only current data
        self.current_segment += 1;
        let new_segment_path = self.path.join(format!("segment_{}.seg", self.current_segment));
        let mut file = File::create(new_segment_path)?;
        
        for (key, value) in &self.data {
            writeln!(file, "{}:{}", key, String::from_utf8_lossy(value))?;
        }
        
        // Remove old segments (simplified - in reality you'd keep track of which to delete)
        for i in 0..self.current_segment {
            let old_path = self.path.join(format!("segment_{}.seg", i));
            let _ = fs::remove_file(old_path); // Ignore errors for non-existent files
        }
        
        Ok(())
    }

    /// Get statistics about the store
    pub fn stats(&self) -> Stats {
        let total_bytes = self.data.values().map(|v| v.len()).sum();
        
        Stats {
            num_keys: self.data.len(),
            num_segments: self.current_segment + 1,
            total_bytes,
        }
    }
}

// Add HTTP server functionality for volume module
pub mod volume {
    pub mod server {
        use super::super::KVStore;
        use axum::{
            extract::{Path, State},
            http::StatusCode,
            response::Json,
            routing::{get, post},
            Router,
        };
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        use std::sync::Arc;
        use tokio::sync::RwLock;

        #[derive(Clone)]
        pub struct VolumeConfig {
            pub volume_id: String,
            pub data_dir: String,
            pub bind_addr: std::net::SocketAddr,
        }

        impl VolumeConfig {
            pub fn new(volume_id: String) -> Self {
                Self {
                    volume_id,
                    data_dir: "volume_data".to_string(),
                    bind_addr: ([127, 0, 0, 1], 9002).into(),
                }
            }

            pub fn with_data_dir(mut self, data_dir: String) -> Self {
                self.data_dir = data_dir;
                self
            }

            pub fn with_bind_addr(mut self, bind_addr: std::net::SocketAddr) -> Self {
                self.bind_addr = bind_addr;
                self
            }
        }

        #[derive(Serialize, Deserialize)]
        pub struct KeyValue {
            pub key: String,
            pub value: String,
        }

        pub struct VolumeServer {
            pub store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
        }

        impl VolumeServer {
            pub fn new() -> Self {
                Self {
                    store: Arc::new(RwLock::new(HashMap::new())),
                }
            }

            pub async fn start(self, config: VolumeConfig) -> Result<(), Box<dyn std::error::Error>> {
                let app = Router::new()
                    .route("/health", get(|| async { "OK" }))
                    .route("/keys/:key", get(get_key))
                    .route("/keys/:key", post(set_key))
                    .route("/keys/:key", axum::routing::delete(delete_key))
                    .with_state(self.store);

                let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
                println!("Volume server listening on {}", config.bind_addr);
                
                axum::serve(listener, app).await?;
                Ok(())
            }
        }

        async fn get_key(
            Path(key): Path<String>,
            State(store): State<Arc<RwLock<HashMap<String, Vec<u8>>>>>,
        ) -> Result<Json<KeyValue>, StatusCode> {
            let store = store.read().await;
            if let Some(value) = store.get(&key) {
                Ok(Json(KeyValue {
                    key,
                    value: String::from_utf8_lossy(value).to_string(),
                }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }

        async fn set_key(
            Path(key): Path<String>,
            State(store): State<Arc<RwLock<HashMap<String, Vec<u8>>>>>,
            Json(kv): Json<KeyValue>,
        ) -> StatusCode {
            let mut store = store.write().await;
            store.insert(key, kv.value.into_bytes());
            StatusCode::OK
        }

        async fn delete_key(
            Path(key): Path<String>,
            State(store): State<Arc<RwLock<HashMap<String, Vec<u8>>>>>,
        ) -> StatusCode {
            let mut store = store.write().await;
            if store.remove(&key).is_some() {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        }

        pub async fn start_volume_server(config: VolumeConfig) -> Result<(), Box<dyn std::error::Error>> {
            let server = VolumeServer::new();
            server.start(config).await
        }
    }
}
