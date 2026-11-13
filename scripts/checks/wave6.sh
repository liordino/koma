#!/usr/bin/env bash
# Wave 6 check — CI config and dependency hygiene. Fails until the outcome is real.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave6-check.log) 2>&1

expect "CI workflow exists" '[ -f .github/workflows/ci.yml ]'
expect "CI runs tests (nextest)" 'grep -qi "nextest" .github/workflows/ci.yml'
expect "CI runs clippy with -D warnings" 'grep -q "clippy" .github/workflows/ci.yml && grep -q "\-D warnings" .github/workflows/ci.yml'
expect "CI runs cargo-deny" 'grep -qi "cargo-deny\|EmbarkStudios/cargo-deny" .github/workflows/ci.yml'
expect "CI runs cargo-audit" 'grep -qi "cargo-audit\|rustsec/audit-check" .github/workflows/ci.yml'
expect "CI runs gitleaks" 'grep -qi "gitleaks" .github/workflows/ci.yml'
expect "sqlx prepare check gated on prepared data" 'grep -q "\.sqlx" .github/workflows/ci.yml'
expect "cargo-deny config exists" '[ -f deny.toml ]'
expect "Cargo.lock pinned and committed" 'git ls-files --error-unmatch Cargo.lock >/dev/null'
expect "workspace still green (dojo-check)" '$(dirname "$0")/../dojo-check.sh > /dev/null'

exit $fail
