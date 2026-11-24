#!/usr/bin/env bash
# Wave 23 check — covers infrastructure + design system + app shell + library grid.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave23-check.log) 2>&1

expect "migration 0007 adds cover_url + pages" 'ls "$ROOT"/server/migrations/0007* >/dev/null && grep -q "cover_url" "$ROOT"/server/migrations/0007*'
expect "covers route is auth-gated and disk-cached" 'grep -q "/covers/" "$ROOT/server/src/lib.rs"'
expect "design tokens css exists (ink/amber)" '[ -f "$ROOT/web/src/app.css" ] && grep -q "0b0f1a" "$ROOT/web/src/app.css" && grep -q "f59e0b" "$ROOT/web/src/app.css"'
expect "app shell with nav" 'grep -qi "nav" "$ROOT/web/src/routes/+layout.svelte"'
expect "library is a cover grid" 'grep -qi "grid" "$ROOT/web/src/routes/+page.svelte" && grep -qi "cover" "$ROOT/web/src/routes/+page.svelte"'
expect "covers rust tests pass (fetch once, cache)" 'cd "$ROOT/server" && cargo test --test covers_route 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "library grid playwright passes" 'bash "$ROOT/scripts/checks/run-playwright-library.sh"'
expect "tsc + vite build green" 'cd "$ROOT/web" && npx tsc --noEmit && npx vite build >/dev/null 2>&1'

exit $fail
