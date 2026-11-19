#!/usr/bin/env bash
# Playwright runner for the wave-13 gate. Exit 0 iff Playwright exited 0.
set -o pipefail
TRACE=/tmp/run-playwright-trace.log
echo "=== runner start pid=$$ pwd=$(pwd)" >> $TRACE
cd "$(dirname "$0")/../../web" || { echo "cd failed" >> $TRACE; exit 1; }
echo "runner cd ok → $(pwd)" >> $TRACE
npx playwright test --project=chromium --reporter=json > ../.dojo/proof/playwright13-rerun.log 2>&1
code=$?
echo "runner playwright rc=$code" >> $TRACE
tail -c 300 ../.dojo/proof/playwright13-rerun.log >> $TRACE || echo "no log file!" >> $TRACE
exit "$code"
