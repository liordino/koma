#!/usr/bin/env bash
# Run the wave-21 image ON the device; prove loopback serving + auth gating; clean up.
set -o pipefail
SSH="ssh -o BatchMode=yes -o ConnectTimeout=20 pi"
$SSH 'docker rm -f koma-wave21 >/dev/null 2>&1; docker run -d --name koma-wave21 -p 127.0.0.1:18101:8080 -e KOMA_TOKEN=wave21 koma:wave21 >/dev/null' || exit 1
sleep 3
code_noauth=$($SSH "curl -s -o /dev/null -w '%{http_code}' --max-time 8 -X POST -H 'content-type: application/grpc-web+proto' --data-binary \$'\x00\x00\x00\x00\x00' http://127.0.0.1:18101/koma.v1.KomaService/Ping")
code_auth=$($SSH "curl -s -o /dev/null -w '%{http_code}' --max-time 8 -X POST -H 'content-type: application/grpc-web+proto' -H 'Authorization: Bearer wave21' --data-binary \$'\x00\x00\x00\x00\x00' http://127.0.0.1:18101/koma.v1.KomaService/Ping")
$SSH 'docker rm -f koma-wave21 >/dev/null 2>&1' || true
echo "noauth=$code_noauth auth=$code_auth" >&2
[ "$code_noauth" = "401" ] && [ "$code_auth" = "200" ]
