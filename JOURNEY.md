# Learning Rust Through Building a Storage Engine

## Timeline

**October 27, 2025**: Started learning Rust  
**November 16, 2025**: Started this project (mini-kvstore-v2)  
**November 21, 2025**: 364 commits, working segmented log KV store with HTTP API

## Why This Project?

I didn't want to build another todo app. 
I wanted to understand how storage engines actually work under the hood.

Coming from Swift and a literature background, I've learned that you understand a system best when you build it yourself. 
Reading about LSM trees and compaction is one thing—implementing them is where the real learning happens.

This project gave me something concrete: a key-value store with segmented logs, in-memory indexing, CRC32 checksums, manual compaction, and an HTTP API. 
Every feature taught me something fundamental about Rust.

## What I Built

### Core Features
- **Segmented append-only log**: Writes go to an active segment that rotates when full
- **In-memory HashMap index**: Maps `key → (segment_id, offset, length)` for instant lookups
- **CRC32 checksums**: Data integrity validation on every record
- **Manual compaction**: Rewrites live keys to new segments, removes old data
- **HTTP REST API**: Axum-based async server with `/blobs` endpoints
- **Persistence**: Crash recovery by rebuilding index from segment files
- **CLI REPL**: Interactive shell for testing and exploration

### The Architecture

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
│  Segment Files      │  segment-0000.dat, segment-0001.dat, ...
└─────────────────────┘

## Week-by-Week Breakdown

### Week 1: Fighting the Borrow Checker (Oct 27 - Nov 2)

**What clicked**: Ownership isn't just about memory safety—it's about designing clear data flow.

**Common mistakes**:
- Trying to mutate while holding immutable references
- Not understanding when to use `&`, `&mut`, or owned values
- Fighting the compiler instead of listening to what it was teaching me

**Key insight**: The borrow checker forces you to think about who owns what and when. 
This felt restrictive at first, but it made me write clearer code. 
If I couldn't express my logic without fighting the borrow checker, my logic was probably unclear to begin with.

