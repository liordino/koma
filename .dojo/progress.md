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

## Wave 3 — Bearer-token auth + transport spike — DONE (2025-11-12)
- RED first: 6/6 failing. protoc installed via scoop (36.2).
- Green fixes: tonic-build as build-dependency; DbPool alias moved to db module; state wiring;
  Request bodies as axum Body; per-test temp DBs (atomic serial) — concurrent tests raced migrations;
  clippy: manual_async_fn + unused import.
- proto/koma/v1/koma.proto = single cross-repo proto source of truth (Ping spike RPC).
- Tests 6/6 (nextest): 401 without/with-wrong token; 200 unary grpc-web with valid token;
  CORS preflight echo for allowed origin. clippy -D warnings clean. dojo-check proof 479e734b…

## Wave 4 — FTS5 trigram search — DONE (2025-11-12)
- RED first: 5/5 failing. Green fixes: stale migrate! embedding (touch lib.rs to re-embed migrations);
  wave check pattern over-strict on `content =`. Probe deleted after diagnosis (SQLite 3.46, FTS5 on).
- Tests 3/3: substring 'piece'→'One Piece'; no false match; update-trigger sync. 9 total nextest green.
- Evidence: wave4-check.log + dojo-check proof committed.
