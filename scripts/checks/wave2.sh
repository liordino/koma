#!/usr/bin/env bash
# Wave 2 check — SQLite operational baseline. Fails until the outcome is real.
# Evidence contract: exit 0 only if every check passes; output is the record.
set -o pipefail
fail=0
expect() { # $1 label, $2 command
  if eval "$2" >/dev/null 2>&1; then
    echo "PASS: $1"
  else
    echo "FAIL: $1"
    fail=1
  fi
}

expect "cargo workspace exists with sqlite feature" 'grep -q "sqlx" Cargo.toml server/Cargo.toml 2>/dev/null && grep -q "sqlite" server/Cargo.toml'
expect "connection-init module exists" '[ -f server/src/db/mod.rs ] || [ -f server/src/db.rs ]'
expect "init sets WAL, synchronous NORMAL, FK ON, busy_timeout" 'grep -qi "journal_mode" server/src/db* -r && grep -qi "synchronous" server/src/db* -r && grep -qi "foreign_keys" server/src/db* -r && grep -qi "busy_timeout" server/src/db* -r'
expect "numbered migrations exist" 'ls server/migrations/*.sql 2>/dev/null | grep -q "0001"'
expect "schema declares foreign keys" 'grep -qi "REFERENCES" server/migrations/*.sql'
expect "integration test proves WAL + FK + orphan rejection" '[ -f server/tests/sqlite_baseline.rs ]'
expect "integration test passes" 'cd server && cargo test --test sqlite_baseline 2>&1 | grep -q "test result: ok"'

exit $fail
