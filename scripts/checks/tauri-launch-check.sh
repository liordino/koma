#!/usr/bin/env bash
# Launch the built shell headless against a live server+SPA; the process must stay
# up (window + webview created, no init crash), then clean up.
set -o pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

powershell -NoProfile -Command "Get-Process koma-desktop -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
rm -rf "$ROOT/target/koma-gate22"; mkdir -p "$ROOT/target/koma-gate22"
export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate22/koma.db?mode=rwc"
export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate22/storage"
export KOMA_TOKEN=t KOMA_ADDR=127.0.0.1:18100
"$ROOT/target/debug/koma-server.exe" > "$ROOT/target/gate22-server.log" 2>&1 &
SERVER_PID=$!
(cd "$ROOT/web" && npx vite preview --port 4173 --strictPort > "$ROOT/target/gate22-preview.log" 2>&1 &)
sleep 3

KOMA_WEB_ORIGIN=http://localhost:4173 KOMA_HEADLESS=1 \
  "$ROOT/target/debug/koma-desktop.exe" > "$ROOT/target/gate22-shell.log" 2>&1 &
SHELL_PID=$!
sleep 6
ALIVE=0; kill -0 $SHELL_PID 2>/dev/null && ALIVE=1
kill $SHELL_PID 2>/dev/null; kill $SERVER_PID 2>/dev/null
powershell -NoProfile -Command "Get-Process koma-desktop,koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" >/dev/null 2>&1
[ "$ALIVE" = "1" ] && ! grep -qi "error\|panic" "$ROOT/target/gate22-shell.log"