**Resources that helped**:
- [The Rust Book](https://doc.rust-lang.org/book/) - Chapter 4 on Ownership
- Writing small functions that did one thing well
- `cargo clippy` teaching me better patterns

### Week 2: File I/O and Storage Primitives (Nov 3 - Nov 9)

**What I learned**:
- `BufReader` and `BufWriter` for efficient I/O
- When to call `sync_all()` for durability guarantees
- How to structure binary data on disk
- Error propagation with `Result<T>` and the `?` operator

**The "aha" moment**: Realizing that my on-disk format needed to be:
1. Simple enough to implement correctly
2. Efficient enough to scan during index rebuild
3. Extensible enough for future features

I settled on: `[key_len: u64][value_len: u64][key bytes][value bytes]`

Using `u64::MAX` as a tombstone marker felt elegant—it's a value that would never occur naturally (no one stores 18 exabytes in a single record).

**Mistakes I made**:
- Initially forgot to `seek(SeekFrom::End(0))` before writing, causing data corruption
- Didn't sync after writes, lost data during testing
- Used `unwrap()` everywhere instead of proper error handling

**How I fixed them**:
// Before: panic on error
let offset = self.file.seek(SeekFrom::End(0)).unwrap();

// After: propagate errors
let offset = self.file.seek(SeekFrom::End(0))?;

### Week 3: Compaction, Testing, and Polish (Nov 10 - Nov 21)

**The complexity of compaction**: This was genuinely hard. The algorithm is conceptually simple:
1. Read all live keys from old segments
2. Write them to new segments
3. Atomically swap old for new

But the details are subtle:
- What if you crash mid-compaction?
- How do you handle the active segment?
- When do you delete old files?

My solution: Collect all live data in memory, clear old segments, rewrite everything. 
Not the most sophisticated approach, but it works and is easy to reason about.

**Testing strategies**:
- Unit tests for individual components (`segment.rs`, `index.rs`)
- Integration tests for full workflows (`tests/store_integration.rs`)
- Example programs that demonstrate real usage
- Manual testing with the REPL
- Benchmarks with Criterion to catch performance regressions

**The examples that taught me most**:
- `examples/compaction.rs`: Writing 100 keys, 10 versions each, then compacting
- `examples/large_dataset.rs`: 10,000 keys to stress-test performance
- `examples/persistence.rs`: Three sessions proving data survives restarts

### Week 4+: HTTP API and Production Features (Nov 16 onwards)

**Adding Axum**: Wrapping the KVStore in an HTTP server taught me:
- Async Rust with Tokio
- Shared mutable state with `Arc<Mutex<T>>`
- REST API design patterns
- Error handling in web contexts

**The Volume abstraction**: Created `BlobStorage` as a higher-level API that:
- Tracks metadata (etags, sizes)
- Provides a cleaner interface for HTTP handlers
- Separates concerns between storage engine and API

## What Surprised Me

### 1. How naturally ownership maps to stateful systems

Rust's ownership model feels *designed* for storage engines. The borrow checker prevents:
- Concurrent mutations to the index
- Using segment references after files are closed
- Keeping pointers to data that might be compacted away

These aren't artificial restrictions—they're real bugs that would cause data corruption.

### 2. Compaction is genuinely complex

Even in the "simple" case, you're dealing with:
- Concurrent reads during compaction (I don't handle this yet)
- Ensuring atomicity
- Managing disk space (what if you run out mid-compaction?)
- Handling crashes at any point

I understand now why production databases have entire teams dedicated to compaction strategies.

### 3. How satisfying cargo clippy is

Having a tool that teaches you better patterns while you code is incredible. Some of my favorite clippy suggestions:
- Using `if let` instead of `match` for single patterns
- Replacing `iter().map().collect()` with more idiomatic patterns
- Catching common mistakes like `clone()` on Copy types

### 4. The power of simple on-disk formats

My format is incredibly basic, but it's:
- Easy to implement correctly
- Easy to debug (you can hex-dump files and understand them)
- Fast enough for the use case
- Extensible (I can add fields later)

Premature optimization would have been a mistake here.

## Mistakes and Lessons

### Mistake 1: Not handling errors properly from the start

**Problem**: Used `unwrap()` everywhere, making debugging painful.

**Solution**: Introduced custom error types with `thiserror`:
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Active segment not found")]
    ActiveSegmentNotFound,
    
    #[error("Segment {0} not found")]
    SegmentNotFound(usize),
    
    #[error(transparent)]
    Io(#[from] io::Error),
}

Then used `anyhow` in the CLI for simpler error handling at the boundary.

### Mistake 2: Not writing tests early enough

**Problem**: Built features without tests, then spent days fixing subtle bugs.

**Solution**: Adopted TDD-ish approach: write a failing test, make it pass, refactor. 
Not strict TDD, but tests before moving on.

### Mistake 3: Trying to optimize too early

**Problem**: Spent time on "performance" before proving correctness.

**Solution**: Make it work, make it right, make it fast—in that order. 
The performance I care about comes from architecture (append-only logs, in-memory index), not micro-optimizations.

### Mistake 4: Not reading enough existing code

**Problem**: Reinvented wheels poorly.

**Solution**: Started reading production storage engine code:
- [sled](https://github.com/spacejam/sled)
- [RocksDB source](https://github.com/facebook/rocksdb)
- [WiredTiger](https://source.wiredtiger.com/)

Even though they're more complex, seeing how experts structure their code taught me patterns I could apply.

## Resources That Actually Helped

### Books
- [The Rust Programming Language](https://doc.rust-lang.org/book/) - The foundation
- [Database Internals](https://www.databass.dev/) by Alex Petrov - Storage engine concepts
- [Designing Data-Intensive Applications](https://dataintensive.net/) by Martin Kleppmann - System design thinking

### Articles & Papers
- [Log-Structured Merge-Trees](http://www.benstopford.com/2015/02/14/log-structured-merge-trees/) by Ben Stopford
- [The Log: What every software engineer should know](https://engineering.linkedin.com/distributed-systems/log-what-every-software-engineer-should-know-about-real-time-datas-unifying) by Jay Kreps
- [Bitcask paper](https://riak.com/assets/bitcask-intro.pdf) - Inspired the segmented log approach

### Tools
- **rust-analyzer**: Real-time feedback in VSCode
- **cargo clippy**: Code quality mentor
- **cargo watch**: Auto-rebuild on save
- **cargo criterion**: Performance regression testing

### Communities
- r/rust - Welcoming, helpful, thoughtful
- Rust Discord - Real-time help
- This Week in Rust newsletter - Stay current

## What I'd Do Differently Next Time

1. **Start with comprehensive tests**: Writing tests after implementation is harder than writing them alongside.

2. **Design the error types upfront**: Retrofitting error handling is painful.

3. **Read more production code earlier**: I learned patterns in week 3 that would have saved time in week 1.

4. **Document as I go**: Writing docs after the fact means I forget the "why" behind decisions.

5. **Use feature flags for experiments**: I rewrote code several times when I could have feature-flagged new approaches.

6. **Focus on one thing at a time**: I tried to add HTTP API + benchmarks + examples simultaneously. Sequential would have been clearer.

## Questions Still Open

### Technical
- **Background compaction**: How do you compact while serving reads/writes?
- **WAL for durability**: Should I add a write-ahead log before segments?
- **Crash recovery**: What happens if you crash during compaction? I need atomic rename guarantees.
- **Bloom filters**: Would they significantly improve read performance?

### Design
- **API surface**: Is my API intuitive for users coming from other KV stores?
- **Configuration**: Should more things be configurable? Or is simplicity better?
- **Observability**: What metrics should I expose?

### Performance
- **Memory-mapped I/O**: Would mmap be faster than BufReader/BufWriter?
- **Compression**: Would compressing segments save significant space?
- **Index snapshots**: Should I persist the index to avoid rebuild on restart?

## For Anyone Learning Rust

### If you're coming from Python/JavaScript
- The type system will feel verbose at first. Trust that it catches real bugs.
- Ownership is learnable. Give yourself 2-3 weeks of confusion.
- The compiler is your pair programmer, not your enemy.

### If you're coming from C/C++
- You already understand memory. Rust makes those concepts explicit.
- `unsafe` exists when you need it, but you probably don't need it yet.
- RAII is everywhere and it's beautiful.

### If you're coming from a non-technical background
- You can do this. I come from literature and I built this.
- The question isn't whether you're "technical enough"—it's whether you're curious enough to sit with not knowing long enough to understand.
- Start before you're ready. The readiness comes from starting.

### General advice
1. **Build something you care about**: Todo apps are fine for syntax, but you'll learn more building something that interests you.

2. **Read compiler errors carefully**: They're usually right and often helpful.

3. **Use clippy aggressively**: It's like having a Rust expert review your code constantly.

4. **Don't skip the hard parts**: I wanted to skip compaction. I'm glad I didn't—it taught me the most.

5. **Share your work**: I posted on Reddit, LinkedIn, and GitHub. The feedback and encouragement kept me going.

## What's Next

- [ ] Background compaction (biggest missing piece)
- [ ] Index snapshots for faster restarts
- [ ] Bloom filters to reduce disk I/O
- [ ] Range queries (requires sorted segments)
- [ ] More sophisticated replication
- [ ] WAL for stronger durability guarantees
- [ ] Performance optimization based on benchmarks
- [ ] Production hardening (fuzzing, stress testing)

## Reflections

Three weeks ago, Rust errors made me panic. 
Today, I'm building storage systems in public.

The journey from "I don't understand ownership" to "I shipped a working storage engine" taught me that learning isn't linear. 
There were days where nothing made sense. There were breakthroughs where everything clicked.

What mattered was showing up every day, writing code, breaking things, fixing them, and trusting the process.

If you're learning Rust—or any new skill—and feeling overwhelmed: that's normal. 
That's the learning happening.

The readiness comes from starting.

---

**Built by [@whispem](https://github.com/whispem)**  
**Project**: [mini-kvstore-v2](https://github.com/whispem/mini-kvstore-v2)

*"Structure determines outcome. Precision isn't optional. You learn by building."*
