# Mini KV Store v2 🦀

**A production-ready, segmented key-value storage engine built in Rust**

[![Rust Version](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![CI](https://img.shields.io/badge/CI-passing-brightgreen)](https://github.com/whispem/mini-kvstore-v2/actions)
[![Production Ready](https://img.shields.io/badge/status-production_ready-success)](https://github.com/whispem/mini-kvstore-v2)
[![Docker](https://img.shields.io/badge/docker-ready-2496ED?logo=docker&logoColor=white)](https://github.com/whispem/mini-kvstore-v2/blob/main/Dockerfile)
[![Performance](https://img.shields.io/badge/writes-240K_ops%2Fs-brightgreen)](https://github.com/whispem/mini-kvstore-v2#benchmarks)
[![Performance](https://img.shields.io/badge/reads-11M_ops%2Fs-brightgreen)](https://github.com/whispem/mini-kvstore-v2#benchmarks)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[Features](#-features) •
[Quick Start](#-quick-start) •
[Architecture](#-architecture) •
[API Documentation](#-api-documentation) •
[Benchmarks](#-benchmarks) •
[Contributing](#-contributing)

---

## 📚 About

Mini KV Store v2 is a high-performance, append-only key-value storage engine with HTTP API capabilities.  
Built as an educational project to explore storage engine fundamentals, it implements core database concepts like segmented logs, compaction, bloom filters, index snapshots, and crash recovery.

> 💡 **New:** [3-week learning journey](JOURNEY.md) — from Rust & database newbie to working engine.

**Why This Project?**  
Not just another key-value store, but a deep dive into real DB internals:  
- Segmented logs: write amplification, log-structured storage
- In-memory indexing: speed/memory tradeoffs
- Compaction: space reclamation, speed vs durability
- Bloom filters: fast "not-found" lookups
- Index snapshots: instant restarts
- HTTP API: async & production-tested

---

## ✨ Features

**Core Engine**  
- Durable, append-only log (fsync guarantees)
- Automatic segmented architecture
- Lightning-fast reads (O(1) in-memory HashMap index)
- Background compaction (auto space reclaim)
- CRC32 checksums for data integrity
- Index snapshots (5ms restarts)
- Tombstone deletions
- Bloom filters (negative lookups)

**Production Ready**  
- HTTP REST API (Axum)
- Interactive CLI (REPL)
- `/metrics` and `/health` endpoints
- Docker & docker-compose support
- Criterion & k6 benchmarks
- Full CI/CD (build, lint, test)
- Rate limiting (100MB body max)
- Unit/integration test suite

**Developer Experience**  
- Rich docs, many examples
- Modular, clean codebase
- Easy config via env vars
- Makefile with all tasks
- Pure safe Rust (no unsafe)

---

## 🚀 Quick Start

**Prerequisites:**  
- Rust 1.75+ ([install](https://rustup.rs/))
- Git

```bash
git clone https://github.com/whispem/mini-kvstore-v2
cd mini-kvstore-v2
cargo build --release
cargo test --release
```

**REPL CLI:**
```bash
cargo run --release
# mini-kvstore-v2 (type help for instructions)
# > set key "value"
# > get key
# > list
# > compact
# > quit
```

**HTTP server:**
```bash
cargo run --release --bin volume-server
# (config: PORT, VOLUME_ID, DATA_DIR env vars)
```

---

## 🌐 API Documentation

- `/health`: health/status
- `/metrics`: server stats
- `POST /blobs/:key`: store blob
- `GET /blobs/:key`: fetch blob
- `DELETE /blobs/:key`: delete blob
- `GET /blobs`: list all keys

Example:
```bash
curl -X POST http://localhost:8000/blobs/user:123 -d "Hello, World!"
curl http://localhost:8000/blobs/user:123
curl -X DELETE http://localhost:8000/blobs/user:123
```

---

## 🏗️ Architecture

```
[CLI / HTTP Client]
        │
    [Axum Server]
        │
   [Blob Storage]
        │
   [KVStore Core]
      /  |  \
 [Index][Segments][Bloom]
```
- Append-only data segments on disk
- HashMap and Bloom filter in RAM for O(1) queries
- Snapshot/compaction for instant restart & small disk use

---

## 📊 Benchmarks

**Apple M4, 16GB RAM:**  
- **Writes:** ~240,000 ops/sec
- **Reads:** ~11M ops/sec (in-memory)
- **Compaction:** ~80,000 keys/sec

Run:
```bash
cargo bench
./run_benchmark.sh
```

---

## 🐳 Docker

**Standalone:**
```bash
docker build -t mini-kvstore-v2:latest .
docker run -d -p 8000:8000 -v $(pwd)/data:/data --name kvstore mini-kvstore-v2:latest
```
**Cluster:**  
```bash
docker-compose up -d
# nodes: localhost:8001 ... :8003
docker-compose logs -f
```

---

## 🧪 Testing

```bash
cargo test --release
cargo test --release --test store_integration
make pre-commit    # fmt, clippy, test
```
- Extensive unit/integration + HTTP
- >80% coverage (goal)
- Benchmarks: Criterion + k6

---

## 📂 Project Structure

[Click to expand](#)

---

## 🗺️ Roadmap

- [x] Append-only log, crash recovery, compaction
- [x] Bloom filters, index snapshot
- [x] REPL, HTTP, Docker, CI/CD, metrics
- [ ] Range queries, WAL, compression
- [ ] Replication, LSM, Prometheus, Admin UI

---

## 🤔 Design Decisions

- Append-only: max I/O, easy recovery
- In-memory index: trade memory for O(1)
- Bloom filter: instant "not found"
- Rust: safety, perf, reliability

---

## 📚 Learning Resources

- [Database Internals](https://www.databass.dev/)
- [DDIA](https://dataintensive.net/)
- [sled](https://github.com/spacejam/sled)
- [Bitcask Paper](https://riak.com/assets/bitcask-intro.pdf)

---

## 🤝 Contributing

- 🐛 Report bugs
- 💡 Suggest features
- 🧪 Add tests
- 📖 Improve docs
- ⚡ Performance PRs
- [How to contribute](CONTRIBUTING.md)

---

## 🌍 Community

- Discord: [Rust Aix-Marseille](https://discord.gg/sXr9ZqBJ)
- LinkedIn: [Rust Aix-Marseille](https://www.linkedin.com/company/rust-aix-marseille-ram/)
- GitHub Discussions, Issues

---

## 📜 License

MIT — see [LICENSE](LICENSE)

---

## 👤 Author

**Em' ([@whispem](https://github.com/whispem))**

From languages to Rust/DB internals in 3 weeks. See [JOURNEY.md](JOURNEY.md).

> "The best way to learn is to build."

---

**Built with ❤️ in Rust** • [Back to Top](#mini-kv-store-v2-)
