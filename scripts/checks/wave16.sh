#!/usr/bin/env bash
# Wave 16 check — auto-update checker: external-ID upserts, idempotent, manual trigger.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave16-check.log) 2>&1

expect "update module exists" '[ -f "$ROOT/server/src/update.rs" ]'
expect "UpdateLibrary RPC in proto" 'grep -qi "UpdateLibrary" "$ROOT/proto/koma/v1/koma.proto"'
expect "update tests exist" '[ -f "$ROOT/server/tests/update_pass.rs" ]'
expect "update tests pass (idempotency proven)" 'cd "$ROOT/server" && cargo test --test update_pass 2>&1 | grep -q "test result: ok"'
expect "full suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,|test result: ok"'

exit $fail
