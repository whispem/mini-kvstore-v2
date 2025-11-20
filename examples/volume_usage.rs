//! Example: Using the Volume server programmatically

use mini_kvstore_v2::volume::BlobStorage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Volume Storage Example ===\n");

    // Create a blob storage instance
    let mut storage = BlobStorage::new("example_volume_data", "example-vol".to_string())?;
    println!(
        "✓ Volume storage initialized (volume_id: {})",
        storage.volume_id()
    );

    // Store some blobs
    let meta1 = storage.put("user:alice:avatar", b"<binary image data>")?;
    println!(
        "✓ Stored blob: key={}, etag={}, size={} bytes",
        meta1.key, meta1.etag, meta1.size
    );

    let meta2 = storage.put("user:bob:profile", b"{\"name\": \"Bob\", \"age\": 30}")?;
    println!(
        "✓ Stored blob: key={}, etag={}, size={} bytes",
        meta2.key, meta2.etag, meta2.size
    );

    // Retrieve a blob
    if let Some(data) = storage.get("user:bob:profile")? {
        println!("✓ Retrieved blob: {}", String::from_utf8_lossy(&data));
    }

    // List all blobs
    let keys = storage.list_keys();
    println!("\n✓ Total blobs: {}", keys.len());
    for key in &keys {
        println!("  - {}", key);
    }

    // Delete a blob
    storage.delete("user:alice:avatar")?;
    println!("\n✓ Deleted blob: user:alice:avatar");

    // Verify deletion
    assert!(storage.get("user:alice:avatar")?.is_none());
    println!("✓ Verified deletion");

    // Show storage stats
    let stats = storage.stats();
    println!("\n✓ Storage statistics:");
    println!("  Keys: {}", stats.num_keys);
    println!("  Segments: {}", stats.num_segments);
    println!("  Total size: {:.2} MB", stats.total_mb());

    println!("\n✓ Volume storage example completed!");

    Ok(())
}
