use mini_kvstore_v2::KVStore;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Large Dataset Example ===\n");

    let mut store = KVStore::open("large_dataset_example")?;

    // Insert 10,000 keys
    println!("Inserting 10,000 keys...");
    let start = Instant::now();
    for i in 0..10_000 {
        let key = format!("user:{:05}:data", i);
        let value = format!("User data for ID {}", i);
        store.set(&key, value.as_bytes())?;

        if (i + 1) % 2000 == 0 {
            println!("  {} keys inserted...", i + 1);
        }
    }
    let insert_duration = start.elapsed();
    println!(
        "✓ Insertion completed in {:.2}s",
        insert_duration.as_secs_f64()
    );

    // Verify first and last keys exist
    let first = store.get("user:00000:data")?;
    assert_eq!(
        first,
        Some(b"User data for ID 0".to_vec()),
        "First key should exist"
    );

    let last = store.get("user:09999:data")?;
    assert_eq!(
        last,
        Some(b"User data for ID 9999".to_vec()),
        "Last key should exist"
    );

    // Verify a sample in the middle
    let middle = store.get("user:05000:data")?;
    assert_eq!(
        middle,
        Some(b"User data for ID 5000".to_vec()),
        "Middle key should exist"
    );

    // Read random keys (every 10th key = 1,000 reads)
    println!("\nReading 1,000 random keys...");
    let start = Instant::now();
    let mut read_count = 0;
    for i in (0..10_000).step_by(10) {
        let key = format!("user:{:05}:data", i);
        let value = store.get(&key)?;
        assert!(value.is_some(), "Key {} should exist", key);
        
        // Verify content for a few samples
        if i % 1000 == 0 {
            let expected = format!("User data for ID {}", i);
            assert_eq!(
                value.unwrap(),
                expected.as_bytes().to_vec(),
                "Key {} should have correct value",
                key
            );
        }
        read_count += 1;
    }
    let read_duration = start.elapsed();
    assert_eq!(read_count, 1_000, "Should have read exactly 1,000 keys");
    println!("✓ Read completed in {:.2}s", read_duration.as_secs_f64());

    // Show statistics
    let stats = store.stats();
    assert_eq!(stats.num_keys, 10_000, "Should have 10,000 keys");
    assert!(stats.num_segments >= 1, "Should have at least one segment");
    assert!(stats.total_bytes > 0, "Should have non-zero data size");
    
    println!("\n✓ Final statistics:");
    println!("{}", stats);

    // Calculate and display performance metrics
    let insert_rate = 10_000.0 / insert_duration.as_secs_f64();
    let read_rate = 1_000.0 / read_duration.as_secs_f64();
    
    println!("\nPerformance:");
    println!("  Insert rate: {:.0} keys/sec", insert_rate);
    println!("  Read rate: {:.0} keys/sec", read_rate);
    println!("  Avg insert latency: {:.2} ms", 1000.0 / insert_rate);
    println!("  Avg read latency: {:.2} ms", 1000.0 / read_rate);

    // Basic performance assertions (sanity checks)
    assert!(
        insert_rate > 100.0,
        "Insert rate should be reasonable (>100 keys/sec)"
    );
    assert!(
        read_rate > 100.0,
        "Read rate should be reasonable (>100 keys/sec)"
    );

    println!("\n✓ Large dataset test completed successfully!");

    Ok(())
}
