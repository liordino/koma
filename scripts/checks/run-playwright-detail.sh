#!/usr/bin/env bash
# Wave-25 detail gate runner: import fixture + wire a downloaded chapter + server + spec.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate25"; mkdir -p "$ROOT/target/koma-gate25"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate25/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate25/storage"
export KOMA_TOKEN="t" KOMA_ADDR="127.0.0.1:18100"
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate25-server.log" 2>&1 &
sleep 1
KOMA_DB_URL="$KOMA_DB_URL" "$ROOT/target/debug/koma-import.exe" "$ROOT/server/tests/fixtures/mihon-real.tachibk" > /dev/null
KOMA_DB_URL="$KOMA_DB_URL" KOMA_STORAGE="$KOMA_STORAGE" "$ROOT/target/debug/bench-seed.exe" > /dev/null
python3 - <<'PYEOF'
import sqlite3, os
c = sqlite3.connect(os.environ['KOMA_DB_URL'].replace('sqlite://','').replace('?mode=rwc',''))
# wire the bench pages to One Piece's NEWEST chapter row (the list shows the latest 200)
c.execute("UPDATE chapter SET storage_dir='pages', downloaded=1, state='downloaded', pages=10 WHERE id=(SELECT MAX(id) FROM chapter WHERE manga_id=1)")
c.commit()
print('gate25 seed: chapter 1 downloaded with 10 pages')
PYEOF

cd "$ROOT/web" || exit 1
npx playwright test detail.spec.ts --project=chromium --reporter=json > "$ROOT/.dojo/proof/playwright25-rerun.log" 2>&1
code=$?

powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
exit "$code"
