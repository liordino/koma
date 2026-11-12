#!/usr/bin/env bash
# Wave 1 check — repo birth. Fails until the outcome is real.
# Evidence contract: exit 0 only if every check passes; output is the record.
set -o pipefail
fail=0
expect() { # $1 label, $2 command
  if eval "$2" >/dev/null 2>&1; then
    echo "PASS: $1"
  else
    echo "FAIL: $1"
    fail=1
  fi
}

expect "repo has at least one commit" '[ "$(git rev-list --count HEAD 2>/dev/null)" -ge 1 ]'
expect "LICENSE exists and is MIT" 'grep -qi "^MIT License" LICENSE && grep -q "Permission is hereby granted" LICENSE'
expect "README states single-operator position" 'grep -qi "single-operator" README.md && tr "\n" " " < README.md | grep -qi "not a community Suwayomi alternative"'
expect "README states two-tier source reality" 'grep -qi "tier A" README.md && grep -qi "tier B" README.md'
expect "README contains DMCA response posture" 'grep -qi "DMCA" README.md'
expect "README documents in-app bearer-token auth" 'grep -qi "bearer" README.md'
expect "CONTEXT.md carries decision-24 supersession" 'grep -q "SUPERSEDED .*decision 24" .dojo/CONTEXT.md CONTEXT.md'

subjects="$(git log --format='%s' 2>/dev/null)"
if [ -z "$subjects" ]; then
  echo "FAIL: repo has no commits"
  fail=1
else
  nonconv="$(printf '%s\n' "$subjects" | grep -cvE '^(feat|fix|docs|chore|refactor|test|perf|build|ci|style)(\(.+\))?: ' || true)"
  if [ "$nonconv" -eq 0 ]; then
    echo "PASS: all commits are conventional"
  else
    echo "FAIL: all commits are conventional ($nonconv non-conventional subject(s))"
    fail=1
  fi
fi

exit $fail
