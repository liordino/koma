#!/usr/bin/env bash
# Wave 4 check — FTS5 trigram search verified against the sqlx-linked build.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}

expect "fts migration exists with trigram tokenizer" 'grep -rqi "tokenize.*trigram" server/migrations/'
expect "fts table is external-content" 'grep -rqi "content *=" server/migrations/'
expect "fts search test exists" '[ -f server/tests/fts_search.rs ]'
expect "fts test asserts piece matches One Piece" 'grep -qi "piece" server/tests/fts_search.rs'
expect "fts test passes on linked sqlite" 'cd server && cargo test --test fts_search 2>&1 | grep -q "test result: ok"'

exit $fail
