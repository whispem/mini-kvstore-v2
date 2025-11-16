use mini_kvstore_v2::store::Store;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::Path;

// Helper to ensure every test runs in a clean directory
fn setup_test_dir(path: &str) {
    let p = Path::new(path);
    let _ = remove_dir_all(p); // Ignore errors if folder doesn't exist
    create_dir_all(p).unwrap();
}

#[test]
fn can_set_and_get_value() {
    let test_dir = "tests_data/can_set_and_get_value";
    setup_test_dir(test_dir);

    let mut store = Store::new(test_dir);

    store.set("foo", "bar");
    assert_eq!(store.get("foo"), Some("bar".to_string()));
}

#[test]
fn can_delete_value() {
    let test_dir = "tests_data/can_delete_value";
    setup_test_dir(test_dir);

    let mut store = Store::new(test_dir);

    store.set("foo", "bar");
    store.delete("foo");
    assert_eq!(store.get("foo"), None);
}

#[test]
fn overwriting_value_updates_storage() {
    let test_dir = "tests_data/overwrite_value";
    setup_test_dir(test_dir);

    let mut store = Store::new(test_dir);

    store.set("foo", "1");
    store.set("foo", "2");

    assert_eq!(store.get("foo"), Some("2".to_string()));
}

#[test]
fn missing_key_returns_none() {
    let test_dir = "tests_data/missing_key";
    setup_test_dir(test_dir);

    let store = Store::new(test_dir);

    assert_eq!(store.get("does_not_exist"), None);
}

#[test]
fn delete_nonexistent_key_is_safe() {
    let test_dir = "tests_data/delete_nonexistent";
    setup_test_dir(test_dir);

    let mut store = Store::new(test_dir);

    // Should not panic or corrupt internal state
    store.delete("nope");

    assert_eq!(store.get("nope"), None);
}
