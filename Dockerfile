# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests
COPY examples ./examples

# Build for release
RUN cargo build --release --bin volume-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/volume-server /usr/local/bin/volume-server

# Create data directory
RUN mkdir -p /data

# Expose the default port
EXPOSE 8000

# Set default environment variables
ENV PORT=8000
ENV VOLUME_ID=vol-1
ENV DATA_DIR=/data

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:${PORT}/health || exit 1

# Run the server
CMD ["volume-server"]
