#!/usr/bin/env bash
# Wave 21 check — Docker packaging: image builds on the pinned device (docker lives there);
# container serves ping on loopback, auth-gated; compose ships the loopback default.
set -o pipefail
fail=0
expect() { # $1 label, $2 snippet — fresh subshell, no cwd drift
  if bash -c "$2" >/dev/null 2>&1; then echo "PASS: $1"; else echo "FAIL: $1"; fail=1; fi
}
cd "$(dirname "$0")/../.." || exit 1
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; export ROOT
exec > >(tee .dojo/proof/wave21-check.log) 2>&1

expect "Dockerfile exists (runtime-only, no in-container compile)" '[ -f "$ROOT/Dockerfile" ] && grep -q "build-once" "$ROOT/Dockerfile"'
expect "compose ships loopback binding" '[ -f "$ROOT/docker-compose.yml" ] && grep -q "127.0.0.1:8080:8080" "$ROOT/docker-compose.yml" && grep -q "8081" "$ROOT/docker-compose.yml"'
expect "release artifacts staged (binary + web)" '[ -f "$ROOT/target/docker-context/docker/koma-server" ] && [ -f "$ROOT/target/docker-context/docker/web/index.html" ]'
expect "context shipped to the device" 'ssh -o BatchMode=yes -o ConnectTimeout=20 pi "mkdir -p ~/koma-docker" && scp -q -r "$ROOT/target/docker-context/." pi:~/koma-docker/ && ssh -o BatchMode=yes -o ConnectTimeout=20 pi "test -f ~/koma-docker/docker/koma-server"'
expect "image builds on the pinned device" 'ssh -o BatchMode=yes -o ConnectTimeout=20 pi "cd ~/koma-docker && docker build -q -t koma:wave21 . 2>&1 | tail -1 | grep -q sha256"'
expect "container serves on loopback (401 without token, 200 with)" 'bash "$ROOT/scripts/checks/docker-serve-check.sh"'

exit $fail
