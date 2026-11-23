#!/usr/bin/env bash
# Wave 19 check — staged .tachibk import (real fixture; snapshot + idempotent upserts).
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet - fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave19-check.log) 2>&1

expect "import module exists" '[ -f "$ROOT/server/src/import_tachibk.rs" ]'
expect "mihon proto subset exists" '[ -f "$ROOT/proto/mihon/backup.proto" ]'
expect "import tests exist" '[ -f "$ROOT/server/tests/tachibk_import.rs" ]'
expect "real fixture committed" '[ -f "$ROOT/server/tests/fixtures/mihon-real.tachibk" ]'
expect "import tests pass (real fixture + idempotent re-import)" 'cd "$ROOT/server" && cargo test --test tachibk_import 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "full suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,|test result: ok"'

exit $fail
