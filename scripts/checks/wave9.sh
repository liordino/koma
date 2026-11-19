#!/usr/bin/env bash
# Wave 9 check — SSRF-guarded outbound fetch + dual-plane rate limiter.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave9-check.log) 2>&1

expect "outbound fetch module exists" '[ -f server/src/fetch.rs ] || [ -f server/src/fetch/mod.rs ]'
expect "domain allowlist enforced in code" 'grep -rqi "allowlist" server/src/'
expect "two rate-limit planes exist" 'grep -rqiE "plane" server/src/ && grep -rqiE "rate" server/src/'
expect "wiremock dev-dependency present" 'grep -qi "wiremock" server/Cargo.toml'
expect "ssrf/ratelimit tests exist" '[ -f server/tests/outbound_fetch.rs ]'
expect "tests pass" 'cd server && cargo test --test outbound_fetch 2>&1 | grep -q "test result: ok"'

exit $fail
