use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Verify a sample key has the latest value
    let sample = store.get("key_0")?;
    assert_eq!(
        sample,
        Some(b"value_0_9".to_vec()),
        "Key should have value from last round (9)"
    );

    let stats_before = store.stats();
    assert_eq!(
        stats_before.num_keys, 100,
        "Should have exactly 100 unique keys"
    );

    println!("\nBefore compaction:");
    println!("  Keys: {}", stats_before.num_keys);
    println!("  Bytes: {:.2} MB", stats_before.total_mb());

    // Actual compaction operation
    println!("\nCompacting...");
    store.compact()?;

    let stats_after = store.stats();
    println!("\nAfter compaction:");
    println!("  Keys: {}", stats_after.num_keys);
    println!("  Bytes: {:.2} MB", stats_after.total_mb());

    // Verify keys still exist
    for i in 0..100 {
        let key = format!("key_{}", i);
        let value = store.get(&key)?;
        assert_eq!(
            value,
            Some(format!("value_{}_9", i).as_bytes().to_vec()),
            "Key value should survive compaction"
        );
    }
    println!("\n✓ All 100 keys verified - data integrity preserved");

    let saved_bytes = stats_before
        .total_bytes
        .saturating_sub(stats_after.total_bytes);
    let saved_mb = saved_bytes as f64 / (1024.0 * 1024.0);
    let saved_pct = if stats_before.total_bytes > 0 {
        (saved_bytes as f64 / stats_before.total_bytes as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Compaction saved {:.2} MB ({:.1}%)",
        saved_mb, saved_pct
    );

    Ok(())
}
