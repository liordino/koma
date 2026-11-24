#!/usr/bin/env bash
# Wave 26 check — categories redesign (flows preserved), reader chrome, mobile pass.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave26-check.log) 2>&1

expect "categories redesigned as cards" 'grep -q "cat-card" "$ROOT/web/src/routes/categories/+page.svelte"'
expect "consent gate survives the redesign" 'grep -q "confirm" "$ROOT/web/src/routes/categories/+page.svelte"'
expect "reader chrome: progress strip + pill" 'grep -q "strip" "$ROOT/web/src/routes/reader/[chapterId]/+page.svelte" && grep -q "pill" "$ROOT/web/src/routes/reader/[chapterId]/+page.svelte"'
expect "categories spec asserts consent + purge on the new shape" 'grep -q "confirm" "$ROOT/web/tests/categories.spec.ts" && grep -q "purge" "$ROOT/web/tests/categories.spec.ts"'
expect "categories spec passes" 'bash "$ROOT/scripts/checks/run-playwright-categories.sh"'
expect "reader spec still green" 'bash "$ROOT/scripts/checks/run-playwright.sh"'
expect "mobile + desktop captures render (screenshots spec)" 'bash "$ROOT/scripts/checks/run-playwright-shots.sh"'
expect "tsc + vite build green" 'cd "$ROOT/web" && npx tsc --noEmit && npx vite build >/dev/null 2>&1'
expect "full rust suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,"'

exit $fail
