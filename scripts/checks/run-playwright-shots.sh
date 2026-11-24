#!/usr/bin/env bash
# Screenshot/regression smoke: screenshots.spec captures desktop + mobile viewports.
# Writes to a throwaway dir — the committed README assets are regenerated deliberately.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate26"; mkdir -p "$ROOT/target/koma-gate26"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate26/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate26/storage"
export KOMA_TOKEN="shots" KOMA_ADDR="127.0.0.1:18080"
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate26-server.log" 2>&1 &
sleep 1
KOMA_DB_URL="$KOMA_DB_URL" "$ROOT/target/debug/koma-import.exe" "$ROOT/server/tests/fixtures/mihon-real.tachibk" > /dev/null
KOMA_DB_URL="$KOMA_DB_URL" KOMA_STORAGE="$KOMA_STORAGE" "$ROOT/target/debug/bench-seed.exe" > /dev/null
python3 - <<'PYEOF'
import sqlite3, os
c = sqlite3.connect(os.environ['KOMA_DB_URL'].replace('sqlite://','').replace('?mode=rwc',''))
c.execute("UPDATE chapter SET storage_dir='pages', downloaded=1, state='downloaded', pages=10 WHERE id=(SELECT MAX(id) FROM chapter WHERE manga_id=1)")
c.commit()
PYEOF
cd "$ROOT/web" || exit 1
KOMA_SHOTS_DIR="$ROOT/target/shots-gate" npx playwright test screenshots.spec.ts --project=chromium --reporter=json > "$ROOT/.dojo/proof/playwright26-shots.log" 2>&1
code=$?
powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
exit "$code"
