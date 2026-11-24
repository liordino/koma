#!/usr/bin/env bash
# Reader gate runner (self-contained): fresh server + bench-seed + spec + teardown.
# The core-loop spec (wave 14) is a device benchmark, not part of local UI gates.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate-reader"; mkdir -p "$ROOT/target/koma-gate-reader"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate-reader/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate-reader/storage"
export KOMA_TOKEN="t" KOMA_ADDR="127.0.0.1:18100"
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate-reader-server.log" 2>&1 &
sleep 2
KOMA_DB_URL="$KOMA_DB_URL" KOMA_STORAGE="$KOMA_STORAGE" "$ROOT/target/debug/bench-seed.exe" >/dev/null
cd "$ROOT/web" || exit 1
npx playwright test reader.spec.ts --project=chromium --reporter=json > "$ROOT/.dojo/proof/playwright13-rerun.log" 2>&1
code=$?
powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
exit "$code"
