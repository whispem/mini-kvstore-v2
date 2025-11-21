.PHONY: help build test bench clean run docker fmt clippy docs

# Default target
help:
	@echo "Mini KV Store v2 - Make targets:"
	@echo ""
	@echo "  make build       - Build release binary"
	@echo "  make test        - Run all tests"
	@echo "  make test-heavy  - Run tests including heavy/resource-intensive"
	@echo "  make bench       - Run benchmarks"
	@echo "  make benchmark   - Run automated k6 benchmark"
	@echo "  make run         - Run CLI in release mode"
	@echo "  make server      - Run HTTP server"
	@echo "  make docker      - Build Docker image"
	@echo "  make docker-up   - Start Docker Compose cluster"
	@echo "  make docker-down - Stop Docker Compose cluster"
	@echo "  make fmt         - Format code"
	@echo "  make clippy      - Run clippy lints"
	@echo "  make docs        - Generate and open documentation"
	@echo "  make clean       - Clean build artifacts"
	@echo ""

# Build
build:
	cargo build --release

# Testing
test:
	cargo test --all --release

test-heavy:
	cargo test --all --release --features heavy-tests

# Benchmarking
bench:
	cargo bench

benchmark:
	@echo "Running automated k6 benchmark..."
	@./run_benchmark.sh

# Run
run:
	cargo run --release

server:
	cargo run --release --bin volume-server

# Docker
docker:
	docker build -t mini-kvstore-v2:latest .

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

# Code quality
fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Documentation
docs:
	cargo doc --no-deps --open

# Cleanup
clean:
	cargo clean
	rm -rf data/ tests_data/ bench_data/ bench_temp_*
	rm -rf volume_data_* example_*_data/

# CI targets
ci: fmt clippy test

# Install dependencies (macOS)
install-deps-mac:
	brew install k6 jq

# Install dependencies (Linux)
install-deps-linux:
	@echo "Installing k6..."
	@sudo gpg -k
	@sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
	@echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
	@sudo apt-get update
	@sudo apt-get install k6
	@echo "Installing jq..."
	@sudo apt-get install -y jq

# Examples
example-basic:
	cargo run --example basic_usage

example-compaction:
	cargo run --example compaction

example-persistence:
	cargo run --example persistence

example-large:
	cargo run --example large_dataset

example-volume:
	cargo run --example volume_usage

# All examples
examples: example-basic example-compaction example-persistence example-large example-volume

# Quick check before commit
pre-commit: fmt clippy test
	@echo "✓ Pre-commit checks passed!"
