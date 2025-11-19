# Mini KV Store v2 🦀

A segmented append-only key-value store implemented in Rust as a learning project.

This is my second iteration exploring storage engine fundamentals: segmented logs, in-memory indexing, checksums, and compaction. 
Built to understand every layer of persistent storage, from file I/O to data structures.

---

## 🎯 Project Goals

This project exists to learn by building. 
Each feature teaches a specific concept:

- **Segmented append-only logs** → Understanding write-ahead logging and durability
- **In-memory indexing** → Fast lookups without scanning entire files  
- **Checksums** → Data integrity and corruption detection
- **Manual compaction** → Space reclamation and merge strategies
- **Persistence** → Rebuilding state from disk on restart

Code clarity is prioritized over optimization. 
Every design decision is intentional and documented.

---

## ✨ Features

- **Persistent storage** with automatic crash recovery
- **Segmented log files** that rotate when full
- **In-memory index** rebuilt on startup for O(1) key lookups
- **Tombstone-based deletion** maintaining append-only semantics
- **Manual compaction** to merge segments and reclaim space
- **Per-record checksums** (CRC32) to detect corruption
- **Interactive CLI** for exploration and testing
- **UTF-8 support** for keys and values

---

## 🚀 Quick Start
```bash
# Clone the repository
git clone https://github.com/whispem/mini-kvstore-v2
cd mini-kvstore-v2

# Build and run
cargo run --release
```

The store creates a `data/` directory for segment files. Override with `--help` for options.

### Example Session
```
> set name Alice
OK
> get name
Alice
> set age 25
OK
> delete name
Deleted
> list
Keys (1):
  age
> compact
Compaction finished
> quit
```

---

## 📖 How It Works

### Architecture
```
┌─────────────┐
│  CLI/API    │
└──────┬──────┘
       │
┌──────▼──────────────┐
│  In-Memory Index    │  HashMap<Key, (SegmentId, Offset, Length)>
│  (Fast Lookups)     │
└──────┬──────────────┘
       │
┌──────▼──────────────┐
│  Segment Manager    │  Handles rotation, compaction
└──────┬──────────────┘
       │
┌──────▼──────────────┐
│  Segment Files      │  segment-0000.dat, segment-0001.dat, ...
│  (Append-Only)      │  [key_len|value_len|key|value] records
└─────────────────────┘
```

### Write Path

1. **Append** to active segment file
2. **fsync** to guarantee durability
3. **Update** in-memory index with location
4. **Rotate** to new segment if size limit reached

### Read Path

1. **Lookup** key in index → (segment_id, offset, length)
2. **Seek** to offset in segment file
3. **Read** and validate checksum
4. **Return** value or None

### Compaction

1. Scan index for all live keys
2. Read their values from old segments
3. Write consolidated data to new segment
4. Atomically swap files and update index
5. Delete old segments

---

## 🛠️ Usage Examples

### Programmatic API
```rust
use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = KVStore::open("my_data")?;
    
    // Basic operations
    store.set("user:1:name", b"Alice")?;
    store.set("user:1:email", b"alice@example.com")?;
    
    if let Some(name) = store.get("user:1:name")? {
        println!("Name: {}", String::from_utf8_lossy(&name));
    }
    
    store.delete("user:1:email")?;
    
    // Maintenance
    store.compact()?;
    
    // Inspection
    let stats = store.stats();
    println!("Keys: {}, Segments: {}", stats.num_keys, stats.num_segments);
    
    Ok(())
}
```

### CLI Commands

| Command | Description |
|---------|-------------|
| `set <key> <value>` | Store or update a key-value pair |
| `get <key>` | Retrieve a value |
| `delete <key>` | Remove a key (tombstone) |
| `list` | Show all keys in the index |
| `compact` | Merge segments and reclaim space |
| `stats` | Display store metrics |
| `help` | Show available commands |
| `quit` / `exit` | Close the store |

---

## 📂 Project Structure
```
mini-kvstore-v2/
├── src/
│   ├── lib.rs              # Public API and tests
│   ├── main.rs             # CLI REPL
│   └── store/
│       ├── engine.rs       # Core KVStore implementation
│       ├── segment.rs      # Segment file management
│       ├── index.rs        # In-memory index (HashMap)
│       ├── compaction.rs   # Compaction logic
│       ├── error.rs        # Error types (thiserror)
│       ├── config.rs       # Configuration
│       ├── stats.rs        # Statistics tracking
│       └── record.rs       # Record format (planned)
├── tests/
│   └── store_integration.rs
├── examples/
│   ├── basic_usage.rs
│   ├── compaction.rs
│   ├── persistence.rs
│   └── large_dataset.rs
├── benches/
│   └── kvstore_bench.rs    # Criterion benchmarks
└── .github/workflows/
    └── ci.yml              # Automated testing
```

---

