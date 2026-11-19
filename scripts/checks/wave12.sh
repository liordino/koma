#!/usr/bin/env bash
# Wave 12 check — library/category RPCs over gRPC-Web through the bearer middleware.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave12-check.log) 2>&1

expect "library RPC proto definitions exist" 'grep -qi "AddManga\|ListLibrary\|CreateCategory" proto/koma/v1/koma.proto'
expect "library service module exists" '[ -f server/src/library.rs ]'
expect "library RPCs registered in router" 'grep -qi "library" server/src/lib.rs'
expect "user identity flows via middleware" 'grep -qi "extensions\|user_id\|UserId" server/src/auth.rs'
expect "library integration tests exist" '[ -f server/tests/library_rpc.rs ]'
expect "library tests pass" 'cd server && cargo test --test library_rpc 2>&1 | grep -q "test result: ok"'

exit $fail
