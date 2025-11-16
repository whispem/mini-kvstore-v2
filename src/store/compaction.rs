use crate::store::engine::KvStore;
use crate::store::segment::Segment;
use std::fs;
use std::io::Result;

/// Naive compaction: write latest values for all keys into a new segment.
pub fn compact_segments(store: &mut KvStore) -> Result<()> {
    let next_id = store
        .segments
        .keys()
        .cloned()
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let dir = store.dir.clone();
    let mut new_seg = Segment::open(&dir, next_id)?;
    let keys: Vec<String> = store.index.map.keys().cloned().collect();
    for key in keys {
        if let Some((seg_id, offset, _val_len)) = store.index.get(&key) {
            if let Some(seg) = store.segments.get_mut(seg_id) {
                if let Ok(opt) = seg.read_value_at(*offset) {
                    if let Some(value) = opt {
                        let off = new_seg.append(key.as_bytes(), &value)?;
                        store
                            .index
                            .insert(key.clone(), next_id, off, value.len() as u64);
                    }
                }
            }
        }
    }
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
    store.segments.clear();
    store.segments.insert(next_id, new_seg);
    store.active_id = next_id;
    Ok(())
}
