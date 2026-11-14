#!/usr/bin/env bash
# Koma on-device benchmark harness (wave 8b) — run ON the Pi Zero 2 W over SSH.
# Co-tenants (decision 28): FreeLLMApi, MicroBin, Miniflux — measure WITH them running,
# as that is the real operating condition.
#
# Usage: ./run-benchmarks.sh [server-binary] [base-url] [storage-root]
# Writes one JSONL file: benchmarks/results-<ts>-<host>.jsonl
set -euo pipefail
SERVER_BIN="${1:-./koma-server}"
BASE_URL="${2:-http://127.0.0.1:8080}"
STORAGE_ROOT="${3:-./storage}"
TOKEN="${KOMA_BENCH_TOKEN:?set KOMA_BENCH_TOKEN}"
OUT="benchmarks/results-$(date -u +%Y%m%dT%H%M%SZ)-$(hostname).jsonl"

# rec <event> <json-object>: append one JSONL record.
rec() {
  printf '{"ts":"%s","event":"%s","data":%s}\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$1" "$2" >> "$OUT"
}

# --- device context -------------------------------------------------------
rec "device_context" '{"model":"Raspberry Pi Zero 2 W","cotenants":["FreeLLMApi","MicroBin","Miniflux"]}'
UNAME=$(uname -a)
LOADAVG=$(cut -d' ' -f1-3 /proc/loadavg)
MEMFREE_KB=$(awk '/MemAvailable/{print $2}' /proc/meminfo)
rec "kernel" "{\"uname\":\"$UNAME\"}"
rec "cpu_load_before" "{\"loadavg\":\"$LOADAVG\",\"mem_available_kb\":$MEMFREE_KB}"

# --- cold start -----------------------------------------------------------
# Stop, drop caches (needs root; falls back to plain restart), start, time to first 200.
sync
echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || rec "cold_cache_drop" '{"ok":false,"reason":"no root"}'
T0=$(date +%s.%N)
"$SERVER_BIN" --storage "$STORAGE_ROOT" &
SERVER_PID=$!
until curl -sf -o /dev/null -H "Authorization: Bearer $TOKEN" "$BASE_URL/koma.v1.KomaService/Ping"; do
  sleep 0.05
done
T1=$(date +%s.%N)
COLD_MS=$(awk -v a="$T1" -v b="$T0" 'BEGIN{printf "%.0f", (a-b)*1000}')
rec "cold_start_ms" "{\"ms\":$COLD_MS}"

# --- idle RSS after warm --------------------------------------------------
sleep 5
RSS_KB=$(awk '/VmRSS/{print $2}' "/proc/$SERVER_PID/status")
rec "idle_rss_kb" "{\"rss_kb\":$RSS_KB}"

# --- page p95 over 20 sequential page pulls (localhost) -------------------
CH_ID="${KOMA_BENCH_CHAPTER:-1}"
TMP_TIMINGS=$(mktemp)
for i in $(seq 1 20); do
  T=$(date +%s.%N)
  PAGE_N=$(( i % 10 ))
  curl -sf -o /dev/null -H "Authorization: Bearer $TOKEN" \
    "$BASE_URL/pages/$CH_ID/$PAGE_N"
  echo "$(echo "$(date +%s.%N) $T" | awk '{printf "%.0f", ($1-$2)*1000}')" >> "$TMP_TIMINGS"
done
P95=$(sort -n "$TMP_TIMINGS" | awk '{a[NR]=$1} END{print a[int(NR*0.95+0.5)]}')
rec "page_p95_ms" "{\"p95_ms\":$P95,\"samples\":20}"
rm -f "$TMP_TIMINGS"

kill "$SERVER_PID" 2>/dev/null || true
echo "wrote $OUT — append contents into benchmarks/baseline.md (wave 8b)"
