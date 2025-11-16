# Mini KV Store v2 🦀

Second iteration of my small key–value store, implemented in Rust.  
This version explores a slightly more realistic storage design: a segmented append-only log, an in-memory index, per-record checksums, and a manual compaction step.  
The goal remains the same — build a clear, minimal system and learn every piece of storage plumbing step by step.

---

## 🧩 What This Is

A compact, educational key–value store with a tiny CLI front-end.  
It’s intentionally focused and incremental — each feature is implemented to teach a specific concept in storage engines:

- segmented append-only log for on-disk persistence
- in-memory index mapping keys to segment locations
- checksums to detect corrupt records
- manual compaction to reclaim space and merge segments

This project prioritizes clarity and readability over performance or production readiness.

---

## ⚙️ Installation & Running

Clone the repository:

```bash
git clone https://github.com/whispem/mini-kvstore-v2
cd mini-kvstore-v2
```

Build and run the CLI:

```bash
cargo run --release
```

By default the store writes segment files to a `data/` directory next to the repo; you can override this with CLI flags (see `--help`).

Quick example session:

```
> set name Alice
OK
> get name
Alice
> set count 42
OK
> delete name
Deleted
> get name
Key not found
> compact
Compaction started -> compacted_0001.segment
Compaction finished, old segments removed
> quit
```

---

## 📌 Current Status

**Started:** November 16, 2025  
**Current phase:** segmented log + in-memory index + manual compaction

Implemented so far:

- Segmented append-only log on disk (segment files with monotonically increasing ids)
- In-memory index (HashMap) mapping keys to (segment_id, offset, length)
- `set(key, value)` — append record to active segment and update index
- `get(key)` — consult in-memory index and read record from segment file
- `delete(key)` — append a tombstone record and remove key from index
- Per-record checksum to validate reads
- Manual `compact` command that rewrites live keys into a new compacted segment and rotates files
- A simple CLI for interactive experimentation

The implementation aims to be small and easy to read; each module is self-contained so you can follow the data flow from API call to disk.

---

## 🧠 Design overview (short)

- Segmented log: Writes are appended to an active segment file (e.g. `0001.segment`). When the segment reaches a target size the store rolls to a new segment file.
- In-memory index: For fast reads, the store keeps a HashMap of key → (segment_id, offset, len). On startup, the store rebuilds the index by scanning existing segments.
- Checksums: Each record includes a checksum (CRC32 or similar) to detect corruption on read; corrupted records are ignored and logged.
- Tombstones: Deletes are recorded as tombstone entries so the log remains append-only.
- Manual compaction: A `compact` operation creates a new segment and copies the latest live values into it, then atomically switches files and removes old compacted segments. Compacting reclaims space and reduces read scanning.
- Durability: Appends are synced to disk (fsync) before acknowledging writes (configurable for experimentation).

---

## 🔍 File & on-disk layout (example)

- data/
  - 0001.segment
  - 0002.segment
  - compacted_0003.segment
- Each segment is a sequence of records:
  - [record-header: key_len, value_len, flags, checksum] + [key bytes] + [value bytes]
- Index (in memory): HashMap<String, (segment_id, offset, len, tombstone_flag)>

---

## 📈 Learning Roadmap

Planned/possible future work (implemented as small, focused steps):

- Background or automatic compaction (triggered by size/ratio heuristics)
- Checkpointing / snapshot of the in-memory index to speed up startup
- Concurrency and multi-threaded compaction with safe coordination
- More robust checksums and corruption recovery strategies
- Benchmarks and microbench tests
- Simple network protocol (TCP) to turn this into a tiny KV server
- LSM-ish layering experiments (memtable + immutable segments)
- Config-driven persistence (segment size, fsync policy, retention)

Each feature will be added incrementally so the codebase stays easy to read and reason about.

---

## 📚 Why Rust?

Rust is an excellent fit for this kind of project because it gives:

- memory safety without a garbage collector
- explicit ownership and clear boundaries for state
- low-level control for working with files and buffers
- tooling and ergonomics (cargo, fmt, clippy) that encourage small, correct experiments

It helps me focus on the storage ideas without accidentally introducing memory bugs.

---

## 🧾 Usage & CLI (short)

Typical commands supported by the CLI:

- set <key> <value>       — append a key/value pair
- get <key>               — retrieve the latest value for a key
- delete <key>            — delete a key (tombstone)
- list                    — list known keys (index-based)
- compact                 — run manual compaction to produce a compacted segment
- info                    — show store status (segments, index size, data directory)
- help                    — show help and flags
- quit / exit             — exit the CLI

Run `cargo run -- --help` for the exact flags and options.

---

## 🔧 Configuration & tuning

The store exposes a few knobs for experimentation:

- segment max size (bytes) — when to roll a new segment
- fsync policy — sync on every write vs batched
- compaction threshold — when compaction should be considered
- data directory — path for segments and checkpoints

These are intentionally simple to keep the learning focused on behavior rather than configuration complexity.

---

## 🧰 Resources I’m learning from

- Rust book and std library docs
- Blog posts and papers about log-structured storage and database internals
- Articles and tutorials on WAL, LSM trees, and compaction strategies
- Implementation notes and blog posts from other tiny KV projects
I update this list as I learn.

---

## 🗒️ Notes

This project is a learning exercise — the code will evolve as I better understand systems programming and storage design. Contributions and suggestions are welcome, but I'll prioritize small, well-reasoned changes that keep the codebase understandable.

Built while exploring Rust and storage fundamentals — November 2025 🦀

If you spot anything that could be written in a more idiomatic or elegant Rust style, I’m always curious to understand why.
