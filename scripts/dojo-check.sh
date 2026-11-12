#!/usr/bin/env bash
set -e
set -o pipefail
mkdir -p .dojo/proof
{
  cargo build 2>&1
  cargo clippy -- -D warnings 2>&1
  cargo nextest run 2>&1
} | tee .dojo/proof/check-output.log
# Reached only if every check passed (set -e + pipefail):
sha() { sha256sum "$1" 2>/dev/null || shasum -a 256 "$1"; }
{
  echo "ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "exit=0"
  echo "output_sha256=$(sha .dojo/proof/check-output.log | awk '{print $1}')"
} > .dojo/proof/check-proof
