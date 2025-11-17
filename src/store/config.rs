#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
    pub segment_size: usize,
}

impl Config {
    pub fn new(db_path: String, segment_size: usize) -> Self {
        Self { db_path, segment_size }
    }

    pub fn default() -> Self {
        Self {
            db_path: String::from("./data"),
            segment_size: 1024 * 1024,
        }
    }
}
