//! Basic usage example for the KVStore.

use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Usage: mini-kvstore-v2 ===");

    // Open the store (will create if missing)
    let mut store = KVStore::open("example_store")?;

    // Set values
    store.set("user:1:name", b"Alice")?;
    store.set("user:1:email", b"alice@example.com")?;
    store.set("user:2:name", b"Bob")?;
    store.set("user:2:email", b"bob@example.com")?;

    // Get values
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
    println!(
        "✓ User 1 email: {}",
        String::from_utf8_lossy(&email.unwrap())
    );

    // Remove a key
    store.delete("user:2:email")?;
    let deleted_email = store.get("user:2:email")?;
    assert_eq!(deleted_email, None, "User 2's email should be deleted");
    println!("✓ Deleted user 2 email");

    // Verify other keys still exist
    let bob_name = store.get("user:2:name")?;
    assert_eq!(
        bob_name,
        Some(b"Bob".to_vec()),
        "Bob's name should still exist"
    );

    // List all keys
    let keys = store.list_keys();
    println!("Keys in store: {:?}", keys);

    Ok(())
}
