//! Record framing for mini-kvstore-v2.
//!
//! Format of each record in segments:
//!
//! [MAGIC: 2 bytes = 0xF0 0xF1]
//! [OP: 1 byte]
//! [KEY_LEN: u32 LE]
//! [VAL_LEN: u32 LE; 0 if delete]
//! [KEY: bytes]
//! [VAL: bytes]
//! [CHECKSUM: u32 LE]  // CRC32 over OP..VAL

use crate::store::error::{Result, StoreError};
use crc32fast::Hasher;
use std::io::{Read, Write};

pub const MAGIC: [u8; 2] = [0xF0, 0xF1];

/// Operation code
pub const OP_SET: u8 = 0;
pub const OP_DEL: u8 = 1;

/// Write a full framed record (set or delete)
pub fn write_record<W: Write>(
    mut w: W,
    op: u8,
    key: &[u8],
    value: Option<&[u8]>,
) -> Result<()> {
    let val = value.unwrap_or(&[]);
    let key_len = key.len() as u32;
    let val_len = val.len() as u32;

    // Compute checksum
    let mut hasher = Hasher::new();
    hasher.update(&[op]);
    hasher.update(&key_len.to_le_bytes());
    hasher.update(&val_len.to_le_bytes());
    hasher.update(key);
    hasher.update(val);
    let checksum = hasher.finalize().to_le_bytes();

    // Write framing
    w.write_all(&MAGIC)?;
    w.write_all(&[op])?;
    w.write_all(&key_len.to_le_bytes())?;
    w.write_all(&val_len.to_le_bytes())?;
    w.write_all(key)?;
    w.write_all(val)?;
    w.write_all(&checksum)?;
    Ok(())
}

/// Read a full record.
/// Returns Ok(None) on EOF.
pub fn read_record<R: Read>(mut r: R) -> Result<Option<(u8, String, Option<Vec<u8>>)>> {
    let mut magic = [0u8; 2];
    match r.read_exact(&mut magic) {
        Ok(_) => {}
        Err(_) => return Ok(None), // EOF
    }

    if magic != MAGIC {
        return Err(StoreError::CorruptedData(
            "Bad record magic".to_string(),
        ));
    }

    // op
    let mut op = [0u8; 1];
    r.read_exact(&mut op)?;

    // lengths
    let mut key_len_buf = [0u8; 4];
    let mut val_len_buf = [0u8; 4];
    r.read_exact(&mut key_len_buf)?;
    r.read_exact(&mut val_len_buf)?;

    let key_len = u32::from_le_bytes(key_len_buf) as usize;
    let val_len = u32::from_le_bytes(val_len_buf) as usize;

    // key
    let mut key = vec![0u8; key_len];
    r.read_exact(&mut key)?;
    let key_str = String::from_utf8(key)
        .map_err(|e| StoreError::CorruptedData(format!("Bad UTF-8 key: {}", e)))?;

    // value (if any)
    let mut val = vec![0u8; val_len];
    if val_len > 0 {
        r.read_exact(&mut val)?;
    }

    // checksum
    let mut checksum_buf = [0u8; 4];
    r.read_exact(&mut checksum_buf)?;
    let checksum = u32::from_le_bytes(checksum_buf);

    // recompute
    let mut hasher = Hasher::new();
    hasher.update(&op);
    hasher.update(&key_len_buf);
    hasher.update(&val_len_buf);
    hasher.update(key_str.as_bytes());
    hasher.update(&val);
    let expected = hasher.finalize();

    if expected != checksum {
        return Err(StoreError::CorruptedData(
            "Checksum mismatch".to_string(),
        ));
    }

    Ok(Some((
        op[0],
        key_str,
        if val_len > 0 { Some(val) } else { None },
    )))
}
