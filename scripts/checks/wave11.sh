#!/usr/bin/env bash
# Wave 11 check — At-Home image flow + chapter download worker (single-writer).
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave11-check.log) 2>&1

expect "at-home flow in adapter" 'grep -qi "at-home\|at_home" server/src/source/mangadex.rs'
expect "download worker module exists" '[ -f server/src/download.rs ]'
expect "download writes page files + DB row update" 'grep -qi "storage_dir" server/src/download.rs && grep -qi "downloaded" server/src/download.rs'
expect "report-key POST implemented" 'grep -qi "report" server/src/source/mangadex.rs'
expect "integration tests exist" '[ -f server/tests/at_home_download.rs ]'
expect "tests pass" 'cd server && cargo test --test at_home_download 2>&1 | grep -q "test result: ok"'

exit $fail
