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

#![allow(clippy::type_complexity)]
#![allow(clippy::empty_line_after_doc_comments)]

use crate::store::error::{Result, StoreError};
use crc32fast::Hasher;
use std::io::{Read, Write};

pub const MAGIC: [u8; 2] = [0xF0, 0xF1];

pub const OP_SET: u8 = 1;
pub const OP_DEL: u8 = 2;

pub fn write_record<W: Write>(
    mut w: W,
    op: u8,
    key: &str,
    value: Option<&[u8]>,
) -> Result<()> {
    // MAGIC
    w.write_all(&MAGIC)?;

    // OP
    w.write_all(&[op])?;

    // KEY
    let key_bytes = key.as_bytes();
    let key_len = key_bytes.len() as u32;
    w.write_all(&key_len.to_le_bytes())?;

    // VAL
    let val_bytes = value.unwrap_or(&[]);
    let val_len = val_bytes.len() as u32;
    w.write_all(&val_len.to_le_bytes())?;

    // PAYLOAD
    w.write_all(key_bytes)?;
    if op == OP_SET {
        w.write_all(val_bytes)?;
    }

    // Compute checksum
    let mut hasher = Hasher::new();
    hasher.update(&[op]);
    hasher.update(&key_len.to_le_bytes());
    hasher.update(&val_len.to_le_bytes());
    hasher.update(key_bytes);
    if op == OP_SET {
        hasher.update(val_bytes);
    }
    let checksum = hasher.finalize();

    // Write checksum
    w.write_all(&checksum.to_le_bytes())?;

    Ok(())
}

pub fn read_record<R: Read>(
    mut r: R,
) -> Result<Option<(u8, String, Option<Vec<u8>>)>> {
    let mut magic = [0u8; 2];
    if r.read_exact(&mut magic).is_err() {
        return Ok(None);
    }
    if magic != MAGIC {
        return Err(StoreError::Corrupted("invalid magic".into()));
    }

    let mut op = [0u8; 1];
    r.read_exact(&mut op)?;

    let mut key_len_bytes = [0u8; 4];
    r.read_exact(&mut key_len_bytes)?;
    let key_len = u32::from_le_bytes(key_len_bytes) as usize;

    let mut val_len_bytes = [0u8; 4];
    r.read_exact(&mut val_len_bytes)?;
    let val_len = u32::from_le_bytes(val_len_bytes) as usize;

    let mut key = vec![0u8; key_len];
    r.read_exact(&mut key)?;
    let key = String::from_utf8(key)
        .map_err(|_| StoreError::Corrupted("invalid UTF-8 in key".into()))?;

    let value = if op[0] == OP_SET {
        let mut val = vec![0u8; val_len];
        r.read_exact(&mut val)?;
        Some(val)
    } else {
        None
    };

    let mut checksum_bytes = [0u8; 4];
    r.read_exact(&mut checksum_bytes)?;
    let checksum_stored = u32::from_le_bytes(checksum_bytes);

    // Recompute checksum
    let mut hasher = Hasher::new();
    hasher.update(&op);
    hasher.update(&key_len_bytes);
    hasher.update(&val_len_bytes);
    hasher.update(key.as_bytes());
    if let Some(ref v) = value {
        hasher.update(v);
    }

    let checksum_calc = hasher.finalize();
    if checksum_calc != checksum_stored {
        return Err(StoreError::Corrupted("checksum mismatch".into()));
    }

    Ok(Some((op[0], key, value)))
}
