use std::io::{Read, Write, Result};

pub const TOMBSTONE_MARKER: u8 = 0x01;

#[derive(Debug, Clone)]
pub struct RecordHeader {
    pub key_len: u32,
    pub value_len: u32,
    pub flags: u8,
    pub checksum: u32,
}

impl RecordHeader {
    pub fn new(key_len: u32, value_len: u32, flags: u8, checksum: u32) -> Self {
        Self {
            key_len,
            value_len,
            flags,
            checksum,
        }
    }

    pub const HEADER_SIZE: usize = 13;

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.key_len.to_le_bytes())?;
        writer.write_all(&self.value_len.to_le_bytes())?;
        writer.write_all(&[self.flags])?;
        writer.write_all(&self.checksum.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; Self::HEADER_SIZE];
        reader.read_exact(&mut buf)?;

        let key_len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let value_len = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let flags = buf[8];
        let checksum = u32::from_le_bytes([buf[9], buf[10], buf[11], buf[12]]);

        Ok(Self {
            key_len,
            value_len,
            flags,
            checksum,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub deleted: bool,
}

impl Record {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self { 
            key, 
            value, 
            deleted: false 
        }
    }

    pub fn tombstone(key: Vec<u8>) -> Self {
        Self { 
            key, 
            value: Vec::new(), 
            deleted: true 
        }
    }
}

pub fn compute_checksum(key: &[u8], value: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(key);
    hasher.update(value);
    hasher.finalize()
}
