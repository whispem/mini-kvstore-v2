use std::fs;
use std::path::Path;


pub fn setup_test_dir(test_dir: &str) {
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    fs::create_dir_all(test_dir).unwrap();
}


pub fn cleanup_test_dir(test_dir: &str) {
    if Path::new(test_dir).exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
}
