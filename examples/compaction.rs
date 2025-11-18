use mini_kvstore_v2::KVStore;

fn main() -> std::io::Result<()> {
    println!("=== Compaction Example ===\n");

    let mut store = KVStore::open("compaction_example")?;

    // Write many versions of the same keys
    println!("Writing 100 keys, 10 versions each...");
    for round in 0..10 {
        for i in 0..100 {
            let key = format!("key_{}", i);
            let value = format!("value_{}_{}", i, round);
            store.set(&key, value.as_bytes())?;
        }
        println!("  Round {} completed", round + 1);
    }

    let stats_before = store.stats();
    println!("\nBefore compaction:");
    println!("  Keys: {}", stats_before.num_keys);
    println!("  Segments: {}", stats_before.num_segments);
    println!("  Total size: {:.2} MB", stats_before.total_mb());

    // Run compaction
    println!("\nRunning compaction...");
    store.compact()?;

    let stats_after = store.stats();
    println!("\nAfter compaction:");
    println!("  Keys: {}", stats_after.num_keys);
    println!("  Segments: {}", stats_after.num_segments);
    println!("  Total size: {:.2} MB", stats_after.total_mb());

    let saved_mb = (stats_before.total_bytes - stats_after.total_bytes) as f64 / (1024.0 * 1024.0);
    println!(
        "\n✓ Saved {:.2} MB ({:.1}%)",
        saved_mb,
        (saved_mb / stats_before.total_mb()) * 100.0
    );

    Ok(())
}
