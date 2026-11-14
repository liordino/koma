#!/usr/bin/env bash
# Wave 8a check — ARM build spike + on-device benchmark harness committed.
set -o pipefail
fail=0

# Cross-compile env: cc-rs emits a rust-flavored --target= that zig cannot parse, filtered by
# the ccz wrapper; zig cc and zig ar provide the C toolchain. (Windows-discovered, wave 8a.)
ZIG_HOME="${ZIG_HOME:-/c/tools/zig/zig-x86_64-windows-0.14.1}"
export CC_aarch64_unknown_linux_musl="C:/tools/zig/ccz.cmd"
export CC_armv7_unknown_linux_musleabihf="C:/tools/zig/ccz.cmd"
export AR_aarch64_unknown_linux_musl="$ZIG_HOME/zig.exe ar"
export AR_armv7_unknown_linux_musleabihf="$ZIG_HOME/zig.exe ar"

expect() {
  if eval "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
exec > >(tee .dojo/proof/wave8a-check.log) 2>&1

export CCZ_TARGET=aarch64-linux-musl
expect "aarch64-unknown-linux-musl cargo check passes (no link needed)" \
  'rustup target list --installed | grep -q "aarch64-unknown-linux-musl" && cargo check --target aarch64-unknown-linux-musl --workspace > /dev/null'
expect "armv7musl fallback target installed" 'rustup target list --installed | grep -q "armv7-unknown-linux-musleabihf"'
export CCZ_TARGET=arm-linux-musleabihf
expect "armv7musl cargo check passes" 'cargo check --target armv7-unknown-linux-musleabihf --workspace > /dev/null'
export CCZ_TARGET=aarch64-linux-musl
expect "sqlite-linked build supports musl targets (bundled feature)" 'grep -q "bundled" server/Cargo.toml server/Cargo.toml.orig 2>/dev/null || grep -q "features.*sqlite" server/Cargo.toml'
expect "benchmark harness exists" '[ -f benchmarks/run-benchmarks.sh ]'
expect "baseline scaffold documents co-tenants" 'grep -qi "FreeLLMApi" benchmarks/baseline.md && grep -qi "MicroBin" benchmarks/baseline.md && grep -qi "Miniflux" benchmarks/baseline.md'
expect "runbook present for 8b on-device capture" 'grep -qi "on-device" benchmarks/run-benchmarks.sh'

exit $fail
