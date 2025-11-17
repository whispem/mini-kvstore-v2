use crate::store::engine::KVStore;
use crate::store::segment::Segment;
use std::fs;
use std::path::Path;

pub fn compact_segments(store: &mut KVStore) -> std::io::Result<()> {
    let data_dir = &store.config.data_dir;
    // Collect all live key-value pairs with latest segment/offset
    let mut live_data: Vec<(String, Vec<u8>)> = Vec::new();

    // Rebuild index from segments (should already be correct)
    for key in store.index.map.keys() {
        if let Some(&(seg_id, offset, _len)) = store.index.get(key) {
            if let Some(seg) = store.segments.get_mut(&seg_id) {
                if let Ok(Some(value)) = seg.read_value_at(offset) {
                    live_data.push((key.clone(), value));
                }
            }
        }
    }

    // Clean up: remove all old segments
    for seg_id in store.segments.keys() {
        let filename = format!("segment-{}.dat", seg_id);
        let path = data_dir.join(&filename);
        let _ = fs::remove_file(&path);
    }

    store.segments.clear();
    store.index.map.clear();

    // Write live data into fresh segment (id = 0, then increment if needed)
    store.active_id = 0;
    let mut active_seg = Segment::open(data_dir, store.active_id)?;
    store.segments.insert(store.active_id, active_seg);

    for (key, value) in live_data {
        let active_seg = store.segments.get_mut(&store.active_id).unwrap();
        // If segment is full, create a new one
        if active_seg.is_full() {
            store.active_id += 1;
            let seg = Segment::open(data_dir, store.active_id)?;
            store.segments.insert(store.active_id, seg);
        }
        let seg_ref = store.segments.get_mut(&store.active_id).unwrap();
        let offset = seg_ref.append(key.as_bytes(), &value)?;
        store.index
            .insert(key.clone(), store.active_id, offset, value.len() as u64);
    }

    Ok(())
}
