use crate::store::engine::KVStore;
use crate::store::segment::Segment;
use std::fs;
use std::io::Result;

pub fn compact_segments(store: &mut KVStore) -> Result<()> {

    let next_id = store
        .segments
        .keys()
        .copied()
        .max()
        .map_or(0, |m| m + 1);

    let dir = store.dir.clone();
    let mut new_seg = Segment::open(&dir, next_id)?;

    let keys: Vec<&str> = store.index.map.keys().map(|s| s.as_str()).collect();

    for key in keys {
        if let Some((seg_id, offset, _val_len)) = store.index.get(key) {
            if let Some(seg) = store.segments.get_mut(seg_id) {
                if let Ok(Some(value)) = seg.read_value_at(*offset) {
                    let off = new_seg.append(key.as_bytes(), &value)?;
                    store
                        .index
                        .insert(key.to_string(), next_id, off, value.len() as u64);
                }
            }
        }
    }

    let ids_to_remove: Vec<usize> = store
        .segments
        .keys()
        .copied()
        .filter(|&id| id < next_id)
        .collect();

    for id in ids_to_remove {
        let fname = format!("segment-{}.dat", id);
        let path = dir.join(fname);

        let _ = fs::remove_file(path);
        store.segments.remove(&id);
    }

    store.segments.insert(next_id, new_seg);
    store.active_id = next_id;

    Ok(())
}
