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
        assert_eq!(stats.num_keys, 3, "Should have 3 keys in session 1");
        println!("  Keys: {}", stats.num_keys);
    }
    println!("✓ Store closed\n");

    // Second session: read and modify
    println!("Session 2: Reading and modifying...");
    {
        let mut store = KVStore::open(data_dir)?;

        // Verify data persisted from session 1
        let session = store.get("session")?;
        assert_eq!(
            session,
            Some(b"first".to_vec()),
            "Session value should persist"
        );

        let counter = store.get("counter")?;
        assert_eq!(
            counter,
            Some(b"42".to_vec()),
            "Counter value should persist"
        );

        let name = store.get("name")?;
        assert_eq!(
            name,
            Some(b"Test Store".to_vec()),
            "Name value should persist"
        );

        println!("✓ All data persisted correctly from session 1");

        // Modify data
        store.set("session", b"second")?;
        store.set("counter", b"43")?;
        store.delete("name")?;
        println!("✓ Modified data in session 2");

        let stats = store.stats();
        assert_eq!(stats.num_keys, 2, "Should have 2 keys after deletion");
        println!("  Keys: {}", stats.num_keys);
    }
    println!("✓ Store closed\n");

    // Third session: verify modifications
    println!("Session 3: Verifying modifications...");
    {
        let mut store = KVStore::open(data_dir)?;

        // Verify modifications from session 2 persisted
        let session = store.get("session")?;
        assert_eq!(
            session,
            Some(b"second".to_vec()),
            "Updated session value should persist"
        );

        let counter = store.get("counter")?;
        assert_eq!(
            counter,
            Some(b"43".to_vec()),
            "Updated counter value should persist"
        );

        let name = store.get("name")?;
        assert_eq!(name, None, "Deleted key should not exist");

        println!("✓ Modifications persisted correctly from session 2");

        let stats = store.stats();
        assert_eq!(stats.num_keys, 2, "Should still have 2 keys");
        println!("  Keys: {}", stats.num_keys);
        println!("  Segments: {}", stats.num_segments);
    }

    println!("\n✓ Persistence verified across 3 sessions!");
    println!("  - Session 1: Write initial data");
    println!("  - Session 2: Read, modify, delete");
    println!("  - Session 3: Verify all changes persisted");

    Ok(())
}
