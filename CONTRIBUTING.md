# Contributing to mini-kvstore-v2

First off, thank you for considering contributing to mini-kvstore-v2! 🎉

This document provides guidelines and instructions for contributing to this project. 
Following these guidelines helps maintain code quality and makes the contribution process smooth for everyone.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Guidelines](#testing-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Project Structure](#project-structure)
- [Getting Help](#getting-help)

---

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to whispem@users.noreply.github.com.

---

## Getting Started

### Prerequisites

Before you begin, ensure you have:

- **Rust 1.75 or higher** - [Install Rust](https://rustup.rs/)
- **Git** - For version control
- **A code editor** - VS Code with rust-analyzer recommended
- **Docker** (optional) - For testing containerized deployments

### Setting Up Your Development Environment

1. **Fork the repository** on GitHub

2. **Clone your fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/mini-kvstore-v2
   cd mini-kvstore-v2
   ```

3. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/whispem/mini-kvstore-v2
   ```

4. **Install development tools**
   ```bash
   # Install rustfmt and clippy
   rustup component add rustfmt clippy
   
   # Optional: Install cargo-watch for auto-rebuilding
   cargo install cargo-watch
   
   # Optional: Install cargo-tarpaulin for coverage
   cargo install cargo-tarpaulin
   ```

5. **Build the project**
   ```bash
   cargo build --release
   ```

6. **Run tests to verify**
   ```bash
   cargo test --all --release
   ```

7. **Try the examples**
   ```bash
   cargo run --example basic_usage
   ```

---

## How Can I Contribute?

### 🐛 Reporting Bugs

Found a bug? Help us fix it!

1. **Check existing issues** - Your bug might already be reported
2. **Use the bug report template** - It helps us understand the problem
3. **Include:**
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - System information (OS, Rust version, architecture)
   - Relevant logs or error messages

**Example:**
```markdown
**Bug:** Compaction fails with large datasets

**Steps to reproduce:**
1. Insert 100,000 keys
2. Run `compact` command
3. Observe error: "segment not found"

**Environment:**
- OS: macOS 14.6.1
- Rust: 1.75.0
- Architecture: aarch64 (M1)

**Error log:**
```
Error: SegmentNotFound(5)
...
```
```

### 💡 Suggesting Features

Have an idea? We'd love to hear it!

1. **Check existing feature requests** - It might already be planned
2. **Use the feature request template**
3. **Explain:**
   - The problem you're trying to solve
   - Your proposed solution
   - Alternative solutions you've considered
   - Use cases and benefits

### 📖 Improving Documentation

Documentation improvements are always welcome:

- Fix typos or unclear explanations
- Add examples or use cases
- Improve API documentation
- Write tutorials or guides
- Translate documentation

### 🔧 Contributing Code

Ready to write some Rust? Great!

**Good first issues** are labeled with `good first issue` - these are great starting points.

**Areas that need help:**
- Performance optimizations
- Test coverage improvements
- New features from the roadmap
- Bug fixes
- Refactoring and code cleanup

---

## Development Workflow

### 1. Create a Feature Branch

```bash
# Update your fork
git checkout main
git pull upstream main

# Create a new branch
git checkout -b feature/my-awesome-feature
# or
git checkout -b fix/bug-description
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions or improvements
- `perf/` - Performance improvements

### 2. Make Your Changes

**Write clean, focused commits:**
- One logical change per commit
- Clear commit messages (see guidelines below)
- Keep commits small and reviewable

**Test as you go:**
```bash
# Run tests frequently
cargo test

# Watch mode for continuous testing
cargo watch -x test
```

### 3. Ensure Code Quality

Before committing, run:

```bash
# Format code
cargo fmt --all

# Check for lints
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all --release

# Or use the Makefile
make pre-commit
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add background compaction support"
```

### 5. Push and Create Pull Request

```bash
git push origin feature/my-awesome-feature
```

Then create a Pull Request on GitHub.

---

## Code Style Guidelines

### Rust Style

We follow **standard Rust conventions** with some project-specific rules:

**Formatting:**
- Use `cargo fmt` with the project's `rustfmt.toml`
- 100-character line limit
- 4-space indentation
- Unix line endings (LF)

**Naming:**
```rust
// Good
fn calculate_segment_size() -> u64 { ... }
let user_id = 42;
const MAX_SEGMENT_SIZE: u64 = 16_777_216;

// Avoid
fn calcSegSize() -> u64 { ... }
let userId = 42;
const max_segment_size: u64 = 16777216;
```

**Error Handling:**
```rust
// Good - Use Result and ? operator
pub fn write_record(&mut self, key: &str, value: &[u8]) -> Result<()> {
    self.validate_input(key)?;
    self.write_to_segment(key, value)?;
    Ok(())
}

// Avoid - Don't use unwrap() in library code
pub fn write_record(&mut self, key: &str, value: &[u8]) {
    self.validate_input(key).unwrap();
    self.write_to_segment(key, value).unwrap();
}
```

**Documentation:**
```rust
/// Compacts the storage by rewriting all live keys to new segments.
///
/// This operation:
/// - Collects all live keys from the in-memory index
/// - Writes them to fresh segments
/// - Removes old segment files
///
/// # Errors
///
/// Returns `StoreError::CompactionFailed` if segment cleanup fails.
///
/// # Examples
///
/// ```
/// let mut store = KVStore::open("data")?;
/// store.compact()?;
/// ```
pub fn compact(&mut self) -> Result<()> {
    // Implementation
}
```

### File Organization

**Import order:**
```rust
// 1. Standard library
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};

// 2. External crates
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

// 3. Internal modules
use crate::store::error::{Result, StoreError};
use crate::store::stats::StoreStats;
```

### Code Philosophy

This project favors:

✅ **Clarity over cleverness** - Code should be easy to understand
✅ **Explicitness over magic** - Make behavior obvious
✅ **Simplicity over optimization** - Optimize when necessary, not prematurely
✅ **Safety over performance** - Use unsafe only when justified

❌ **Avoid:**
- Unnecessary `unsafe` blocks
- Overly complex abstractions
- Magic numbers (use named constants)
- Giant functions (break them down)

---

## Testing Guidelines

### Writing Tests

**Every change should include tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut store = KVStore::open("test_data").unwrap();
        store.set("key", b"value").unwrap();
        
        let result = store.get("key").unwrap();
        assert_eq!(result, Some(b"value".to_vec()));
        
        // Cleanup
        std::fs::remove_dir_all("test_data").ok();
    }

    #[test]
    fn test_compaction_preserves_data() {
        // Arrange
        let mut store = KVStore::open("test_compact").unwrap();
        for i in 0..100 {
            store.set(&format!("key_{}", i), b"value").unwrap();
        }
        
        // Act
        store.compact().unwrap();
        
        // Assert
        for i in 0..100 {
            let result = store.get(&format!("key_{}", i)).unwrap();
            assert!(result.is_some());
        }
        
        // Cleanup
        std::fs::remove_dir_all("test_compact").ok();
    }
}
```

**Test categories:**

1. **Unit tests** - Test individual functions/modules
   ```bash
   cargo test --lib
   ```

2. **Integration tests** - Test end-to-end workflows
   ```bash
   cargo test --test store_integration
   ```

3. **Doc tests** - Verify example code in documentation
   ```bash
   cargo test --doc
   ```

4. **Benchmarks** - Measure performance
   ```bash
   cargo bench
   ```

### Test Coverage

Aim for high coverage, especially for:
- Core storage operations
- Error handling paths
- Edge cases (empty store, large datasets, etc.)

```bash
# Generate coverage report
cargo tarpaulin --verbose --all-features --workspace --timeout 300
```

### Integration Tests

Create test utilities in `tests/common/mod.rs`:

```rust
pub fn setup_test_dir(path: &str) {
    let _ = std::fs::remove_dir_all(path);
    std::fs::create_dir_all(path).unwrap();
}

pub fn cleanup_test_dir(path: &str) {
    let _ = std::fs::remove_dir_all(path);
}
```

Use them in your tests:

```rust
use common::{setup_test_dir, cleanup_test_dir};

#[test]
fn my_integration_test() {
    setup_test_dir("test_data");
    // ... test code ...
    cleanup_test_dir("test_data");
}
```

---

## Commit Message Guidelines

We follow **Conventional Commits** for clear history:

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks
- `ci:` - CI/CD changes

### Scopes (optional)

- `store` - Storage engine
- `api` - HTTP API
- `cli` - Command-line interface
- `volume` - Volume module
- `bench` - Benchmarks

### Examples

```bash
# Good commits
feat(api): add health check endpoint
fix(store): handle empty segments correctly
docs(readme): add installation instructions
perf(compaction): optimize live key collection
test(integration): add crash recovery tests

# With body and footer
feat(store): implement background compaction

Add automatic compaction triggered when segment count exceeds threshold.
Compaction runs in a background thread without blocking operations.

Closes #42
```

### Commit Message Rules

- **Use imperative mood** - "add feature" not "added feature"
- **Be concise but descriptive** - 50 chars max for subject
- **Capitalize subject line**
- **No period at the end of subject**
- **Separate subject from body with blank line**
- **Wrap body at 72 characters**
- **Reference issues** - Use "Closes #123" or "Fixes #123"

---

## Pull Request Process

### Before Submitting

**Checklist:**
- [ ] Code follows style guidelines
- [ ] All tests pass (`cargo test --all --release`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Documentation updated (if needed)
- [ ] CHANGELOG.md updated (for significant changes)
- [ ] Examples updated (if API changed)
- [ ] Commit messages follow conventions

### Creating the PR

1. **Use the PR template** - Fill it out completely

2. **Write a clear description:**
   - What does this PR do?
   - Why is this change needed?
   - How was it tested?
   - Are there any breaking changes?

3. **Link related issues:**
   ```markdown
   Fixes #123
   Relates to #456
   ```

4. **Add appropriate labels:**
   - `bug` - Bug fixes
   - `enhancement` - New features
   - `documentation` - Docs changes
   - `breaking-change` - API breaking changes

### PR Title Format

Use the same format as commit messages:

```
feat(store): add bloom filter support
fix(api): correct status code for missing keys
docs: update contributing guidelines
```

### Review Process

**What happens next:**

1. **Automated checks** - CI runs tests, lints, formatting checks
2. **Code review** - Maintainer reviews your code
3. **Discussion** - Address feedback, make changes if needed
4. **Approval** - Once approved, your PR will be merged
5. **Celebration** 🎉 - You're a contributor!

**During review:**
- Be responsive to feedback
- Don't take criticism personally
- Explain your reasoning if you disagree
- Make requested changes promptly
- Squash commits if asked

### Merge Requirements

Your PR must:
- ✅ Pass all CI checks
- ✅ Have at least one approval
- ✅ Be up-to-date with main branch
- ✅ Have no merge conflicts
- ✅ Follow coding standards

---

## Project Structure

Understanding the codebase:

```
mini-kvstore-v2/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── main.rs             # CLI binary
│   ├── store/              # Core storage engine
│   │   ├── engine.rs       # Main KVStore implementation
│   │   ├── compaction.rs   # Compaction logic
│   │   ├── error.rs        # Error types
│   │   ├── index.rs        # In-memory index
│   │   ├── segment.rs      # Segment abstraction
│   │   ├── stats.rs        # Statistics
│   │   └── config.rs       # Configuration
│   └── volume/             # HTTP API layer
│       ├── main.rs         # Volume server binary
│       ├── server.rs       # Server setup
│       ├── handlers.rs     # HTTP request handlers
│       ├── storage.rs      # BlobStorage wrapper
│       └── config.rs       # Volume config
├── tests/                  # Integration tests
├── examples/               # Example programs
├── benches/                # Performance benchmarks
└── .github/                # CI/CD workflows
```

**Key modules:**

- `store::engine` - Core key-value operations
- `store::compaction` - Space reclamation
- `volume::handlers` - HTTP endpoint handlers
- `volume::storage` - High-level blob API

---

## Getting Help

### Resources

- 📖 **Documentation** - Run `cargo doc --open`
- 📚 **Examples** - Check the `examples/` directory
- 🗺️ **Architecture** - See `JOURNEY.md` for design decisions
- 🐛 **Issues** - Browse existing issues for context

### Communication

- **GitHub Issues** - For bugs and feature requests
- **GitHub Discussions** - For questions and ideas
- **Email** - whispem@users.noreply.github.com

### Questions?

Don't hesitate to ask! Common questions:

**Q: I'm new to Rust, can I still contribute?**
A: Absolutely! Start with issues labeled `good first issue`.

**Q: How long does review take?**
A: Usually within a few days. Be patient!

**Q: Can I work on multiple issues?**
A: Sure, but finish one before starting another.

**Q: My tests pass locally but fail in CI**
A: Check platform-specific issues (Windows vs Linux/macOS).

**Q: Should I update CHANGELOG.md?**
A: Yes, for significant changes. Follow the format in the file.

---

## Recognition

All contributors are recognized in:
- GitHub contributors list
- CHANGELOG.md for significant contributions
- Project README (for major features)

---

## Thank You! 🙏

Your contributions make this project better. Whether you're:
- Reporting a bug
- Suggesting a feature
- Improving documentation
- Writing code
- Helping others

**Every contribution matters!**

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---


**Happy coding! 🦀**

[Back to README](README.md) | [Code of Conduct](CODE_OF_CONDUCT.md) | [License](LICENSE)
