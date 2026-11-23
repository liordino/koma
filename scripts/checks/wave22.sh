#!/usr/bin/env bash
# Wave 22 check — Tauri 2 desktop shell on probation (separate-origin native client).
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave22-check.log) 2>&1

expect "desktop shell crate exists" '[ -f "$ROOT/desktop/src/main.rs" ] && grep -q "tauri" "$ROOT/desktop/Cargo.toml"'
expect "shell config present (no sidecar; window from web origin)" '[ -f "$ROOT/desktop/tauri.conf.json" ] && grep -q "KOMA_WEB_ORIGIN" "$ROOT/desktop/src/main.rs"'
expect "probation note recorded" 'grep -rqi "probation" "$ROOT/desktop/src/main.rs" "$ROOT/README.md"'
expect "cargo check on the desktop crate" 'cd "$ROOT" && cargo check -q -p koma-desktop 2>&1'
expect "shell launches against a running server (headless)" 'bash "$ROOT/scripts/checks/tauri-launch-check.sh"'

exit $fail
