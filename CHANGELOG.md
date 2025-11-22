# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-11-21

### Added
- **HTTP REST API** with Axum framework for async operations
  - `/health` endpoint for monitoring and stats
  - `/blobs` endpoints for PUT, GET, DELETE, LIST operations
  - ETag generation for stored blobs
- **Volume server** binary (`volume-server`) as dedicated HTTP service
- **BlobStorage** abstraction layer for high-level blob operations
- **Docker support**
  - Dockerfile for containerized deployment
  - docker-compose.yml for multi-volume cluster (3 nodes)
- **Comprehensive benchmarking**
  - k6 integration for HTTP load testing
  - `run_benchmark.sh` script for automated performance testing
  - Performance metrics and reporting
- **HTTP handler tests** with tokio test runtime
- **Makefile** for streamlined development workflow
  - Build, test, bench, docker commands
  - Pre-commit checks and CI targets
- **Enhanced examples**
  - `volume_usage.rs` demonstrating BlobStorage API
  - Updated all examples with better error handling
- **GitHub Actions improvements**
  - Added coverage reporting workflow
  - Docker build validation
  - Security audit checks

### Changed
- **Project structure** reorganized with `volume/` module
  - Separated concerns: storage engine vs HTTP API
  - Cleaner module boundaries
- **Error handling** improved across all modules
  - Better error messages in handlers
  - Consistent error types throughout
- **Documentation** significantly enhanced
  - README.md completely rewritten with modern format
  - Added architecture diagrams
  - Comprehensive API documentation
  - Added JOURNEY.md with learning story
- **CLI binary** now separate from volume server
  - Distinct binaries for different use cases
  - Better separation of concerns

### Technical
- **New dependencies**
  - `axum = "0.7"` - HTTP server framework
  - `tokio` with full features for async runtime
  - `tower` for middleware support
  - `serde` and `serde_json` for serialization
- **Build improvements**
  - Profile optimizations in Cargo.toml
  - LTO and codegen-units tuning
  - Strip symbols in release builds
- **Testing**
  - Integration tests for HTTP handlers
  - Async test utilities with tokio::test
  - Enhanced test coverage

### Performance
- Benchmarked at ~240K writes/sec and ~11M reads/sec (M4)
- HTTP API adds minimal overhead (~1-2ms latency)
- Compaction performance: ~80K keys/sec

### Fixed
- Segment file discovery now handles non-sequential IDs
- Better handling of empty stores on startup
- Fixed potential race conditions with mutex guards

## [0.2.0] - 2025-11-18

### Changed
- **Breaking:** Replaced `std::io::Result` with custom `StoreError` type
- Improved error handling with `thiserror` for the library
- Binary (CLI) now uses `anyhow` for simpler error handling
- All examples updated to use the new error types

### Added
- Custom error type `StoreError` with specific variants:
  - `ActiveSegmentNotFound`
  - `SegmentNotFound(usize)`
  - `SegmentDisappeared`
  - `Io(io::Error)` with transparent conversion
- Better error messages throughout the codebase

### Technical
- Added `thiserror = "1.0"` dependency for library
- Moved `anyhow` to dev-dependencies (binary only)
- Improved API documentation with proper error handling examples

## [0.1.0] - 2025-11-16

### Added
- **Initial project structure** and segmented log implementation
- **In-memory index** with key lookup using HashMap
- **Per-record checksums** (CRC32) for data integrity
- **Manual compaction** via `compact` command
- **Interactive CLI** with REPL interface
- **Core operations**
  - `set` - Store or update key-value pairs
  - `get` - Retrieve values by key
  - `delete` - Tombstone-based deletion
  - `list` - Show all keys
  - `stats` - Display storage statistics
- **Persistence** across restarts with automatic index rebuild
- **UTF-8 support** for keys and values
- **Unit tests** in `src/lib.rs`
- **Integration tests** in `tests/` directory
- **Performance benchmarks** using Criterion
- **Comprehensive API documentation** with examples
- **Example programs**
  - `basic_usage.rs` - Getting started guide
  - `compaction.rs` - Compaction demonstration
  - `persistence.rs` - Crash recovery example
  - `large_dataset.rs` - Performance testing with 10K keys

### Features
- **Segmented append-only log** storage
- **Automatic segment rotation** when segments reach size limit
- **Tombstone-based deletion** for append-only architecture
- **Crash recovery** via segment replay
- **Statistics and monitoring** built-in

### Technical
- **CI/CD pipeline** with GitHub Actions
  - Automated testing on Linux, macOS, Windows
  - Code formatting checks with `cargo fmt`
  - Linting with `cargo clippy`
  - Security audits
- **Development tools**
  - EditorConfig for consistent formatting
  - Clippy configuration
  - rustfmt configuration
- **Documentation**
  - Issue templates for bugs and features
  - Pull request template
  - Contributing guidelines
  - Code of Conduct

### Dependencies
- `crc32fast = "1.4"` - Fast CRC32 checksums
- `thiserror = "1.0"` - Error handling
- `criterion = "0.5"` - Benchmarking framework

## [Unreleased]

### Planned
- Background/automatic compaction
- Index snapshots for faster restarts
- Bloom filters for negative lookups
- Range query support
- Write-ahead log (WAL) for stronger durability
- Compression support (LZ4/Zstd)
- Replication protocol
- gRPC API option
- Metrics export (Prometheus)
- Admin web dashboard

---

## Release Notes

### Version 0.3.0 Highlights

This release transforms mini-kvstore-v2 from a learning project into a production-ready storage system with a full HTTP REST API. The addition of the volume server, Docker support, and comprehensive benchmarking makes it suitable for real-world use cases.

**Key improvements:**
- 🌐 Full HTTP REST API with Axum
- 🐳 Production-ready Docker deployment
- 📊 Professional benchmarking suite
- 🏗️ Clean separation of storage engine and API layer
- 📈 Performance validated with real metrics

### Migration Guide: 0.2.0 → 0.3.0

**No breaking changes to the core KVStore API.** Existing code using `KVStore` directly continues to work without modifications.

**New capabilities:**
```rust
// Old way (still works)
let mut store = KVStore::open("data")?;
store.set("key", b"value")?;

// New way (with BlobStorage)
let mut storage = BlobStorage::new("data", "vol-1".to_string())?;
let meta = storage.put("key", b"value")?;
println!("ETag: {}", meta.etag);
```

**HTTP API** is entirely new - no migration needed.

### Version 0.2.0 Highlights

Focused on error handling improvements and code quality. The introduction of custom error types makes the API more robust and easier to use.

### Version 0.1.0 Highlights

Initial release with all core storage engine functionality. A complete, working key-value store with persistence, compaction, and comprehensive testing.

---

## Contributors

- [@whispem](https://github.com/whispem) - Creator and maintainer

## Links

- [Repository](https://github.com/whispem/mini-kvstore-v2)
- [Issues](https://github.com/whispem/mini-kvstore-v2/issues)
- [Discussions](https://github.com/whispem/mini-kvstore-v2/discussions)

---

**Note:** This project follows semantic versioning. Version numbers follow the format MAJOR.MINOR.PATCH where:
- MAJOR: Incompatible API changes
- MINOR: Backwards-compatible functionality additions
- PATCH: Backwards-compatible bug fixes
