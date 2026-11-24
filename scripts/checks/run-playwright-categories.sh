#!/usr/bin/env bash
# Categories gate runner (self-contained): fresh server + spec + teardown.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate-cats"; mkdir -p "$ROOT/target/koma-gate-cats"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate-cats/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate-cats/storage"
export KOMA_TOKEN="t" KOMA_ADDR="127.0.0.1:18100"
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate-cats-server.log" 2>&1 &
sleep 2
cd "$ROOT/web" || exit 1
npx playwright test categories.spec.ts --project=chromium --reporter=json > "$ROOT/.dojo/proof/playwright20-rerun.log" 2>&1
code=$?
powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
exit "$code"
