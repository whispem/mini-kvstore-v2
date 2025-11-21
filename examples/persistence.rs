use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Persistence Example ===");

    // Session 1: set values
    {
        let mut store = KVStore::open("persisted_store")?;
        store.set("session", b"first")?;
        store.set("counter", b"42")?;
        store.set("name", b"Test Store")?;
        println!("✓ Values written: session, counter, name");
    }

    // Session 2: read and modify values
    {
        let mut store = KVStore::open("persisted_store")?;
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

        // Update counter
        let new_counter = b"43";
        store.set("counter", new_counter)?;
        println!("✓ Counter updated to 43");

        // Delete "name"
        store.delete("name")?;
        println!("✓ Name deleted");
    }

    // Session 3: verify changes
    {
        let store = KVStore::open("persisted_store")?;
        let session = store.get("session")?;
        assert_eq!(
            session,
            Some(b"first".to_vec()),
            "Session should still persist"
        );
        let counter = store.get("counter")?;
        assert_eq!(
            counter,
            Some(b"43".to_vec()),
            "Counter should reflect update"
        );
        let name = store.get("name")?;
        assert_eq!(name, None, "Name should have been deleted");
        println!("✓ Session, updated counter, and delete verified");
    }

    println!("  - Session 2: Read, modify, delete");
    println!("  - Session 3: Verify all changes persisted");

    Ok(())
}
