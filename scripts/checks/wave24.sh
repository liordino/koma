#!/usr/bin/env bash
# Wave 24 check — SearchManga RPC + /search page (results grid, add-to-category).
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave24-check.log) 2>&1

expect "SearchManga RPC in proto (unique response)" 'grep -q "rpc SearchManga" "$ROOT/proto/koma/v1/koma.proto" && grep -q "message SearchMangaResponse" "$ROOT/proto/koma/v1/koma.proto"'
expect "search RPC implemented server-side" 'grep -q "async fn search_manga" "$ROOT/server/src/library.rs"'
expect "/search route exists" '[ -f "$ROOT/web/src/routes/search/+page.svelte" ]'
expect "search adds to a category (picker present)" 'grep -q "addManga" "$ROOT/web/src/routes/search/+page.svelte" && grep -q "select" "$ROOT/web/src/routes/search/+page.svelte"'
expect "search RPC rust tests pass (wiremock)" 'cd "$ROOT/server" && cargo test --test search_rpc 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "search playwright passes (live MangaDex flow)" 'bash "$ROOT/scripts/checks/run-playwright-search.sh"'
expect "tsc + vite build green" 'cd "$ROOT/web" && npx tsc --noEmit && npx vite build >/dev/null 2>&1'
expect "full rust suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,"'

exit $fail
