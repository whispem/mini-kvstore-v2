#!/usr/bin/env bash

echo "=== AUTO-FIX MINI-KVSTORE-V2 ==="

cargo fmt --all

cargo clippy --fix --allow-dirty --allow-staged || true

sed -i '' "s/use crate::store::error::{Result, StoreError};/use crate::store::error::Result;/" src/store/segment.rs

sed -i '' "s/let mut storage/let storage/" src/volume/handlers.rs

sed -i '' "1s/^/#![allow(dead_code)]\n/" src/store/segment.rs
sed -i '' "s/pub enum FsyncPolicy/#[allow(dead_code)]\npub enum FsyncPolicy/" src/store/config.rs

perl -i -pe 's/std::io::Error::new\(std::io::ErrorKind::Other, ([^)]+)\)/std::io::Error::other(\1)/g' src/store/engine.rs

cargo check

echo "=== DONE ==="

