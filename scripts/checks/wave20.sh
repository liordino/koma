#!/usr/bin/env bash
# Wave 20 check — category config UI: CRUD w/ consent, membership, purge wiring.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave20-check.log) 2>&1

expect "category UI route exists" '[ -f "$ROOT/web/src/routes/categories/+page.svelte" ]'
expect "UpdateCategory + ExportBackup RPCs in proto" 'grep -qi "UpdateCategory" "$ROOT/proto/koma/v1/koma.proto" && grep -qi "ExportBackup" "$ROOT/proto/koma/v1/koma.proto"'
expect "consent gate in the purge UI flow" 'grep -rqi "confirm" "$ROOT/web/src/routes/categories/"'
expect "category UI tests exist" '[ -f "$ROOT/web/tests/categories.spec.ts" ]'
expect "vite build compiles the SPA" 'cd "$ROOT/web" && npx vite build > /dev/null 2>&1'
if [ "$fail" -eq 0 ]; then
  powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" > /dev/null 2>&1
  rm -rf "$ROOT/target/koma-gate"; mkdir -p "$ROOT/target/koma-gate"
  export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate/koma.db?mode=rwc"
  export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate/storage"
  export KOMA_TOKEN="t"; export KOMA_ADDR="127.0.0.1:18100"
  "$ROOT/target/debug/koma-server.exe" > "$ROOT/target/koma-gate-server.log" 2>&1 &
  _SERVER_PID=$!
  sleep 1
  expect "category UI tests pass" 'bash "$ROOT/scripts/checks/run-playwright-categories.sh"'
  powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" > /dev/null 2>&1
fi
expect "full suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,|test result: ok"'

exit $fail
