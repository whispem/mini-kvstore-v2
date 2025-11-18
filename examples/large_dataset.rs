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

    // Read random keys
    println!("\nReading 1,000 random keys...");
    let start = Instant::now();
    for i in (0..10_000).step_by(10) {
        let key = format!("user:{:05}:data", i);
        let _ = store.get(&key)?;
    }
    let read_duration = start.elapsed();
    println!("✓ Read completed in {:.2}s", read_duration.as_secs_f64());

    // Show statistics
    let stats = store.stats();
    println!("\n✓ Final statistics:");
    println!("{}", stats);

    println!("\nPerformance:");
    println!(
        "  Insert rate: {:.0} keys/sec",
        10_000.0 / insert_duration.as_secs_f64()
    );
    println!(
        "  Read rate: {:.0} keys/sec",
        1_000.0 / read_duration.as_secs_f64()
    );

    Ok(())
}
