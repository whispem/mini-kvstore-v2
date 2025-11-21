#!/usr/bin/env bash
# Comprehensive benchmark script for mini-kvstore-v2
# Builds volume server, launches it, and runs k6 benchmark

set -euo pipefail

# Default configuration
NUM_VOLUMES="${1:-1}"
COORD_PORT="${2:-8000}"
VOLUME_START_PORT="${3:-9000}"
VUS="${4:-16}"
DURATION="${5:-30s}"
OBJECT_SIZE="${6:-1048576}" # 1 MB default

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  mini-kvstore-v2 Benchmark${NC}"
    echo -e "${BLUE}================================${NC}"
}

print_help() {
    cat << EOF
Usage: $0 [NUM_VOLUMES] [COORD_PORT] [VOLUME_START_PORT] [VUS] [DURATION] [OBJECT_SIZE]

Arguments:
  NUM_VOLUMES         Number of volume servers (default: 1)
  COORD_PORT          Main server port (default: 8000)
  VOLUME_START_PORT   Starting port for volumes (default: 9000)
  VUS                 Number of virtual users for k6 (default: 16)
  DURATION            Test duration (default: 30s)
  OBJECT_SIZE         Object size in bytes (default: 1048576 = 1 MB)

Examples:
  $0                    # Use all defaults
  $0 3                  # 3 volumes, other defaults
  $0 3 8000 9000 32 60s 2097152  # Full config

Requirements:
  - Rust toolchain (cargo)
  - k6 (brew install k6 or https://k6.io/docs/get-started/installation/)
  - jq (brew install jq)
EOF
}

if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
    print_help
    exit 0
fi

# Check dependencies
check_deps() {
    local missing=()
    
    if ! command -v cargo &> /dev/null; then
        missing+=("cargo (Rust toolchain)")
    fi
    
    if ! command -v k6 &> /dev/null; then
        missing+=("k6 (brew install k6)")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing+=("jq (brew install jq)")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        echo -e "${RED}Missing dependencies:${NC}"
        for dep in "${missing[@]}"; do
            echo -e "  - ${dep}"
        done
        exit 1
    fi
}

print_banner
echo ""
echo -e "${GREEN}Configuration:${NC}"
echo "  Volumes: ${NUM_VOLUMES}"
echo "  Main port: ${COORD_PORT}"
echo "  Volume ports: ${VOLUME_START_PORT}+"
echo "  Virtual users: ${VUS}"
echo "  Duration: ${DURATION}"
echo "  Object size: $((OBJECT_SIZE / 1024 / 1024)) MB"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"
check_deps
echo -e "${GREEN}✓ All dependencies found${NC}"
echo ""

# Build
echo -e "${YELLOW}Building release binary...${NC}"
cargo build --release --quiet
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Create temp directories
BENCH_DIR="./bench_temp_$$"
mkdir -p "${BENCH_DIR}"
trap "rm -rf ${BENCH_DIR}" EXIT

# Start volume server
echo -e "${YELLOW}Starting volume server on port ${COORD_PORT}...${NC}"
VOLUME_DATA="${BENCH_DIR}/volume_data"
mkdir -p "${VOLUME_DATA}"

PORT="${COORD_PORT}" ./target/release/mini-kvstore-v2 --volume "${VOLUME_DATA}" --id "bench-vol" > "${BENCH_DIR}/server.log" 2>&1 &
SERVER_PID=$!

# Wait for server to be ready
echo -n "Waiting for server to be ready"
for i in {1..30}; do
    if curl -s "http://127.0.0.1:${COORD_PORT}/health" > /dev/null 2>&1; then
        echo ""
        echo -e "${GREEN}✓ Server ready${NC}"
        break
    fi
    echo -n "."
    sleep 1
    if [ $i -eq 30 ]; then
        echo ""
        echo -e "${RED}✗ Server failed to start${NC}"
        cat "${BENCH_DIR}/server.log"
        kill ${SERVER_PID} 2>/dev/null || true
        exit 1
    fi
done
echo ""

# Create k6 test script
cat > "${BENCH_DIR}/test.js" << 'EOFK6'
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const putRate = new Rate('put_success');
const getRate = new Rate('get_success');
const putLatency = new Trend('put_latency');
const getLatency = new Trend('get_latency');

const BASE_URL = __ENV.BASE_URL || 'http://127.0.0.1:8000';
const OBJECT_SIZE = parseInt(__ENV.OBJECT_SIZE || '1048576');

export let options = {
    vus: parseInt(__ENV.VUS || '16'),
    duration: __ENV.DURATION || '30s',
    thresholds: {
        'put_success': ['rate>0.95'],
        'get_success': ['rate>0.99'],
    },
};

// Generate random data of specified size
function generateData(size) {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < size; i++) {
        result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
}

export default function () {
    const key = `benchmark-key-${__VU}-${__ITER}`;
    const data = generateData(OBJECT_SIZE);
    
    // PUT operation
    const putStart = Date.now();
    const putRes = http.post(`${BASE_URL}/blobs/${key}`, data, {
        headers: { 'Content-Type': 'application/octet-stream' },
    });
    const putDuration = Date.now() - putStart;
    
    putRate.add(putRes.status === 201);
    putLatency.add(putDuration);
    
    check(putRes, {
        'PUT status is 201': (r) => r.status === 201,
    });
    
    // GET operation
    const getStart = Date.now();
    const getRes = http.get(`${BASE_URL}/blobs/${key}`);
    const getDuration = Date.now() - getStart;
    
    getRate.add(getRes.status === 200);
    getLatency.add(getDuration);
    
    check(getRes, {
        'GET status is 200': (r) => r.status === 200,
        'GET body correct': (r) => r.body.length === OBJECT_SIZE,
    });
    
    sleep(0.1);
}
EOFK6

# Run k6 benchmark
echo -e "${YELLOW}Running k6 benchmark...${NC}"
echo ""

k6 run \
    --out json="${BENCH_DIR}/results.json" \
    --summary-export="${BENCH_DIR}/summary.json" \
    --env BASE_URL="http://127.0.0.1:${COORD_PORT}" \
    --env VUS="${VUS}" \
    --env DURATION="${DURATION}" \
    --env OBJECT_SIZE="${OBJECT_SIZE}" \
    "${BENCH_DIR}/test.js"

# Parse results
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Benchmark Results${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Extract metrics using jq
if [ -f "${BENCH_DIR}/summary.json" ]; then
    PUT_P50=$(jq -r '.metrics.put_latency.values.p50' "${BENCH_DIR}/summary.json" 2>/dev/null || echo "N/A")
    PUT_P90=$(jq -r '.metrics.put_latency.values.p90' "${BENCH_DIR}/summary.json" 2>/dev/null || echo "N/A")
    PUT_P95=$(jq -r '.metrics.put_latency.values.p95' "${BENCH_DIR}/summary.json" 2>/dev/null || echo "N/A")
    
    GET_P50=$(jq -r '.metrics.get_latency.values.p50' "${BENCH_DIR}/summary.json" 2>/dev/null || echo "N/A")
    GET_P90=$(jq -r '.metrics.get_latency.values.p90' "${BENCH_DIR}/summary.json" 2>/dev/null || echo "N/A")
    GET_P95=$(jq -r '.metrics.get_latency.values.p95' "${BENCH_DIR}/summary.json" 2>/dev/null || echo "N/A")
    
    echo "Host: $(uname -m) · $(sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 " GB"}' || echo 'N/A') · $(uname -s) $(sw_vers -productVersion 2>/dev/null || uname -r)"
    echo "Configuration: ${NUM_VOLUMES} volume(s), size=$((OBJECT_SIZE / 1024 / 1024)) MiB, VUs=${VUS}, Duration=${DURATION}"
    echo ""
    echo "PUT Latency:"
    echo "  p50: ${PUT_P50} ms"
    echo "  p90: ${PUT_P90} ms"
    echo "  p95: ${PUT_P95} ms"
    echo ""
    echo "GET Latency:"
    echo "  p50: ${GET_P50} ms"
    echo "  p90: ${GET_P90} ms"
    echo "  p95: ${GET_P95} ms"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo ""

# Cleanup
echo -e "${YELLOW}Cleaning up...${NC}"
kill ${SERVER_PID} 2>/dev/null || true
wait ${SERVER_PID} 2>/dev/null || true
echo -e "${GREEN}✓ Benchmark complete${NC}"
