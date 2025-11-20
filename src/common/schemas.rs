//! Shared types for mini-kvstore-v2 Coordinator/Volume.

#[derive(Debug, Clone)]
pub struct KeyMeta {
    /// Key string identifier
    pub key: String,
    /// Unique content hash (etag)
    pub etag: String,
    /// Value size in bytes
    pub size: u64,
    /// List of volume IDs storing this key
    pub replicas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VolumeInfo {
    /// Volume ID
    pub id: String,
    /// Public or accessible volume URL
    pub url: String,
    /// Health/status (e.g. "Alive")
    pub status: VolumeStatus,
}

#[derive(Debug, Clone)]
pub enum VolumeStatus {
    Alive,
    Down,
    Unknown,
}

impl std::fmt::Display for VolumeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VolumeStatus::Alive => write!(f, "Alive"),
            VolumeStatus::Down => write!(f, "Down"),
            VolumeStatus::Unknown => write!(f, "Unknown"),
        }
    }
}
