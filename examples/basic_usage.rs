use mini_kvstore_v2::KVStore;

fn main() -> std::io::Result<()> {
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

    // Get values
    if let Some(name) = store.get("user:1:name")? {
        println!("✓ User 1 name: {}", String::from_utf8_lossy(&name));
    }

    // Update a value
    store.set("user:1:email", b"alice.new@example.com")?;
    println!("✓ Updated user:1:email");

    // Delete a key
    store.delete("user:2:email")?;
    println!("✓ Deleted user:2:email");

    // List all keys
    let keys = store.list_keys();
    println!("\n✓ Current keys ({}):", keys.len());
    for key in keys {
        println!("  - {}", key);
    }

    // Show statistics
    let stats = store.stats();
    println!("\n✓ Store statistics:");
    println!("{}", stats);

    println!("\n✓ Example completed successfully!");

    Ok(())
}
