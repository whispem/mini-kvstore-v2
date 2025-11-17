#[derive(Debug, Clone)]
pub struct Record {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub deleted: bool,
}

impl Record {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { key, value, deleted: false }
    }

    pub fn tombstone(key: Vec<u8>) -> Self {
        Self { key, value: Vec::new(), deleted: true }
    }
}
