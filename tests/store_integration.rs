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

    store.compact().unwrap();

    // Verify data integrity (most important)
    for i in 0..100 {
        let key = format!("key_{}", i);
        let expected = format!("value_{}_4", i); // Last round
        assert_eq!(store.get(&key).unwrap(), Some(expected.as_bytes().to_vec()));
    }

    // Verify key count is correct
    let stats = store.stats();
    assert_eq!(stats.num_keys, 100, "Should have exactly 100 keys after compaction");

    cleanup_test_dir(test_dir);
}
