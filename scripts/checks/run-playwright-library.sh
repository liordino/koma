#!/usr/bin/env bash
# Wave-23 library gate runner: seed (real fixture import) + server + Playwright library.spec.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate23"; mkdir -p "$ROOT/target/koma-gate23"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate23/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate23/storage"
export KOMA_TOKEN="t" KOMA_ADDR="127.0.0.1:18100"
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate23-server.log" 2>&1 &
sleep 1
KOMA_DB_URL="$KOMA_DB_URL" "$ROOT/target/debug/koma-import.exe" "$ROOT/server/tests/fixtures/mihon-real.tachibk" > /dev/null

cd "$ROOT/web" || exit 1
npx playwright test library.spec.ts --project=chromium --reporter=json > "$ROOT/.dojo/proof/playwright23-rerun.log" 2>&1
code=$?

powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
exit "$code"
