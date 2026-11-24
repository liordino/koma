#!/usr/bin/env bash
# Wave-24 search gate runner: fresh server + live-source search spec.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate24"; mkdir -p "$ROOT/target/koma-gate24"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate24/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate24/storage"
export KOMA_TOKEN="t" KOMA_ADDR="127.0.0.1:18100"
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate24-server.log" 2>&1 &
sleep 1

cd "$ROOT/web" || exit 1
npx playwright test search.spec.ts --project=chromium --reporter=json > "$ROOT/.dojo/proof/playwright24-rerun.log" 2>&1
code=$?

powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
exit "$code"
