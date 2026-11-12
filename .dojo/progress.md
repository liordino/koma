# Progress — Koma (single per-wave log; append only)

## Session 1 — kickoff (DEFINE phase)
- Documents audited: BRIEF.md, CONTEXT.md (seed). Rigor: real, mode: autonomous, type: feature.
- commit_style: conventional; log sink: text-file JSONL (recorded in .dojo/CONTEXT.md Decisions).
- Scaffold created: .dojo/ artifacts, scripts/dojo-check.sh.

## Wave 1 — Repo birth — DONE (2025-11-12)
- RED first: 7/8 checks failing pre-implementation (README position grep fixed for wrapped line;
  conventional-commit regex fixed for plain scopes).
- Commits: `chore(repo)`, `docs`, `chore(session)`, `chore(session)` x2 (evidence + .gitattributes).
- Evidence: .dojo/proof/wave1-check.log (8/8 PASS) committed with the wave check script.
- Supersession edits applied to BRIEF.md and seed CONTEXT.md (decision 24/26).
- Note: owner should replace "Koma repository owner" copyright line in LICENSE with real name/handle.
- dojo-check (cargo gate) intentionally not runnable until wave 2 creates the workspace.

## Wave 2 — SQLite operational baseline — DONE (2025-11-12)
- RED first: 7/7 checks failing pre-implementation. nextest installed (cargo-nextest 0.9.151).
- Green fixes: max_connections moved to PoolOptions; journal_mode decoded as TEXT in test;
  wave check's migration-name expectation corrected (_0001 → 0001).
- Tests: sqlite_baseline (pragmas WAL/NORMAL/FK/busy_timeout + orphan rejection + valid insert)
  and log::writes_valid_jsonl_lines — 2/2 pass; clippy -D warnings clean.
- Evidence: .dojo/proof/check-proof (sha 76d260df…), .dojo/proof/wave2-check.log committed.
