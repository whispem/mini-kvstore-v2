// src/volume/config.rs

use std::net::SocketAddr;

#[derive(Clone)]
pub struct VolumeConfig {
    pub volume_id: String,
    pub data_dir: String,
    pub bind_addr: SocketAddr,
}

impl VolumeConfig {
    pub fn new(volume_id: impl Into<String>) -> Self {
        Self {
            volume_id: volume_id.into(),
            data_dir: "data".to_string(),
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 9002)),
        }
    }

    pub fn with_data_dir(mut self, dir: impl Into<String>) -> Self {
        self.data_dir = dir.into();
        self
    }

    pub fn with_bind_addr(mut self, addr: SocketAddr) -> Self {
        self.bind_addr = addr;
        self
    }
}
