#!/usr/bin/env bash
# Wave 3 check — bearer-token auth + unary gRPC-Web transport. Fails until the outcome is real.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}

expect "axum and tonic-web are dependencies" 'grep -q "tonic" server/Cargo.toml && grep -q "axum" server/Cargo.toml'
expect "bearer middleware module exists" '[ -f server/src/auth.rs ] || [ -f server/src/auth/mod.rs ]'
expect "gRPC-Web route mounted" 'grep -rqi "grpc_web\|GrpcWeb" server/src/'
expect "CORS restricted to configured origins" 'grep -rqi "cors" server/src/'
expect "auth integration test exists" '[ -f server/tests/auth_transport.rs ]'
expect "auth integration test passes" 'cd server && cargo test --test auth_transport 2>&1 | grep -q "test result: ok"'

exit $fail
