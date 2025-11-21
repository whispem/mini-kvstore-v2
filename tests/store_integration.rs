//! Integration tests for the KVStore.

use mini_kvstore_v2::KVStore;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::Path;

fn setup_test_dir(path: &str) {
    let p = Path::new(path);
    let _ = remove_dir_all(p);
    create_dir_all(p).expect("Failed to create test directory");
}

fn cleanup_test_dir(path: &str) {
    let _ = remove_dir_all(Path::new(path));
}

#[test]
fn can_set_and_get_value() {
    let test_dir = "tests_data/int_can_set_and_get_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();
    store.set("foo", b"bar").unwrap();
    assert_eq!(store.get("foo").unwrap(), Some(b"bar".to_vec()));

    cleanup_test_dir(test_dir);
}

#[test]
fn can_delete_value() {
    let test_dir = "tests_data/int_can_delete_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();
    store.set("foo", b"bar").unwrap();
    store.delete("foo").unwrap();
    assert_eq!(store.get("foo").unwrap(), None);

    cleanup_test_dir(test_dir);
}

#[test]
fn overwriting_value_updates_storage() {
    let test_dir = "tests_data/int_overwrite_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();
    store.set("foo", b"1").unwrap();
    store.set("foo", b"2").unwrap();
    assert_eq!(store.get("foo").unwrap(), Some(b"2".to_vec()));

    cleanup_test_dir(test_dir);
}

#[test]
fn missing_key_returns_none() {
    let test_dir = "tests_data/int_missing_key";
    setup_test_dir(test_dir);

    let store = KVStore::open(test_dir).unwrap();
    assert_eq!(store.get("does_not_exist").unwrap(), None);

    cleanup_test_dir(test_dir);
}

#[test]
fn delete_nonexistent_key_is_safe() {
    let test_dir = "tests_data/int_delete_nonexistent";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();
    store.delete("nope").unwrap();
    assert_eq!(store.get("nope").unwrap(), None);

    cleanup_test_dir(test_dir);
}

#[test]
fn persistence_after_reopen() {
    let test_dir = "tests_data/int_persistence";
    setup_test_dir(test_dir);

    {
        let mut store = KVStore::open(test_dir).unwrap();
        store.set("persistent", b"value").unwrap();
    }

    let store = KVStore::open(test_dir).unwrap();
    assert_eq!(store.get("persistent").unwrap(), Some(b"value".to_vec()));

    cleanup_test_dir(test_dir);
}

#[test]
fn compaction_preserves_data() {
    let test_dir = "tests_data/int_compaction";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    store.set("key1", b"value1").unwrap();
    store.set("key2", b"value2").unwrap();
    store.set("key3", b"value3").unwrap();

    store.set("key1", b"updated1").unwrap();
    store.delete("key2").unwrap();

    store.compact().unwrap();

    assert_eq!(store.get("key1").unwrap(), Some(b"updated1".to_vec()));
    assert_eq!(store.get("key2").unwrap(), None);
    assert_eq!(store.get("key3").unwrap(), Some(b"value3".to_vec()));

    cleanup_test_dir(test_dir);
}

#[test]
fn large_value_handling() {
    let test_dir = "tests_data/int_large_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    // 1MB value
    let large_value = vec![b'x'; 1024 * 1024];
    store.set("large", &large_value).unwrap();

    let retrieved = store.get("large").unwrap();
    assert_eq!(retrieved, Some(large_value));

    cleanup_test_dir(test_dir);
}

#[test]
fn utf8_keys_and_values() {
    let test_dir = "tests_data/int_utf8";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    store.set("key", "value".as_bytes()).unwrap();
    store.set("🔑", "🎉".as_bytes()).unwrap();

    assert_eq!(store.get("key").unwrap(), Some(b"value".to_vec()));
    assert_eq!(store.get("🔑").unwrap(), Some("🎉".as_bytes().to_vec()));

    cleanup_test_dir(test_dir);
}

#[test]
fn empty_value() {
    let test_dir = "tests_data/int_empty_value";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    store.set("empty", b"").unwrap();
    assert_eq!(store.get("empty").unwrap(), Some(vec![]));

    cleanup_test_dir(test_dir);
}

#[test]
fn many_keys() {
    let test_dir = "tests_data/int_many_keys";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    // Insert 1000 keys
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        store.set(&key, value.as_bytes()).unwrap();
    }

    // Verify a sample
    assert_eq!(store.get("key_0").unwrap(), Some(b"value_0".to_vec()));
    assert_eq!(store.get("key_500").unwrap(), Some(b"value_500".to_vec()));
    assert_eq!(store.get("key_999").unwrap(), Some(b"value_999".to_vec()));

    let stats = store.stats();
    assert_eq!(stats.num_keys, 1000);

    cleanup_test_dir(test_dir);
}

#[test]
fn compaction_after_many_updates() {
    let test_dir = "tests_data/int_compaction_updates";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    // Write same keys multiple times
    for round in 0..5 {
        for i in 0..100 {
            let key = format!("key_{}", i);
            let value = format!("value_{}_{}", i, round);
            store.set(&key, value.as_bytes()).unwrap();
        }
    }

    let stats_before = store.stats();
    store.compact().unwrap();
    let stats_after = store.stats();

    // Should have reduced total bytes
    assert!(stats_after.total_bytes < stats_before.total_bytes);

    // Verify data integrity
    for i in 0..100 {
        let key = format!("key_{}", i);
        let expected = format!("value_{}_4", i); // Last round
        assert_eq!(store.get(&key).unwrap(), Some(expected.as_bytes().to_vec()));
    }

    cleanup_test_dir(test_dir);
}

#[test]
fn list_keys_works() {
    let test_dir = "tests_data/int_list_keys";
    setup_test_dir(test_dir);

    let mut store = KVStore::open(test_dir).unwrap();

    store.set("a", b"1").unwrap();
    store.set("b", b"2").unwrap();
    store.set("c", b"3").unwrap();

    let keys = store.list_keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"a".to_string()));
    assert!(keys.contains(&"b".to_string()));
    assert!(keys.contains(&"c".to_string()));

    cleanup_test_dir(test_dir);
}

#[test]
fn persistence_after_compaction_and_reopen() {
    let test_dir = "tests_data/int_persistence_compaction";
    setup_test_dir(test_dir);

    // Write and compact
    {
        let mut store = KVStore::open(test_dir).unwrap();
        for i in 0..10 {
            store.set("key", format!("value_{}", i).as_bytes()).unwrap();
        }
        store.compact().unwrap();
    }

    // Reopen and verify
    {
        let store = KVStore::open(test_dir).unwrap();
        assert_eq!(store.get("key").unwrap(), Some(b"value_9".to_vec()));
    }

    cleanup_test_dir(test_dir);
}
