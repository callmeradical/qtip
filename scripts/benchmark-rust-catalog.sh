#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_PATH="${QTIP_CATALOG_BENCH_REPORT:-${ROOT_DIR}/crates/qtip-catalog/benchmarks/reports/us-008-baseline.md}"
FILES="${QTIP_CATALOG_BENCH_FILES:-1000}"
ITERATIONS="${QTIP_CATALOG_BENCH_ITERATIONS:-5}"
WARMUP="${QTIP_CATALOG_BENCH_WARMUP:-1}"
TARGET_MS="${QTIP_CATALOG_BENCH_TARGET_MS:-500}"
CONCURRENCY="${QTIP_CATALOG_BENCH_CONCURRENCY:-1,8,32,100}"
TEMPLATE_PATH="${QTIP_CATALOG_BENCH_TEMPLATE:-${ROOT_DIR}/crates/qtip-catalog/benchmarks/fixture/scenario-template.yaml}"

if [[ $# -gt 0 ]]; then
  REPORT_PATH="$1"
fi

os_info="$(uname -srm 2>/dev/null || uname -a)"

if command -v sysctl >/dev/null 2>&1; then
  cpu_info="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || true)"
  cores="$(sysctl -n hw.logicalcpu 2>/dev/null || true)"
  mem_bytes="$(sysctl -n hw.memsize 2>/dev/null || true)"
else
  cpu_info=""
  cores=""
  mem_bytes=""
fi

if [[ -z "${cpu_info}" ]] && command -v lscpu >/dev/null 2>&1; then
  cpu_info="$(lscpu | awk -F: '/Model name/ {gsub(/^ +/, "", $2); print $2; exit}')"
fi

if [[ -z "${cores}" ]]; then
  if command -v nproc >/dev/null 2>&1; then
    cores="$(nproc)"
  else
    cores="unknown"
  fi
fi

if [[ -n "${mem_bytes}" && "${mem_bytes}" =~ ^[0-9]+$ ]]; then
  memory_gib="$(awk -v bytes="${mem_bytes}" 'BEGIN {printf "%.1f GiB", bytes / 1024 / 1024 / 1024}')"
else
  memory_gib="unknown"
fi

if [[ -z "${cpu_info}" ]]; then
  cpu_info="unknown"
fi

machine_assumptions="OS ${os_info}; CPU ${cpu_info}; Cores ${cores}; Memory ${memory_gib}; Local SSD assumed"

mkdir -p "$(dirname "${REPORT_PATH}")"

set +e
cargo run \
  --manifest-path "${ROOT_DIR}/Cargo.toml" \
  -p qtip-catalog \
  --example benchmark_catalog \
  --release \
  -- \
  --files "${FILES}" \
  --iterations "${ITERATIONS}" \
  --warmup "${WARMUP}" \
  --target-ms "${TARGET_MS}" \
  --concurrency "${CONCURRENCY}" \
  --fixture-template "${TEMPLATE_PATH}" \
  --machine "${machine_assumptions}" \
  --report "${REPORT_PATH}"
status=$?
set -e

if [[ ${status} -eq 2 ]]; then
  echo "Benchmark target missed (regression flagged). Report: ${REPORT_PATH}" >&2
elif [[ ${status} -ne 0 ]]; then
  echo "Benchmark run failed. Report may be incomplete: ${REPORT_PATH}" >&2
  exit ${status}
else
  echo "Benchmark target met. Report: ${REPORT_PATH}"
fi

exit ${status}
