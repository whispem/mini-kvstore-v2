# mini-kvstore-v2

A log-structured key-value storage engine in Rust, with an interactive prompt and an HTTP server. Started on November 16, 2025, it was my first store with a real on-disk format: records with checksums, written to segment files, and replayed at startup.

[JOURNEY.md](JOURNEY.md) tells how it was built, in the first weeks of my learning Rust.

## Features

- **Segments.** Records are appended to segment files (`segment-N.dat`); a new segment starts once the active one reaches 16 MB.
- **Checksums.** Every record carries a CRC32 of its contents.
- **Replay.** On startup, the store reads every segment in order to rebuild its in-memory index.
- **Index snapshot.** The index is saved to `index.snapshot` (after compaction, and when the server shuts down) and loaded at startup.
- **Bloom filter.** A Bloom filter of the written keys is kept next to the index.
- **Compaction.** A `compact` command at the prompt; the HTTP server also compacts periodically when segments pile up.
- **HTTP server.** A blob API built with Axum, with `/health` and `/metrics` endpoints and a 100 MB limit on request bodies.
- **Tooling.** Criterion benchmarks, examples, a Dockerfile and a three-node `docker-compose.yml`, and a CI workflow (check, format, lint, tests).

## Run it

Requires Rust 1.75 or later.

```bash
git clone https://github.com/whispem/mini-kvstore-v2
cd mini-kvstore-v2
cargo build --release
cargo test --release
```

### The prompt

```bash
cargo run --release
```

The prompt stores its data in `./db`.

```text
mini-kvstore-v2 (type help for instructions)
> set name Alice
OK
> get name
Alice
> list
  name
> delete name
Deleted
> get name
Key not found
```

Commands: `set <key> <value>`, `get <key>`, `delete <key>`, `list`, `compact`, `stats`, `help`, `quit`.

### The HTTP server

```bash
cargo run --release --bin volume-server
```

It is configured through environment variables:

| Variable | Default |
| --- | --- |
| `PORT` | `9002` |
| `VOLUME_ID` | `vol-1` |
| `DATA_DIR` | `data` |
| `COMPACTION_THRESHOLD` | `5` (segments) |
| `COMPACTION_INTERVAL_SECS` | `60` |

| Route | Effect |
| --- | --- |
| `POST /blobs/:key` | Stores the request body under the key. |
| `GET /blobs/:key` | Returns the value. |
| `DELETE /blobs/:key` | Deletes the key. |
| `GET /blobs` | Lists the keys, as JSON. |
| `GET /health` | Status, number of keys and segments. |
| `GET /metrics` | Storage statistics, as JSON. |

```text
$ curl -X POST localhost:9002/blobs/user:123 -d 'Hello, World!'
{"key":"user:123","etag":"ec4ac3d0","size":13,"volume_id":"vol-1"}
$ curl localhost:9002/blobs/user:123
Hello, World!
$ curl localhost:9002/blobs
["user:123"]
```

### Docker

```bash
docker build -t mini-kvstore-v2 .
docker run -d -p 8000:8000 -v "$(pwd)/data:/data" mini-kvstore-v2
```

The image listens on port 8000. `docker-compose up -d` starts three independent volumes, on ports 8001 to 8003.

## Record format

```text
magic (2 bytes: F0 F1) | op (1 byte: 1 = set, 2 = delete) | key length (u32, LE) | value length (u32, LE) | key | value | CRC32 (u32, LE)
```

The CRC32 covers the op, both lengths, the key and the value.

## Limits

- All values are kept in memory as well as on disk, so the data set must fit in RAM.
- Writes are flushed to the operating system, but not forced to disk with `fsync`.
- A single node: the three volumes of `docker-compose.yml` do not replicate each other.
- No authentication on the HTTP server.

## Benchmarks

```bash
cargo bench
./run_benchmark.sh
```

## Before and after

- [Tiny-Log-Structured-KV-Store](https://github.com/whispem/Tiny-Log-Structured-KV-Store), three days earlier: a single append-only log, without checksums.
- [mini-kvstore-go](https://github.com/whispem/mini-kvstore-go): this engine translated into Go.
- [minikv](https://github.com/whispem/minikv): the distributed version, with Raft and two-phase commit.

## Community

I'm part of [Rust Aix-Marseille](https://www.linkedin.com/company/rust-aix-marseille-ram/) ([Discord](https://discord.gg/sXr9ZqBJ)). Issues and discussions are welcome here; see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT. See [LICENSE](LICENSE).
