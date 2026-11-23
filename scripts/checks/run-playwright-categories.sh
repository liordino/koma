#!/usr/bin/env bash
# Playwright runner for the wave-20 gate. Exit 0 iff Playwright exited 0.
set -o pipefail
cd "$(dirname "$0")/../../web" || exit 1
npx playwright test categories.spec.ts --project=chromium --reporter=json > ../.dojo/proof/playwright20-rerun.log 2>&1
exit $?
