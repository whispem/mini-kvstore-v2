#!/bin/bash

set -e

echo "Running cargo fmt..."
cargo fmt

echo "Adding allow attributes to silence dead code/unimplemented modules..."
sed -i '' '1s/^/#![allow(dead_code)]\n/' src/store/config.rs
sed -i '' '1s/^/#![allow(dead_code)]\n/' src/store/segment.rs
sed -i '' '1s/^/#![allow(unused_imports)]\n/' src/store/segment.rs

echo "Running cargo clippy..."
cargo clippy --all-targets --all-features -- -D warnings || true

echo "Done!"

