# Changelog

All notable changes to this project will be documented in this file.

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
- Initial project structure and segmented log implementation
- In-memory index with key lookup
- Per-record checksums (CRC32)
- Manual compaction (compact command)
- Interactive CLI with REPL
- Unit tests in `src/lib.rs`
- Integration tests in `tests/`
- Performance benchmarks using Criterion
- Comprehensive API documentation with examples
- Example programs (basic_usage, compaction, persistence, large_dataset)

### Features
- Segmented append-only log storage
- Automatic segment rotation when full
- Tombstone-based deletion
- Persistence across restarts
- UTF-8 key/value support
- Statistics and monitoring
- CI/CD pipeline with GitHub Actions
