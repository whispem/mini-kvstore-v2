use crate::store::engine::KVStore;
use crate::store::segment::Segment;
use std::fs;
use std::io::{Result, Seek, SeekFrom, Write};

pub fn compact_segments(store: &mut KVStore) -> Result<()> {
    println!("Starting compaction...");

    let next_id = store.segments.keys().copied().max().map_or(1, |m| m + 1);

    // Create temporary segment
    let dir = store.config.data_dir.clone();
    let temp_filename = format!("segment-{:04}.tmp", next_id);
    let temp_path = dir.join(&temp_filename);

    let final_filename = format!("segment-{:04}.dat", next_id);
    let final_path = dir.join(&final_filename);

    // Write all live keys to temporary file using the simple format
    let mut temp_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open(&temp_path)?;

    let keys: Vec<String> = store.index.map.keys().cloned().collect();
    let mut new_index_entries = Vec::new();

    for key in keys {
        if let Some((seg_id, offset, _val_len)) = store.index.get(&key) {
            if let Some(seg) = store.segments.get_mut(seg_id) {
                if let Ok(Some(value)) = seg.read_value_at(*offset) {
                    // Write using simple format: key_len + value_len + key + value
                    let write_offset = temp_file.seek(SeekFrom::End(0))?;

                    let key_bytes = key.as_bytes();
                    let key_len = key_bytes.len() as u64;
                    let value_len = value.len() as u64;

                    // Write header
                    temp_file.write_all(&key_len.to_le_bytes())?;
                    temp_file.write_all(&value_len.to_le_bytes())?;

                    // Write data
                    temp_file.write_all(key_bytes)?;
                    temp_file.write_all(&value)?;

                    new_index_entries.push((
                        key.clone(),
                        next_id,
                        write_offset,
                        value.len() as u64,
                    ));
                }
            }
        }
    }

    // Sync temp file
    temp_file.sync_all()?;
    drop(temp_file);

    // Atomically rename temp to final
    fs::rename(&temp_path, &final_path)?;

    println!(
        "Compacted {} keys into segment {}",
        new_index_entries.len(),
        next_id
    );

    // Collect old segment IDs to remove
    let ids_to_remove: Vec<usize> = store
        .segments
        .keys()
        .copied()
        .filter(|&id| id < next_id)
        .collect();

    // Remove old segments from disk
    for id in &ids_to_remove {
        let fname = format!("segment-{:04}.dat", id);
        let path = dir.join(fname);
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Warning: Failed to remove old segment {}: {}", id, e);
        } else {
            println!("Removed old segment {}", id);
        }
    }

    // Remove old segments from memory
    for id in ids_to_remove {
        store.segments.remove(&id);
    }

    // Open the new compacted segment
    let compacted_seg = Segment::open(&dir, next_id)?;
    store.segments.insert(next_id, compacted_seg);
    store.active_id = next_id;

    // Update index with new positions
    for (key, seg_id, offset, value_len) in new_index_entries {
        store.index.insert(key, seg_id, offset, value_len);
    }

    println!("Compaction complete!");
    Ok(())
}
