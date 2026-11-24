#!/usr/bin/env bash
# Wave 25 check — detail page, chapter RPCs, self-driving download worker.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave25-check.log) 2>&1

expect "ListChapters + DownloadChapter RPCs in proto" 'grep -q "rpc ListChapters" "$ROOT/proto/koma/v1/koma.proto" && grep -q "rpc DownloadChapter" "$ROOT/proto/koma/v1/koma.proto"'
expect "self-driving worker in main (spawned loop)" 'grep -q "run_queue_once" "$ROOT/server/src/main.rs"'
expect "worker records page counts" 'grep -q "pages" "$ROOT/server/src/download.rs" | head -2'
expect "detail route exists" '[ -f "$ROOT/web/src/routes/manga/[id]/+page.svelte" ]'
expect "detail rust tests pass (download -> downloaded + pages)" 'cd "$ROOT/server" && cargo test --test detail_rpc 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "detail playwright passes (seeded download + read link)" 'bash "$ROOT/scripts/checks/run-playwright-detail.sh"'
expect "tsc + vite build green" 'cd "$ROOT/web" && npx tsc --noEmit && npx vite build >/dev/null 2>&1'
expect "full rust suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,"'

exit $fail
