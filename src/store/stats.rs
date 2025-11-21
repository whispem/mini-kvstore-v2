//! Store statistics and metrics.

use std::fmt;

/// Statistics about the key-value store.
#[derive(Debug, Clone, Default)]
pub struct StoreStats {
    /// Number of keys in the store.
    pub num_keys: usize,
    /// Number of segment files.
    pub num_segments: usize,
    /// Total bytes used across all segments.
    pub total_bytes: u64,
    /// ID of the currently active segment.
    pub active_segment_id: usize,
    /// ID of the oldest segment.
    pub oldest_segment_id: usize,
}

impl StoreStats {
    /// Creates empty statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total size in megabytes.
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Returns the total size in kilobytes.
    pub fn total_kb(&self) -> f64 {
        self.total_bytes as f64 / 1024.0
    }
}

impl fmt::Display for StoreStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Store Statistics:")?;
        writeln!(f, "  Keys: {}", self.num_keys)?;
        writeln!(f, "  Segments: {}", self.num_segments)?;
        writeln!(f, "  Total size: {:.2} MB", self.total_mb())?;
        writeln!(f, "  Active segment: {}", self.active_segment_id)?;
        write!(f, "  Oldest segment: {}", self.oldest_segment_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_display() {
        let stats = StoreStats {
            num_keys: 100,
            num_segments: 3,
            total_bytes: 1024 * 1024 * 2, // 2 MB
            active_segment_id: 2,
            oldest_segment_id: 0,
        };

        let display = format!("{}", stats);
        assert!(display.contains("Keys: 100"));
        assert!(display.contains("Segments: 3"));
        assert!(display.contains("2.00 MB"));
    }

    #[test]
    fn test_stats_conversions() {
        let stats = StoreStats {
            total_bytes: 1024 * 1024,
            ..Default::default()
        };

        assert!((stats.total_mb() - 1.0).abs() < 0.001);
        assert!((stats.total_kb() - 1024.0).abs() < 0.001);
    }
}
