#!/usr/bin/env bash
# Wave 13 check — web UI scaffold + windowed vertical reader with resumable progress.
# Sequential by design: cargo/tool invocations never overlap (they race the target dir).
set -o pipefail
# Anchor everything absolutely: expect() evals mutate THIS shell's cwd (the 'cd web' of one
# check used to persist and silently break every later relative path — the gate bug).
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
export ROOT
fail=0
expect() { # $1 label, $2 quoted shell snippet (may use $ROOT) — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then
    echo "PASS: $1"
  else
    echo "FAIL: $1"
    fail=1
  fi
}
cd "$ROOT" || exit 1
exec > >(tee .dojo/proof/wave13-check.log) 2>&1

expect "sveltekit app scaffolded in web/" '[ -f "$ROOT/web/src/routes/+page.svelte" ] || [ -f "$ROOT/web/src/app.html" ]'
expect "windowed reader exists (fixed placeholders, ±2 window, unmount far pages)" 'grep -rqi "placeholder" "$ROOT/web/src/routes/" && grep -rqi "WINDOW" "$ROOT/web/src/routes/" && grep -rqi "blobs\[i\]" "$ROOT/web/src/routes/"'
expect "page-anchored progress via RPC" 'grep -rqi "UpsertProgress" "$ROOT/web/src/" && grep -rqi "GetProgress" "$ROOT/web/src/"'
expect "fetch-to-blob page loading" 'grep -rqi "createObjectURL" "$ROOT/web/src/lib/" "$ROOT/web/src/routes/"'
expect "playwright config + reader test" '[ -f "$ROOT/web/playwright.config.ts" ] && [ -f "$ROOT/web/tests/reader.spec.ts" ]'
expect "vite build compiles the SPA" 'cd "$ROOT/web" && npx vite build > /dev/null 2>&1'

# The reader spec needs a live seeded server. Start the gate server (native, bench-seed
# chapter 1 with 10 pages), run the spec, then stop the server.
if [ "$fail" -eq 0 ]; then
  powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" > /dev/null 2>&1
  rm -rf "$ROOT/target/koma-gate"
  mkdir -p "$ROOT/target/koma-gate"
  export KOMA_DB_URL="sqlite://C:/Users/liord/src/koma/target/koma-gate/koma.db?mode=rwc"
  export KOMA_STORAGE="C:/Users/liord/src/koma/target/koma-gate/storage"
  export KOMA_TOKEN="t"
  export KOMA_ADDR="127.0.0.1:18100"
  ./target/debug/bench-seed.exe > /dev/null 2>&1
  ./target/debug/koma-server.exe > "$ROOT/target/koma-gate-server.log" 2>&1 &
  _SERVER_PID=$!
  sleep 1
  expect "playwright reader test passes" 'bash "$ROOT/scripts/checks/run-playwright.sh"'
  powershell -NoProfile -Command "Get-Process koma-server -ErrorAction SilentlyContinue | Stop-Process -Force" > /dev/null 2>&1
fi

exit $fail
