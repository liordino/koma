#!/usr/bin/env bash
# Wave 5 check — buf + connect-es codegen contract: TS generated from the repo proto compiles.
set -o pipefail
fail=0
expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}

mkdir -p .dojo/proof
exec > >(tee .dojo/proof/wave5-check.log) 2>&1
  expect "buf module config exists at repo root" '[ -f buf.yaml ]'
  expect "buf image generation config exists" '[ -f buf.gen.yaml ]'
  expect "generated TS from repo proto exists" '[ -f web/src/gen/koma/v1/koma_pb.ts ]'
  expect "web workspace declares connect deps" 'grep -q "@connectrpc" web/package.json'
  expect "generated TS compiles (tsc --noEmit)" 'cd web && npx tsc --noEmit > ../.dojo/proof/wave5-tsc.log 2>&1'
exit $fail
