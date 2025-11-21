use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = KVStore::open("example_store")?;

    store.set("user:1:name", b"Alice")?;
    let name = store.get("user:1:name")?;
    assert_eq!(name, Some(b"Alice".to_vec()));
    if let Some(name) = name {
        println!("✓ Name: {}", String::from_utf8_lossy(&name));
    }
    Ok(())
}
