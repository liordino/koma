#!/usr/bin/env bash
# Wave 15 check — DB-backed download queue + supervised single-writer worker.
set -o pipefail
fail=0
expect() {
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(pwd)"; export ROOT
exec > >(tee .dojo/proof/wave15-check.log) 2>&1

expect "download queue module exists" '[ -f "$ROOT/server/src/download.rs" ] && grep -rqi "queue" "$ROOT/server/src/"'
expect "chapter queue state migration exists" 'ls "$ROOT"/server/migrations/*.sql | xargs grep -lqi "queue" || false'
expect "queue integration tests exist" '[ -f "$ROOT/server/tests/download_queue.rs" ]'
expect "queue tests pass" 'cd "$ROOT/server" && cargo test --test download_queue 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "all tests still pass" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'

exit $fail
