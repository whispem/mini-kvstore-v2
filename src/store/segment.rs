#![allow(unused_imports)]
#![allow(unused_variables)]
use crate::store::bloom::BloomIndex;
use crate::store::error::{Result, StoreError};
use crate::store::record;
use crate::store::record::{OP_DEL, OP_SET};
use crc32fast::Hasher as Crc32;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct Segment {
    pub id: u64,
    pub path: PathBuf,
    pub bloom: BloomIndex,
    pub size: u64,
    pub max_size: u64,
}

impl Segment {
    pub fn open(dir: &Path, id: u64, max_size: u64) -> Result<Self> {
        let path = dir.join(format!("segment-{}.dat", id));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)
            .map_err(StoreError::Io)?;

        let metadata = file.metadata().map_err(StoreError::Io)?;
        let size = metadata.len();

        Ok(Self {
            id,
            path,
            bloom: BloomIndex::new(10000),
            size,
            max_size,
        })
    }

    pub fn is_full(&self) -> bool {
        self.size >= self.max_size
    }

    pub fn read_record_at(&self, offset: u64) -> Result<Option<(String, Option<Vec<u8>>)>> {
        let mut file = File::open(&self.path).map_err(StoreError::Io)?;
        file.seek(SeekFrom::Start(offset)).map_err(StoreError::Io)?;

        let mut reader = BufReader::new(file);

        match record::read_record(&mut reader)? {
            Some((op, key, val)) => Ok(Some((key, val))),
            None => Ok(None),
        }
    }
}
