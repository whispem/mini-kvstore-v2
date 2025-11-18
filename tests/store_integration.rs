use mini_kvstore_v2::KVStore;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::Path;

fn setup_test_dir(path: &str) {
    let p = Path::new(path);
    let _ = remove_dir_all(p);
    create_dir_all(p).expect("Failed to create test directory");
}

#[test]
fn can_set_and_get_value() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/can_set_and_get_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("foo", b"bar")?;
    assert_eq!(store.get("foo")?, Some(b"bar".to_vec()));

    Ok(())
}

#[test]
fn can_delete_value() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/can_delete_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("foo", b"bar")?;
    store.delete("foo")?;
    assert_eq!(store.get("foo")?, None);

    Ok(())
}

#[test]
fn overwriting_value_updates_storage() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/overwrite_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("foo", b"1")?;
    store.set("foo", b"2")?;

    assert_eq!(store.get("foo")?, Some(b"2".to_vec()));

    Ok(())
}

#[test]
fn missing_key_returns_none() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/missing_key";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    assert_eq!(store.get("does_not_exist")?, None);

    Ok(())
}

#[test]
fn delete_nonexistent_key_is_safe() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/delete_nonexistent";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.delete("nope")?;
    assert_eq!(store.get("nope")?, None);

    Ok(())
}

#[test]
fn persistence_after_reopen() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/persistence";
    setup_test_dir(test_dir);

    {
        let mut store = KVStore::open(test_dir)?;
        store.set("persistent", b"value")?;
    }

    let mut store = KVStore::open(test_dir)?;
    assert_eq!(store.get("persistent")?, Some(b"value".to_vec()));

    Ok(())
}

#[test]
fn compaction_preserves_data() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/compaction";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("key1", b"value1")?;
    store.set("key2", b"value2")?;
    store.set("key3", b"value3")?;

    store.set("key1", b"updated1")?;
    store.delete("key2")?;

    store.compact()?;

    assert_eq!(store.get("key1")?, Some(b"updated1".to_vec()));
    assert_eq!(store.get("key2")?, None);
    assert_eq!(store.get("key3")?, Some(b"value3".to_vec()));

    Ok(())
}

#[test]
fn large_value_handling() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/large_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    // 1MB value
    let large_value = vec![b'x'; 1024 * 1024];
    store.set("large", &large_value)?;

    let retrieved = store.get("large")?;
    assert_eq!(retrieved, Some(large_value));

    Ok(())
}

#[test]
fn utf8_keys_and_values() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/utf8";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("key", "value".as_bytes())?;
    store.set("🔑", "🎉".as_bytes())?;

    assert_eq!(store.get("key")?, Some(b"value".to_vec()));
    assert_eq!(store.get("🔑")?, Some("🎉".as_bytes().to_vec()));

    Ok(())
}

#[test]
fn empty_value() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/empty_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("empty", b"")?;
    assert_eq!(store.get("empty")?, Some(vec![]));

    Ok(())
}

#[test]
fn many_keys() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/many_keys";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    // Insert 1000 keys
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        store.set(&key, value.as_bytes())?;
    }

    // Verify a sample
    assert_eq!(store.get("key_0")?, Some(b"value_0".to_vec()));
    assert_eq!(store.get("key_500")?, Some(b"value_500".to_vec()));
    assert_eq!(store.get("key_999")?, Some(b"value_999".to_vec()));

    let stats = store.stats();
    assert_eq!(stats.num_keys, 1000);

    Ok(())
}

#[test]
fn compaction_after_many_updates() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/compaction_updates";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    // Write same keys multiple times
    for round in 0..5 {
        for i in 0..100 {
            let key = format!("key_{}", i);
            let value = format!("value_{}_{}", i, round);
            store.set(&key, value.as_bytes())?;
        }
    }

    let stats_before = store.stats();
    store.compact()?;
    let stats_after = store.stats();

    // Should have reduced total bytes
    assert!(stats_after.total_bytes < stats_before.total_bytes);

    // Verify data integrity
    for i in 0..100 {
        let key = format!("key_{}", i);
        let expected = format!("value_{}_4", i); // Last round
        assert_eq!(store.get(&key)?, Some(expected.as_bytes().to_vec()));
    }

    Ok(())
}

#[test]
fn list_keys_works() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = "tests_data/list_keys";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir)?;

    store.set("a", b"1")?;
    store.set("b", b"2")?;
    store.set("c", b"3")?;

    let keys = store.list_keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"a".to_string()));
    assert!(keys.contains(&"b".to_string()));
    assert!(keys.contains(&"c".to_string()));

    Ok(())
}
