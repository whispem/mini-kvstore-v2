# Mini KV Store v2 🦀


**A production-ready, segmented key-value store built in Rust**

[![CI](https://img.shields.io/badge/CI-passing-brightgreen)](https://github.com/whispem/mini-kvstore-v2/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

[Features](#-features) •
[Quick Start](#-quick-start) •
[API Documentation](#-api-reference) •
[Architecture](#-architecture) •
[Benchmarks](#-benchmarks) •
[Contributing](#-contributing)


---

## 📚 About

Mini KV Store v2 is a high-performance, append-only key-value storage engine with HTTP API capabilities. Built as an educational project to explore storage engine fundamentals, it implements core database concepts like segmented logs, write-ahead logging, compaction, and crash recovery.

> 💡 **New!** Read about my [3-week learning journey](JOURNEY.md) from Rust beginner to building a working storage engine.

### Why This Project?

This isn't just another key-value store—it's a deep dive into how databases work under the hood. Every feature teaches fundamental concepts:

- **Segmented logs** → Understanding write amplification and log-structured storage
- **In-memory indexing** → Learning trade-offs between memory and disk I/O
- **Compaction** → Exploring space reclamation strategies
- **Checksums** → Implementing data integrity guarantees
- **HTTP API** → Building async production services

---

## ✨ Features

### Core Storage Engine
- 🔐 **Durable & crash-safe** - Append-only log with fsync guarantees
- 📦 **Segmented architecture** - Automatic rotation when segments reach size limits
- ⚡ **Lightning-fast reads** - O(1) lookups via in-memory HashMap index
- 🗜️ **Manual compaction** - Space reclamation by rewriting live keys
- ✅ **Data integrity** - CRC32 checksums on every record
- 💾 **Persistence** - Automatic index rebuild from segments on restart
- 🪦 **Tombstone deletions** - Efficient deletion in append-only architecture

### Production Ready
- 🌐 **HTTP REST API** - Async server built with Axum
- 🖥️ **Interactive CLI** - REPL for testing and exploration
- 📊 **Monitoring** - Built-in stats and health endpoints
- 🧪 **Comprehensive tests** - Unit, integration, and benchmark suites
- 🐳 **Docker support** - Multi-container deployment with docker-compose
- 📈 **Performance benchmarks** - Criterion-based regression testing
- 🔧 **CI/CD pipeline** - Automated testing, linting, and builds

### Developer Experience
- 📖 **Rich documentation** - API docs, examples, and learning resources
- 🎨 **Clean architecture** - Modular design with clear separation of concerns
- 🛠️ **Makefile included** - Simple commands for common tasks
- 🎯 **Zero unsafe code** - Pure safe Rust implementation

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository

### Installation

```bash
# Clone the repository
git clone https://github.com/whispem/mini-kvstore-v2
cd mini-kvstore-v2

# Build the project
cargo build --release

# Run tests to verify
cargo test --release
```

### Running the CLI

```bash
# Start the interactive REPL
cargo run --release

# You'll see:
# mini-kvstore-v2 (type help for instructions)
# >
```

**CLI Commands:**

```bash
> set name "Alice"          # Store a key-value pair
OK

> get name                  # Retrieve a value
Alice

> set age "30"              # Store another pair
OK

> list                      # List all keys
  name
  age

> delete name               # Remove a key
Deleted

> stats                     # Show storage statistics
Store Statistics:
  Keys: 1
  Segments: 1
  Total size: 0.00 MB
  Active segment: 1
  Oldest segment: 0

> compact                   # Reclaim space
Compaction finished

> quit                      # Exit
```

### Running the HTTP Server

```bash
# Start the volume server on port 8000
cargo run --release --bin volume-server

# Or with custom configuration
PORT=9000 VOLUME_ID=my-vol DATA_DIR=./data cargo run --release --bin volume-server
```

---

## 🌐 REST API Reference

### Health Check

```bash
GET /health

# Response (200 OK)
{
  "status": "healthy",
  "volume_id": "vol-1",
  "keys": 42,
  "segments": 2,
  "total_mb": 1.5
}
```

### Store a Blob

```bash
POST /blobs/:key
Content-Type: application/octet-stream

# Example
curl -X POST http://localhost:8000/blobs/user:123 \
  -H "Content-Type: application/octet-stream" \
  -d "Hello, World!"

# Response (201 Created)
{
  "key": "user:123",
  "etag": "3e25960a",
  "size": 13,
  "volume_id": "vol-1"
}
```

### Retrieve a Blob

```bash
GET /blobs/:key

# Example
curl http://localhost:8000/blobs/user:123

# Response (200 OK)
Hello, World!

# Not Found (404)
{
  "error": "Blob not found"
}
```

### Delete a Blob

```bash
DELETE /blobs/:key

# Example
curl -X DELETE http://localhost:8000/blobs/user:123

# Response (204 No Content)
```

### List All Blobs

```bash
GET /blobs

# Response (200 OK)
[
  "user:123",
  "user:456",
  "config:settings"
]
```

---

## 🏗️ Architecture

### System Overview

```
┌─────────────────────────────────────────────────┐
│              Client Applications                 │
│         (CLI, HTTP Clients, Rust API)           │
└────────────────────┬────────────────────────────┘
                     │
         ┌───────────▼───────────┐
         │     HTTP Server       │
         │      (Axum)           │
         └───────────┬───────────┘
                     │
         ┌───────────▼───────────┐
         │   BlobStorage Layer   │
         │   (High-level API)    │
         └───────────┬───────────┘
                     │
         ┌───────────▼───────────┐
         │      KVStore Core     │
         │   (Storage Engine)    │
         └───────────┬───────────┘
                     │
    ┌────────────────┼────────────────┐
    │                │                │
    ▼                ▼                ▼
┌────────┐    ┌────────────┐    ┌──────────┐
│ Index  │    │  Segment   │    │  Stats   │
│HashMap │    │  Manager   │    │ Tracker  │
└────────┘    └──────┬─────┘    └──────────┘
                     │
         ┌───────────▼───────────┐
         │    Segment Files      │
         │ segment-0000.dat      │
         │ segment-0001.dat      │
         │ segment-0002.dat      │
         └───────────────────────┘
```

### Data Flow

**Write Path:**
1. Client calls `set(key, value)`
2. KVStore appends operation to active segment
3. Segment writes: `[op_code][key_len][key][val_len][value]`
4. In-memory index updated: `key → (segment_id, offset, length)`
5. fsync() ensures durability

**Read Path:**
1. Client calls `get(key)`
2. Index lookup: O(1) HashMap access
3. Returns value directly from memory (rebuilt on startup)

**Delete Path:**
1. Client calls `delete(key)`
2. Tombstone appended to segment
3. Key removed from in-memory index

**Compaction:**
1. Collect all live keys from index
2. Write to new segments
3. Delete old segments
4. Index remains unchanged (still valid)

### On-Disk Format

Each segment file contains a sequence of records:

```
╔════════════════════════════════════════════╗
║              Segment Record                ║
╠════════════════════════════════════════════╣
║  op_code    │ 1 byte  │ 0=SET, 1=DELETE   ║
║  key_len    │ 4 bytes │ u32 little-endian ║
║  key        │ N bytes │ UTF-8 string      ║
║  [val_len]  │ 4 bytes │ Only if op=SET    ║
║  [value]    │ M bytes │ Only if op=SET    ║
╚════════════════════════════════════════════╝
```

**Example SET record:**
```
[0x00][0x04 0x00 0x00 0x00]['u''s''e''r'][0x05 0x00 0x00 0x00]['A''l''i''c''e']
```

**Example DELETE record:**
```
[0x01][0x04 0x00 0x00 0x00]['u''s''e''r']
```

---

## 💻 Programmatic Usage

### Basic Operations

```rust
use mini_kvstore_v2::KVStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open or create a store
    let mut store = KVStore::open("my_database")?;
    
    // Store data
    store.set("user:1:name", b"Alice")?;
    store.set("user:1:email", b"alice@example.com")?;
    
    // Retrieve data
    if let Some(name) = store.get("user:1:name")? {
        println!("Name: {}", String::from_utf8_lossy(&name));
    }
    
    // Delete data
    store.delete("user:1:email")?;
    
    // List all keys
    for key in store.list_keys() {
        println!("Key: {}", key);
    }
    
    // Get statistics
    let stats = store.stats();
    println!("Keys: {}, Segments: {}", stats.num_keys, stats.num_segments);
    
    // Manual compaction
    store.compact()?;
    
    Ok(())
}
```

### Using BlobStorage (Higher-Level API)

```rust
use mini_kvstore_v2::volume::BlobStorage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut storage = BlobStorage::new("data", "vol-1".to_string())?;
    
    // Store with metadata
    let meta = storage.put("image:123", b"<binary data>")?;
    println!("Stored: etag={}, size={}", meta.etag, meta.size);
    
    // Retrieve
    if let Some(data) = storage.get("image:123")? {
        println!("Retrieved {} bytes", data.len());
    }
    
    // Delete
    storage.delete("image:123")?;
    
    Ok(())
}
```

---

## 📊 Benchmarks

Run the included benchmarks to measure performance on your hardware:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_set

# Generate HTML reports
cargo bench -- --verbose
```

### Sample Results

**Environment:** Apple M4, 16GB RAM, macOS 15

```
set_operations/10       time: [45.2 µs 46.1 µs 47.2 µs]
set_operations/100      time: [402 µs 410 µs 419 µs]
set_operations/1000     time: [4.12 ms 4.18 ms 4.25 ms]

get_existing_key        time: [89.3 ns 91.2 ns 93.5 ns]

compact_1000_keys       time: [12.3 ms 12.5 ms 12.8 ms]
```

**Throughput:**
- **Writes:** ~240,000 ops/sec
- **Reads:** ~11M ops/sec
- **Compaction:** ~80,000 keys/sec

### HTTP API Benchmark

```bash
# Run k6 benchmark (requires k6 installation)
./run_benchmark.sh

# Custom configuration
./run_benchmark.sh 1 8000 9000 32 60s 1048576
```

---

## 🐳 Docker Deployment

### Single Container

```bash
# Build image
docker build -t mini-kvstore-v2:latest .

# Run container
docker run -d \
  -p 8000:8000 \
  -e VOLUME_ID=vol-1 \
  -e DATA_DIR=/data \
  -v $(pwd)/data:/data \
  --name kvstore \
  mini-kvstore-v2:latest
```

### Multi-Volume Cluster

```bash
# Start 3-node cluster
docker-compose up -d

# Nodes available at:
# - http://localhost:8001 (vol-1)
# - http://localhost:8002 (vol-2)
# - http://localhost:8003 (vol-3)

# View logs
docker-compose logs -f

# Stop cluster
docker-compose down
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --release

# Run with output
cargo test --release -- --nocapture

# Run specific test
cargo test --release test_compaction

# Run heavy tests (long-running)
cargo test --release --features heavy-tests

# Run integration tests only
cargo test --release --test store_integration
```

**Test Coverage:**
- Unit tests for core components
- Integration tests for workflows
- Example programs as executable tests
- Benchmark suite for performance regression

---

## 📂 Project Structure

```
mini-kvstore-v2/
├── src/
│   ├── lib.rs                  # Public API exports
│   ├── main.rs                 # CLI binary entrypoint
│   ├── store/
│   │   ├── engine.rs           # Core KVStore implementation
│   │   ├── compaction.rs       # Compaction logic
│   │   ├── error.rs            # Error types
│   │   ├── index.rs            # In-memory index
│   │   ├── segment.rs          # Segment abstraction
│   │   ├── stats.rs            # Statistics tracking
│   │   └── config.rs           # Configuration
│   └── volume/
│       ├── main.rs             # Volume server binary
│       ├── server.rs           # Axum server setup
│       ├── handlers.rs         # HTTP handlers
│       ├── storage.rs          # BlobStorage wrapper
│       └── config.rs           # Volume configuration
├── tests/
│   ├── common/                 # Test utilities
│   └── store_integration.rs    # Integration tests
├── examples/
│   ├── basic_usage.rs          # Getting started
│   ├── compaction.rs           # Compaction demo
│   ├── persistence.rs          # Crash recovery
│   ├── large_dataset.rs        # Performance test
│   └── volume_usage.rs         # Volume API demo
├── benches/
│   └── kvstore_bench.rs        # Criterion benchmarks
├── .github/
│   └── workflows/
│       └── ci.yml              # GitHub Actions CI
├── Cargo.toml                  # Dependencies
├── Dockerfile                  # Container image
├── docker-compose.yml          # Multi-node setup
├── Makefile                    # Build automation
├── README.md                   # This file
├── JOURNEY.md                  # Learning journey
├── CONTRIBUTING.md             # Contribution guide
├── LICENSE                     # MIT License
└── CHANGELOG.md                # Version history
```

---

## 🛠️ Development

### Using the Makefile

```bash
make help           # Show all available commands
make build          # Build release binary
make test           # Run all tests
make bench          # Run benchmarks
make fmt            # Format code
make clippy         # Run lints
make docs           # Generate documentation
make clean          # Clean build artifacts
make examples       # Run all examples
make docker         # Build Docker image
make docker-up      # Start cluster
```

### Code Quality

The project maintains high code quality standards:

- **Formatting:** `cargo fmt` with custom rules
- **Linting:** `cargo clippy` with strict settings
- **Testing:** Comprehensive test suite
- **CI:** Automated checks on every push
- **Documentation:** Inline docs for all public APIs

```bash
# Pre-commit checks
make pre-commit

# This runs:
# - cargo fmt (formatting)
# - cargo clippy (linting)
# - cargo test (all tests)
```

---

## 🗺️ Roadmap

### Completed ✅
- [x] Append-only log architecture
- [x] In-memory HashMap index
- [x] Crash recovery & persistence
- [x] Manual compaction
- [x] CRC32 checksums
- [x] Interactive CLI/REPL
- [x] HTTP REST API (Axum)
- [x] Comprehensive benchmarks
- [x] Docker support
- [x] CI/CD pipeline

### In Progress 🚧
- [ ] Background compaction (automatic)
- [ ] Index snapshots for faster restarts
- [ ] Bloom filters for negative lookups

### Planned 📋
- [ ] Range queries (requires sorted segments)
- [ ] Write-ahead log (WAL) for stronger guarantees
- [ ] Compression (LZ4/Zstd)
- [ ] Replication protocol
- [ ] LSM-tree / SSTable support
- [ ] gRPC API option
- [ ] Metrics/observability (Prometheus)
- [ ] Admin dashboard (web UI)

---

## 🤔 Design Decisions

### Why Append-Only?

Append-only architectures offer several advantages:
- **Sequential writes** - Maximizes disk throughput
- **Simplified concurrency** - No in-place updates
- **Natural versioning** - Easy to implement MVCC
- **Crash recovery** - Incomplete writes don't corrupt data

### Why In-Memory Index?

Trading memory for speed is worth it for most workloads:
- **O(1) lookups** - No disk seeks
- **Rebuild on startup** - Index is derived data
- **Simple implementation** - Standard HashMap

### Why Manual Compaction?

Manual control teaches the fundamentals:
- **Understand trade-offs** - Space vs. performance
- **Predictable behavior** - No surprise pauses
- **Learning tool** - See compaction effects clearly

*Production systems would use background compaction.*

### Why Rust?

- **Memory safety** - No segfaults or data races
- **Performance** - Zero-cost abstractions
- **Ecosystem** - Excellent libraries (Axum, Tokio)
- **Learning curve** - Forces good design decisions

---

## 📚 Learning Resources

### Storage Engines
- [Database Internals](https://www.databass.dev/) by Alex Petrov
- [Designing Data-Intensive Applications](https://dataintensive.net/) by Martin Kleppmann
- [Log-Structured Merge-Trees](http://www.benstopford.com/2015/02/14/log-structured-merge-trees/) by Ben Stopford
- [Bitcask Paper](https://riak.com/assets/bitcask-intro.pdf) - Inspiration for this project

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Real-World Examples
- [sled](https://github.com/spacejam/sled) - Embedded database in Rust
- [RocksDB](https://github.com/facebook/rocksdb) - LSM-tree KV store
- [LevelDB](https://github.com/google/leveldb) - Google's KV storage library

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Ways to Contribute

- 🐛 **Report bugs** - Open an issue with reproduction steps
- 💡 **Suggest features** - Share your ideas
- 📖 **Improve docs** - Fix typos, add examples
- 🧪 **Add tests** - Increase coverage
- ⚡ **Optimize** - Profile and improve performance
- 🎨 **Refactor** - Clean up code

### Development Setup

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/mini-kvstore-v2
cd mini-kvstore-v2

# Create feature branch
git checkout -b feature/my-new-feature

# Make changes, then test
make test
make clippy

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/my-new-feature

# Open a Pull Request
```

---

## 📜 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

```
MIT License

Copyright (c) 2025 Em'

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED...
```

---

## 🙏 Acknowledgments

- **Rust Community** - For excellent documentation and welcoming forums
- **Database Internals** - Alex Petrov's book was invaluable
- **DDIA** - Martin Kleppmann's book for system design thinking
- **Bitcask** - For the elegant append-only log design
- **RocksDB/LevelDB** - For LSM-tree inspiration

---

## 👤 Author

**Em' ([@whispem](https://github.com/whispem))**

From literature & languages background to building storage engines in 3 weeks. Read about the journey in [JOURNEY.md](JOURNEY.md).

> *"The best way to learn is to build."*

---

## 📬 Contact & Support

- 🐛 **Issues:** [GitHub Issues](https://github.com/whispem/mini-kvstore-v2/issues)
- 💬 **Discussions:** [GitHub Discussions](https://github.com/whispem/mini-kvstore-v2/discussions)
- 📧 **Email:** whispem@users.noreply.github.com

---

## 🌟 Star History

If you find this project helpful, please consider giving it a star! ⭐

It helps others discover the project and motivates continued development.

---


**Built with ❤️ in Rust**

[⬆ Back to Top](#mini-kv-store-v2-)
