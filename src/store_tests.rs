#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, remove_dir_all};
    use std::path::Path;

    // Helper for clean test directories
    fn setup_test_dir(path: &str) {
        let _ = remove_dir_all(path);
        create_dir_all(path).unwrap();
    }

    #[test]
    fn can_set_and_get_value() {
        setup_test_dir("data_test");
        let mut store = Store::new("data_test");
        store.set("foo", "bar");
        assert_eq!(store.get("foo"), Some("bar".to_string()));
    }

    #[test]
    fn can_delete_value() {
        setup_test_dir("data_test");
        let mut store = Store::new("data_test");
        store.set("foo", "bar");
        store.delete("foo");
        assert_eq!(store.get("foo"), None);
    }

    // Add more tests: compaction, corrupted records, etc.
}
