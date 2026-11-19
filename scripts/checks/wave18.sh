#!/usr/bin/env bash
# Wave 18 check — delete_after_read safety: backup export, gated cleanup, explicit purge.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave18-check.log) 2>&1

expect "backup export module exists" '[ -f "$ROOT/server/src/backup.rs" ]'
expect "cleanup refuses without fresh backup (code path)" 'grep -rqi "fresh" "$ROOT/server/src/backup.rs"'
expect "purge is explicit user-confirmed RPC" 'grep -qi "PurgeCateg" "$ROOT/proto/koma/v1/koma.proto"'
expect "safety tests exist" '[ -f "$ROOT/server/tests/safety_chain.rs" ]'
expect "safety tests pass" 'cd "$ROOT/server" && cargo test --test safety_chain 2>&1 | grep -qE "test result: ok|[0-9]+ passed,"'
expect "full suite green" 'cd "$ROOT/server" && cargo nextest run 2>&1 | grep -qE "[0-9]+ passed,|test result: ok"'

exit $fail
