use std::fmt;

#[derive(Debug, Clone)]
pub struct StoreStats {
    pub num_keys: usize,
    pub num_segments: usize,
    pub total_bytes: u64,
    pub active_segment_id: usize,
    pub oldest_segment_id: usize,
}

impl StoreStats {
    pub fn new() -> Self {
        Self {
            num_keys: 0,
            num_segments: 0,
            total_bytes: 0,
            active_segment_id: 0,
            oldest_segment_id: 0,
        }
    }

    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }
}

impl fmt::Display for StoreStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Store Statistics:")?;
        writeln!(f, "  Keys: {}", self.num_keys)?;
        writeln!(f, "  Segments: {}", self.num_segments)?;
        writeln!(f, "  Total size: {:.2} MB", self.total_mb())?;
        writeln!(f, "  Active segment: {}", self.active_segment_id)?;
        writeln!(f, "  Oldest segment: {}", self.oldest_segment_id)?;
        Ok(())
    }
}

impl Default for StoreStats {
    fn default() -> Self {
        Self::new()
    }
}