## 🧪 Testing & Benchmarking
```bash
# Run all tests
cargo test --all --release

# Run integration tests
cargo test --test store_integration

# Run examples with assertions
cargo run --example basic_usage
cargo run --example compaction
cargo run --example persistence
cargo run --example large_dataset

# Benchmark performance
cargo bench
```

Current benchmark results (10K keys):
- Insert: ~5,000-10,000 ops/sec
- Read: ~50,000-100,000 ops/sec
- Compaction (1,000 keys): ~20-50ms

---

## 🗺️ Learning Roadmap

### ✅ Implemented (v0.2.0)
- [x] Segmented append-only log
- [x] In-memory index with HashMap
- [x] Basic operations (set, get, delete)
- [x] Manual compaction
- [x] Custom error types with `thiserror`
- [x] Persistence and crash recovery
- [x] CLI with REPL
- [x] Integration tests
- [x] Benchmarks with Criterion

### 🎯 Next Steps
- [ ] Background/automatic compaction (trigger on threshold)
- [ ] Index snapshots (faster startup for large stores)
- [ ] Bloom filters to reduce disk seeks
- [ ] Multi-threaded compaction with safe coordination
- [ ] Write-ahead log (WAL) for enhanced durability
- [ ] Simple TCP protocol (network KV server)
- [ ] Range queries and prefix scans
- [ ] LSM-tree inspired architecture (memtable + SSTables)

Each feature will be added incrementally with thorough documentation.

---

## 📊 On-Disk Format

### Segment File Layout
```
segment-NNNN.dat:
┌─────────────────────────────────────────┐
│ Record 1                                │
│ ┌─────────────────────────────────────┐ │
│ │ key_len   (8 bytes, u64)            │ │
│ │ value_len (8 bytes, u64)            │ │
│ │ key       (key_len bytes)           │ │
│ │ value     (value_len bytes)         │ │
│ └─────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│ Record 2 ...                            │
├─────────────────────────────────────────┤
│ Record 3 ...                            │
└─────────────────────────────────────────┘

Tombstone: value_len = u64::MAX (no value bytes)
```

### Index Structure (In-Memory)
```rust
HashMap<String, (usize, u64, u64)>
         ↓       ↓      ↓    ↓
        key   seg_id offset len
```

---

## ⚙️ Configuration

Default settings (can be customized via `StoreConfig`):
```rust
StoreConfig {
    data_dir: "data",
    segment_size: 1MB,           // Rotate after 1MB
    fsync: FsyncPolicy::Always,  // Sync on every write
    compaction_threshold: 3,     // Compact after 3 segments
}
```

---

## 🤔 Design Decisions

### Why append-only?
Sequential writes are fast and simple. 
Deletes become tombstones; compaction reclaims space later.

### Why in-memory index?
Scanning files for every read is too slow. 
The index trades memory for speed (O(1) lookups).

### Why manual compaction?
Simplicity first. 
Automatic compaction adds complexity (background threads, locking). 
Manual control helps understand the tradeoffs.

### Why Rust?
- Memory safety without GC
- Explicit ownership clarifies data flow
- Zero-cost abstractions for systems programming
- Excellent tooling (`cargo`, `clippy`, `rustfmt`)

---

## 🐛 Known Limitations

- **No concurrency**: Single-threaded only (no locks yet)
- **No WAL**: Uncommitted writes lost on crash (current fsync policy helps)
- **Linear index rebuild**: Startup time grows with data size
- **No compression**: All data stored as-is
- **No bloom filters**: Every missing key requires disk I/O
- **No transactions**: Operations are isolated but not atomic across multiple keys

These are intentional tradeoffs for learning. Future iterations will address them.

---

## 📚 Learning Resources

Resources I'm studying while building this:

- **Books**: *Database Internals* (Alex Petrov), *Designing Data-Intensive Applications* (Martin Kleppmann)
- **Papers**: Log-Structured Merge Trees, Bitcask (Riak's storage engine)
- **Projects**: RocksDB source code, BadgerDB, sled
- **Rust**: The Rust Book, Rust by Example, std docs

---

## 🤝 Contributing

This is primarily a learning project, but I welcome:

- **Bug reports** with reproduction steps
- **Suggestions** on more idiomatic Rust patterns
- **Questions** about design decisions (I love explaining my thinking!)

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Please keep changes small and focused**—I want to understand every line of code I merge.

---

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

**Latest (v0.2.0 - 2025-11-19):**
- Custom error types with `thiserror`
- Improved error messages
- Updated all examples

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 👤 About

Built by [@whispem](https://github.com/whispem) as a hands-on exploration of storage engine internals.

Started learning Rust: October 27, 2025  
Project started: November 16, 2025

*"The best way to learn is to build."*

---

**⭐ If you're learning Rust or databases, feel free to explore, fork, and experiment!**
