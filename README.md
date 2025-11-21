# Mini KV Store v2 🦀

> 📚 **New!** Read about [my learning journey](JOURNEY.md) building this project

A segmented, append-only key-value store implemented in Rust. **Now with:**
- Async HTTP API (Axum, REST endpoints)
- SVG and benchmarks (Plotters, Criterion)
- Persistent, crash-safe, and multi-segment log store
- Interactive CLI and programmatic API
- Full CI with code linting, formatting and tests

---

## 🎯 Project Goals

To explore and teach storage engine fundamentals like segmented logs, in-memory indexing, checksums, compaction, multi-threading, and now async servers.

- **Segmented append-only logs** → durability & write-ahead
- **In-memory index** → instant lookups
- **Checksums** → integrity
- **Manual compaction** → space reclaim
- **Persistence** → restart & recovery

---

## ✨ Features

- Persistent storage, crash recovery
- Segmented log files that rotate when full
- In-memory index (rebuilt on startup)
- Tombstone-based deletion, append-only
- Manual compaction/reclamation
- Per-record checksums (CRC32)
- Interactive CLI for exploration/testing
- UTF-8 keys and values
- HTTP REST API (Axum, async)
    - `/blobs` (GET: list keys)
    - `/blobs/:key` (PUT, GET, DELETE: key ops)
    - `/health` (GET: volume stats)
- Programmatic API (`KVStore`)
- Stats (key count, segments, total size)
- CLI and REPL
- Benchmarks (Criterion)
- Graph/report support (Plotters: SVG)
- Multi-threaded concurrency (Rayon)
- Logging, error reporting (`thiserror`, `anyhow`)
- Automated CI, lint, tests
- Pretty terminal/color output (`anes`, `anstyle`)

---

## 🚀 Quick Start
```bash
# Clone
git clone https://github.com/whispem/mini-kvstore-v2
cd mini-kvstore-v2

# Build & run CLI
cargo run --release

# Run as HTTP server (Axum, REST)
cargo run --release -- --volume data --id my-vol
```

### REST API Examples
```bash
# Get health/stats
curl http://localhost:8000/health

# Set a key
curl -X POST http://localhost:8000/blobs/user -d 'Alice'

# Get a key
curl http://localhost:8000/blobs/user

# Delete a key
curl -X DELETE http://localhost:8000/blobs/user

# List all blobs
curl http://localhost:8000/blobs
```

### CLI Session

```
> set name Alice
OK
> get name
Alice
> set age 25
OK
> list
Keys (2): name, age
> delete name
Deleted
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
└──────┬──────────────┘
       │
┌──────▼──────────────┐
│  Segment Manager    │
└──────┬──────────────┘
       │
┌──────▼──────────────┐
│  Segment Files      │  segment-0000.dat, ...
└─────────────────────┘
```

### Example Programmatic Use

```rust
use mini_kvstore_v2::KVStore;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = KVStore::open("my_data")?;
    store.set("user:1:name", b"Alice")?;
    let name = store.get("user:1:name")?;
    Ok(())
}
```

---

## 📊 On-Disk Format

### Segment File Layout

```
segment-NNNN.dat:
| Record |
| key_len | value_len | key | value |
(Tombstone: value_len = u64::MAX)
```

### Index Structure

```rust
HashMap<String, (usize, u64, u64)> // key → segment, offset, len
```

---

## 🛠️ Usage Examples

- **Programmatic API (Rust)**
- **HTTP REST API**
- **CLI and REPL commands**

#### CLI

| Command         | Description                                      |
|-----------------|--------------------------------------------------|
| set <k> <v>     | Store or update                                  |
| get <k>         | Retrieve                                         |
| delete <k>      | Tombstone/delete                                 |
| list            | Show all keys                                    |
| compact         | Merge segments                                   |
| stats           | Store metrics                                    |
| help            | List commands                                    |
| quit/exit       | Exit                                             |

---

## 📂 Project Structure

```
mini-kvstore-v2/
├── src/
│   ├── lib.rs              # API & tests
│   ├── main.rs             # CLI/HTTP startup
│   └── volume/             # Core implementation
│       ├── handlers.rs     # HTTP handlers
│       ├── storage.rs      # Storage engine
│       └── ...
├── tests/
├── examples/
├── benches/
├── .github/workflows/ci.yml
```

---

## 🧪 Testing & Benchmarking

```bash
cargo test --all --release
cargo bench
```

---

## 🟩 Roadmap

- [x] Append-only logs
- [x] In-memory indexing
- [x] Persistence/crash recovery
- [x] Manual compaction
- [x] CLI and REPL
- [x] REST API (Axum)
- [x] Benchmarks
- [x] SVG/plotting
- [x] Segment statistics
- [ ] Background/automatic compaction (Next!)
- [ ] Index snapshots
- [ ] Bloom filters
- [ ] Range queries
- [ ] WAL for durability
- [ ] Network mode (TCP)
- [ ] LSM-tree/SSTable support
- [ ] More!

---

## 📦 Dependencies

- axum, tokio, serde, criterion, plotters, plotters-svg, clap, parking_lot, anyhow, thiserror, rayon, anestyle, anes, etc.

---

## 🤔 Design choices

- Append-only, single-writer policy
- In-memory hashmap for speed
- Manual compaction for learning
- Rust for safety, performance, clarity

---

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md)

---

## 📄 License

MIT License - see [LICENSE](LICENSE)

---

## 👤 About

Built by [@whispem](https://github.com/whispem) as an exploration of storage engine internals.

*"The best way to learn is to build."*

---

**If you’re learning Rust or databases, feel free to explore, fork, and experiment!**
