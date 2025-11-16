use crate::store::engine::KvStore;
use crate::store::segment::Segment;
use std::fs;
use std::io::Result;

/// Naive compaction: write a new segment (next id) with latest values for all keys,
/// then remove old segments and load the new one.
///
/// This is simple and blocking — fine for a learning project / demo.
pub fn compact_segments(store: &mut KvStore) -> Result<()> {
    // compute next id
    let next_id = store
        .segments
        .keys()
        .cloned()
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let dir = store.dir.clone();
    // create a fresh segment file
    let mut new_seg = Segment::open(&dir, next_id)?;
    // iterate current index and write latest value into new segment
    let keys: Vec<String> = store.index.map.keys().cloned().collect();
    for key in keys {
        if let Some((seg_id, offset, _val_len)) = store.index.get(&key) {
            // read value from old segment
            if let Some(seg) = store.segments.get_mut(seg_id) {
                if let Ok(opt) = seg.read_value_at(*offset) {
                    if let Some(value) = opt {
                        // write into new segment and update index
                        let off = new_seg.append(key.as_bytes(), &value)?;
                        store
                            .index
                            .insert(key.clone(), next_id, off, value.len() as u64);
                    }
                }
            }
        }
    }
    // delete old segments files (conservative: remove only segment-<id>.dat for ids < next_id)
    let ids_to_remove: Vec<usize> = store
        .segments
        .keys()
        .cloned()
        .filter(|&id| id < next_id)
        .collect();
    for id in ids_to_remove {
        let fname = format!("segment-{}.dat", id);
        let path = dir.join(fname);
        let _ = fs::remove_file(path);
    }
    // replace segments map: only keep next_id
    store.segments.clear();
    store.segments.insert(next_id, new_seg);
    store.active_id = next_id;
    Ok(())
}
