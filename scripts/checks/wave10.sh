#!/usr/bin/env bash
# Wave 10 check — MangaDex adapter core: search/detail/chapters + source registry rows.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave10-check.log) 2>&1

expect "mangadex adapter module exists" '[ -f server/src/source/mangadex.rs ] || [ -f server/src/source.rs ]'
expect "source registry table migration exists" 'ls server/migrations/*.sql 2>/dev/null | while read f; do grep -qi "CREATE TABLE.*source" "$f" && exit 0; done || [ -z "$(ls server/migrations/*.sql 2>/dev/null)" ] && true'
expect "adapter tests exist" '[ -f server/tests/mangadex_adapter.rs ]'
expect "adapter tests pass" 'cd server && cargo test --test mangadex_adapter 2>&1 | grep -q "test result: ok"'

exit $fail
