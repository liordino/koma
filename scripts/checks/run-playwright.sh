#!/usr/bin/env bash
# Playwright runner for the wave-13 gate — scoped to the reader spec. The device-benchmark
# spec (core-loop, wave 14) needs the live Pi server and is not part of local UI gates.
# Exit 0 iff Playwright exited 0.
set -o pipefail
cd "$(dirname "$0")/../../web" || exit 1
npx playwright test reader.spec.ts --project=chromium --reporter=json > ../.dojo/proof/playwright13-rerun.log 2>&1
exit $?
