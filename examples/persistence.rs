use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Persistence Example ===\n");

    let data_dir = "persistence_example";

    // First session: write data
    println!("Session 1: Writing data...");
    {
        let mut store = KVStore::open(data_dir)?;
        store.set("session", b"first")?;
        store.set("counter", b"42")?;
        store.set("name", b"Test Store")?;
        println!("✓ Wrote 3 keys");

        let stats = store.stats();
        println!("  Keys: {}", stats.num_keys);
    }
    println!("✓ Store closed\n");

    // Second session: read and modify
    println!("Session 2: Reading and modifying...");
    {
        let mut store = KVStore::open(data_dir)?;

        // Verify data persisted
        assert_eq!(store.get("session")?, Some(b"first".to_vec()));
        assert_eq!(store.get("counter")?, Some(b"42".to_vec()));
        assert_eq!(store.get("name")?, Some(b"Test Store".to_vec()));
        println!("✓ All data persisted correctly");

        // Modify data
        store.set("session", b"second")?;
        store.set("counter", b"43")?;
        store.delete("name")?;
        println!("✓ Modified data");

        let stats = store.stats();
        println!("  Keys: {}", stats.num_keys);
    }
    println!("✓ Store closed\n");

    // Third session: verify modifications
    println!("Session 3: Verifying modifications...");
    {
        let mut store = KVStore::open(data_dir)?;

        assert_eq!(store.get("session")?, Some(b"second".to_vec()));
        assert_eq!(store.get("counter")?, Some(b"43".to_vec()));
        assert_eq!(store.get("name")?, None);
        println!("✓ Modifications persisted correctly");

        let stats = store.stats();
        println!("  Keys: {}", stats.num_keys);
        println!("  Segments: {}", stats.num_segments);
    }

    println!("\n✓ Persistence verified across 3 sessions!");

    Ok(())
}
