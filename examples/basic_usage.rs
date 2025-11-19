use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic KVStore Usage ===\n");

    // Create or open a store
    let mut store = KVStore::open("example_data")?;
    println!("✓ Store opened");

    // Set some key-value pairs
    store.set("user:1:name", b"Alice")?;
    store.set("user:1:email", b"alice@example.com")?;
    store.set("user:2:name", b"Bob")?;
    store.set("user:2:email", b"bob@example.com")?;
    println!("✓ Added 4 keys");

    // Get values - with assertions to demonstrate expected behavior
    let name = store.get("user:1:name")?;
    assert_eq!(
        name,
        Some(b"Alice".to_vec()),
        "Should retrieve Alice's name"
    );
    println!("✓ User 1 name: {}", String::from_utf8_lossy(&name.unwrap()));

    let email = store.get("user:1:email")?;
    assert_eq!(
        email,
        Some(b"alice@example.com".to_vec()),
        "Should retrieve Alice's email"
    );

    // Update a value
    store.set("user:1:email", b"alice.new@example.com")?;
    let updated = store.get("user:1:email")?;
    assert_eq!(
        updated,
        Some(b"alice.new@example.com".to_vec()),
        "Should retrieve updated email"
    );
    println!("✓ Updated user:1:email");

    // Delete a key
    store.delete("user:2:email")?;
    let deleted = store.get("user:2:email")?;
    assert_eq!(deleted, None, "Deleted key should return None");
    println!("✓ Deleted user:2:email");

    // Verify other keys still exist
    let bob_name = store.get("user:2:name")?;
    assert_eq!(
        bob_name,
        Some(b"Bob".to_vec()),
        "Bob's name should still exist"
    );

    // List all keys
    let keys = store.list_keys();
    assert_eq!(keys.len(), 3, "Should have 3 keys after one deletion");
    println!("\n✓ Current keys ({}):", keys.len());
    for key in &keys {
        println!("  - {}", key);
    }

    // Verify the expected keys are present
    assert!(keys.contains(&"user:1:name".to_string()));
    assert!(keys.contains(&"user:1:email".to_string()));
    assert!(keys.contains(&"user:2:name".to_string()));

    // Show statistics
    let stats = store.stats();
    assert_eq!(stats.num_keys, 3, "Stats should show 3 keys");
    assert!(stats.num_segments >= 1, "Should have at least one segment");
    assert!(stats.total_bytes > 0, "Should have non-zero data");
    
    println!("\n✓ Store statistics:");
    println!("{}", stats);

    println!("\n✓ Example completed successfully!");

    Ok(())
}
