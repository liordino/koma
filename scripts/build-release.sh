#!/usr/bin/env bash
# Build the release artifacts for the runtime-only Docker image (build-once-repackage-many).
# Host side: cross-build static aarch64-musl koma-server (zig recipe, .dojo/findings.md)
# + build the SPA, staging the whole docker CONTEXT under target/docker-context/ (build
# outputs live under target/, never inside the repo source tree).
set -euo pipefail
cd "$(dirname "$0")/.."

ZIG_HOME="${ZIG_HOME:-/c/tools/zig/zig-x86_64-windows-0.14.1}"
export CC_aarch64_unknown_linux_musl="C:/tools/zig/ccz.cmd"
export AR_aarch64_unknown_linux_musl="$ZIG_HOME/zig.exe ar"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="C:/tools/zig/ccz.cmd"
export CCZ_TARGET=aarch64-linux-musl
export RUSTFLAGS="-C link-self-contained=no"

CTX=target/docker-context
rm -rf "$CTX" && mkdir -p "$CTX/docker"

cargo build --release --target aarch64-unknown-linux-musl -p koma-server
cp target/aarch64-unknown-linux-musl/release/koma-server "$CTX/docker/koma-server"

(cd web && npx vite build >/dev/null)
cp -r web/build "$CTX/docker/web"
cp Dockerfile "$CTX/"

echo "staged: $CTX/docker/koma-server ($(stat -c%s "$CTX/docker/koma-server") bytes), $CTX/docker/web/"
