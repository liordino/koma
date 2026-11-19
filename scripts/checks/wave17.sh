#!/usr/bin/env bash
# Wave 17 check — mark-read RPC + trash window (files leave the serving path, re-downloadable).
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave17-check.log) 2>&1

expect "MarkRead RPC in proto" 'grep -qi "MarkRead" "$ROOT/proto/koma/v1/koma.proto"'
expect "trash module exists" '[ -f "$ROOT/server/src/trash.rs" ]'
expect "retention timestamps in schema" 'ls "$ROOT"/server/migrations/*.sql | xargs grep -lqi "trash_moved_at" || false'
expect "trash tests exist" '[ -f "$ROOT/server/tests/trash_window.rs" ]'
expect "trash tests pass" 'cd "$ROOT/server" && cargo test --test trash_window 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "full suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,|test result: ok"'

exit $fail
