use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mini_kvstore_v2::KVStore;
use std::fs::remove_dir_all;

fn setup_bench_dir(path: &str) {
    let _ = remove_dir_all(path);
    std::fs::create_dir_all(path).unwrap();
}

fn bench_set(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let test_dir = format!("bench_data/set_{}", size);
            setup_bench_dir(&test_dir);
            let mut store = KVStore::open(&test_dir).unwrap();

            b.iter(|| {
                for i in 0..size {
                    let key = format!("key_{}", i);
                    let value = format!("value_{}", i);
                    store.set(&key, value.as_bytes()).unwrap();
                }
            });

            let _ = remove_dir_all(&test_dir);
        });
    }
    group.finish();
}

fn bench_get(c: &mut Criterion) {
    let test_dir = "bench_data/get";
    setup_bench_dir(test_dir);
    let mut store = KVStore::open(test_dir).unwrap();

    // Pre-populate with data
    for i in 0..1000 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        store.set(&key, value.as_bytes()).unwrap();
    }

    c.bench_function("get_existing_key", |b| {
        b.iter(|| {
            let result = store.get(black_box("key_500")).unwrap();
            black_box(result);
        });
    });

    let _ = remove_dir_all(test_dir);
}

fn bench_compaction(c: &mut Criterion) {
    c.bench_function("compact_1000_keys", |b| {
        b.iter_with_setup(
            || {
                let test_dir = "bench_data/compact";
                setup_bench_dir(test_dir);
                let mut store = KVStore::open(test_dir).unwrap();

                // Write same keys multiple times
                for round in 0..5 {
                    for i in 0..1000 {
                        let key = format!("key_{}", i);
                        let value = format!("value_{}_{}", i, round);
                        store.set(&key, value.as_bytes()).unwrap();
                    }
                }
                store
            },
            |mut store| {
                store.compact().unwrap();
            },
        );
    });
}

criterion_group!(benches, bench_set, bench_get, bench_compaction);
criterion_main!(benches);
