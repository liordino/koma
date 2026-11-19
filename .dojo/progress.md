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

## Wave 5 — buf + connect-es codegen contract — DONE (2026-11-13)
- RED first: 5/5 failing (also fixed a contract bug in the check script: { } | tee swallowed
  the fail flag into a subshell; replaced with exec tee).
- buf installed via npm @bufbuild/buf (scoop release URL broken). Remote plugin switched to
  buf.build/bufbuild/es (connect-es v2 world); stale koma_connect.ts from the v1-lineage plugin removed.
- buf lint forced service rename Koma → KomaService; propagated to tonic module
  koma_service_server and test route /koma.v1.KomaService/Ping.
- Smoke web/src/index.ts builds the connect transport + typed client; tsc --noEmit clean.
- Tests 9/9 nextest; clippy clean; dojo-check proof committed.

## Wave 6 — CI & dependency hygiene — DONE (2026-11-13)
- RED first: 8/10 failing (lockfile + dojo-check already green carry-over).
- Green: .github/workflows/ci.yml (build/clippy -D warnings/nextest; hygiene: gitleaks, cargo-deny,
  cargo-audit; buf lint job; sqlx prepare --check gated on .sqlx presence until first query! macro).
- deny.toml: license allowlist with notice-preservation note, yanked=deny, unknown-source deny.
- Known-accepted Semgrep advisories: mutable action tags, missing top-level permissions (default
  restricted now applies), artipacked shallow checkout — backlog candidate, not wave blockers.
- Evidence: wave6-check.log + dojo-check proof committed.

## Wave 7 — Path-safe /pages pre-proto — DONE (2026-11-13)
- RED first: 6/6 failing (one check-pattern bug fixed: 'from .+chapter' demanded chars between
  FROM and chapter; rewritten 'from.*chapter').
- Green fixes: std::path Path collision with axum Path extractor; PagesState Clone for with_state;
  probe isolated 400 → GrpcWebLayer rejects plain GETs — layer now wraps only grpc routes, not
  the whole app; missing seed_owner in the /pages fixture (auth middleware correctly 401'd).
- Migration 0003: chapter.storage_dir. Handler: DB-keyed resolution, page_n ≤ 2000 bound,
  canonicalize + prefix check, 64 MB cap.
- Tests 14/14 nextest; traversal tests prove: traversal storage_dir rejected, page bounds enforced,
  unknown chapter 404, valid request serves bytes. Evidence committed (proof 7 committed).

## Wave 8a — ARM musl cross-build spike + benchmark harness — DONE (2026-11-13)
- Device confirmed by owner: Pi Zero 2 W (aarch64); co-tenants FreeLLMApi, MicroBin, Miniflux declared (decision 28).
- RED 5/7 → green: rustup musl targets; zig journey (0.17 parse fail → 0.14.1 pinned from
  ziglang.org; musl.cc rejected as Linux-ELF-only on Windows; ccz.cmd wrapper filters cc-rs's
  rust-flavored --target, found batch `for` splits on `=` and cc quotes args);
  AR_<target>=zig.exe ar completes the C toolchain. Both targets cargo check green (7/7).
- Harness run-benchmarks.sh (cold start, idle RSS, page p95; JSONL; co-tenants declared) +
  baseline.md scaffold with PENDING placeholders and Suwayomi capture procedure committed.
- Wave 8b (on-device numbers) remains blocked on owner: needs device access + KOMA_BENCH_TOKEN.

## Wave 8b — On-device benchmarks — DONE (2026-11-18, agent-run over owner-granted SSH)
- main.rs made serveable (env-configured boot: DB, migrate, seed, router); bench-seed bin added.
- zig-cross-built static aarch64 release (5.1 MB) via link-self-contained=no (zig owns CRT/libc).
- On-device findings: harness cold-start loop bug (until-loop spun on 401), port collisions with
  co-tenants, sudo env drop (-E), harness must be the solo server orchestrator — all fixed.
- Results (co-tenants live): idle RSS 4.3 MB (target <100MB), genuine cold start 768 ms
  (root drop_caches; warm 451/612), page p95 62–65 ms (envelope 50–200). Raw JSONL in benchmarks/raw/.
- Decision 29 recorded: fs storage stands, BLOBs rejected on measurement (decision 12 satisfied).
- Suwayomi comparative baseline deferred: no JVM on device; JVM+Suwayomi on ~190MB free RAM
  alongside co-tenants risks OOM-skewing the comparison (reason recorded in baseline.md).

## Suwayomi host reference — measured (2026-11-18)
- Owner-authorized direct test on host machine. Suwayomi on port 4567 (custom), API compatible
  with expectation; page delivery is WEBP through source-fetch cache (first-touch expensive).
- Reference numbers: idle RSS 578 MB, restart-to-first-200 8.6s, page p95 233ms (host).
- NOT the device baseline — labeled informational in baseline.md. Device JVM infeasibility now
  evidence-backed (578MB > 512MB RAM).

## Wave 9 — Outbound-fetch foundation — DONE (2026-11-18)
- RED 6/6 first. Green: OutboundClient (domain allowlist with suffix-boundary match, rejection
  pre-connection pre-throttle), dual PlaneLimiter (Api / AtHome) as try_grant+wait-hint so no
  mutex is held across await (clippy catch), reqwest+rustls for static-friendliness.
- tests: 9 new (unit: allowlist boundaries incl. lookalike + case; wiremock: allowlisted JSON ok,
  disallowed host hits ZERO wire requests, lookalike rejected, At-Home budget throttles in time).
- 22/22 nextest; clippy -D warnings clean; evidence committed (wave9-check.log + check-proof).
