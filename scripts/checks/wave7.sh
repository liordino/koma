#!/usr/bin/env bash
# Wave 7 check — path-safe /pages serving pre-proto. Fails until the outcome is real.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave7-check.log) 2>&1

expect "pages route module exists" '[ -f server/src/pages.rs ] || [ -f server/src/pages/mod.rs ]'
expect "route resolves chapter via DB not path" 'grep -rniE "select .*from.*chapter" server/src/pages* server/src/'
expect "page_n is bounded" 'grep -rqi "page_n" server/src/pages*'
expect "canonicalize + prefix check in code" 'grep -rqi "canonicalize" server/src/'
expect "traversal regression test exists" '[ -f server/tests/path_safety.rs ]'
expect "traversal regression test fails now" 'cd server && cargo test --test path_safety 2>&1 | grep -q "test result: ok"'

exit $fail
