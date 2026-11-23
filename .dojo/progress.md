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

## Wave 10 — MangaDex adapter core — DONE (2026-11-19)
- RED 3/4 first. Green: src/source/{mod,mangadex}.rs — search/detail/chapters mapped from recorded
  {result,data[attributes+relationships]} shapes into Koma types (SearchResult carries source_key
  badge per owner decision 3), title fallback en→ja-ro→first value, cover_art relationship →
  covers URL, hard-bounded limits, urlencode fixed (self-collision bug caught at write time).
- 3 wiremock tests (search/detail+error-on-empty/feed). 25/25 nextest; clippy clean; proof signed.

## Wave 11 — At-Home flow + chapter download — DONE (2026-11-19)
- RED 5/6 first. Green: post_json added to OutboundClient (report-key POST via Api plane);
  guarded_host() unified all pre-connection rejection; MangaDexAdapter::at_home (recorded block
  shape: baseUrl/hash/data/reportUrl) + send_report best-effort; download.rs single-writer worker
  (AtHomeUrls type alias + DownloadContext struct after clippy hits, path-safe storage_dir_for
  with fail-fast key assertions, 64MB page cap at write time).
- Orphan-rule fix: PageSource impl moved into crate (test crate can't impl crate trait for
  crate-foreign adapter). Fixture FK bug in oversized-page test fixed.
- Process lesson: transient gate failures when cargo invocations overlap — sequential cargo only.
- 28/28 nextest; wave11 6/6; dojo proof signed. /pages serves freshly downloaded bytes end-to-end.

## Wave 12 — Library/category RPCs — DONE (2026-11-19)
- RED 4/6 first. Green: proto Library service (7 unary RPCs) + generated TS synced (tsc clean);
  UserId request-extension from bearer middleware (multi-user-ready, v1 single-user);
  library.rs: idempotent manga/category upserts, by-category/all list views, progress UPSERT
  with clamped ratios; router now merges BOTH services in ONE tonic Routes (fallback-clash
  lesson: two into_axum_router() merges panic).
- 3 service-DB tests + 1 wire-level test (CreateCategory through gRPC-Web + bearer, 401 without).
- Same-ms temp-collision race fixed with atomic serial. 31/31 nextest; clippy clean; proof signed.

## Wave 13 — Web UI + windowed reader — DONE (2026-11-19)
- RED 5/6 first. Green: SvelteKit scaffold (adapter-static SPA) in web/; windowed vertical reader
  (fixed 1200px placeholders, ±2-image window, blob cache; far pages unmount); page-anchored
  progress = round(scrollY/PAGE_H) + ratio, debounced UpsertProgress; resume via new GetProgress
  RPC (proto + rust + TS synced) restores scroll BEFORE first paint of the anchor.
- fetch-to-blob decision 26 implemented as transport-level authedFetch.
- Playwright gate: read to deep page → reload → restored with zero further interactions (7.6s).
- Bugs found on the way: CORS must allow connect/grpc-web protocol headers (x-grpc-web,
  connect-protocol-version...) or every browser RPC dies at preflight; connect-es client must use
  createGrpcWebTransport (tonic-web speaks gRPC-Web, not the Connect protocol — HTTP 400s explained);
  tonic/webview stray processes and Windows forbidden-socket ranges forced gate port 18100.
- Gate-quality lesson: expect() evals shared one shell — a 'cd web' persisted and silently broke
  later relative paths; ROOT resolution was one level short; ROOT must be exported into bash -c
  children. Gate now uses fresh subshells + scripts/checks/run-playwright.sh (exit-code contract).
- Evidence: wave13-check.log (7/7 PASS) + playwright13-rerun.json committed.

## Wave 14 — Core-loop benchmark on device — DONE (2026-11-18)
- bin/bench-loop added (find/add/download timed via real MangaDex over device network; source row
  registered by the bin — registry-as-data); aarch64 release rebuilt via zig recipe; shared_production_client.
- On-device friction (all process, not product): ssh link flake (retry discipline), stale server
  holding deleted DB inode (AddrInUse → worker now pkills predecessor), stdout block buffering.
- Results: device half 11.3s (find 1533 + add 15 + download 9801 = 23 pages At-Home compliant),
  host half 14.6s (open+discover over LAN 7.1, close+resume 7.5) → total ≈25.9s vs 90s gate: PASS 3.5x.
- First target ("Vivi" webtoon) was 1-page long-strip — loop run against 23-page chapter instead,
  noted in baseline.md. Evidence in benchmarks/baseline.md + core-loop.spec.ts (90s assertion in CI).

## Wave 15 — Download queue + supervised worker — DONE (2026-11-19)
- RED 4/5. Green: migration 0004 (chapter.state pending/queued/downloading/downloaded/failed +
  state_error + queued_at, index); enqueue (idempotent, never resets downloaded), next_queued
  FIFO by (queued_at, id), run_queue_once supervised pass (one task at a time, per-task failure
  contained into state='failed'+error text, done work never redone); QueueRunner trait injectable
  (deterministic test runner; RealRunner resolves adapter from the source row's key, At-Home report fired).
- Real bugs fixed: FromRow maps by column name (aliased SQL); migration default 'pending' vs
  legacy UPDATE; RPITIT style (no async-trait dep).
- FLAKE ROOT CAUSE (findings.md): Windows pid reuse + stale temp DBs -> nanos-unique
  db::temp_test_dir used by all 7 suites; 10+ consecutive full-suite passes after.
- Gate bug fixed: wave15's "all tests" grep expected plain-cargo 'test result: ok' which nextest
  never prints (34/34 was passing while the gate said FAIL).
- 34/34 nextest repeatedly; clippy clean; dojo proof signed.

## Wave 16 — Auto-update checker — DONE (2026-11-19)
- RED 4/5. Green: rpc UpdateLibrary (proto + TS synced); update.rs with UpdatableSource trait
  (MangaDexAdapter implements; mock-able), run_update_pass scoped per source_key, INSERT OR IGNORE
  + rows_affected novelty counting — idempotency contract proven (2 add, re-runs 0; incremental
  adds only new; other-source manga untouched). 37/37 nextest; clippy clean; proof signed.

## Wave 17 — Mark-read + trash window — DONE (2026-11-19)
- MarkRead RPC (proto+TS synced); trash.rs: explicit read predicate, delete_after_read-only file
  relocation to a trash SIBLING dir (leaves /pages prefix naturally), retention 7d recorded, chapter
  reset to pending (re-download possible); non-delete categories keep files. temp_test_dir got a
  counter+pid component (Windows 15ms timer granularity); rename now moves file-by-file into a
  created trash dir. 39/39 nextest; clippy clean after fixture fix; proof signed.

## Wave 18 — delete_after_read safety chain — DONE (2026-11-19)
- backup.rs: metadata-only JSON export; purge_category refuses unconfirmed/stale-backup/unknown
  category; freshness = last_backup_at >= newest trash_moved_at of the category's chapters.
- safety_chain tests 5/5: export real JSON; unconfirmed refused; stale refused; fresh-backup
  purge deletes trash files + rows; unknown category refused. 44/44 nextest; proof signed.
- M2 COMPLETE -> merge to main.

## Wave 19 — .tachibk import — DONE (2026-11-20, owner real fixture)
- Fixture: owner's real app.mihon_2026-04-08 .tachibk (gzip protobuf, 26 manga, 6589 chapters,
  mangabats+mangadex sources, no favorite=true entries on this export).
- import_tachibk.rs: decode (gzip -> prost, subset proto with Apache-2.0 notice; prost skips
  unknowns), source registry from the backup's own backupSources (name->slug), VACUUM INTO
  snapshot (nanos name), single-transaction upserts, novelty counters.
- Real-encoding lessons: Mihon v2 backups include browsed-but-not-library manga (favorite=false
  on everything here) -> import-all-as-metadata policy with favorites reported separately;
  ground truth via protoc --decode_raw; manga unique is a column constraint (no composite
  conflict); title refresh on re-import for upsert semantics; idempotency proven (re-import adds 0).
- 47/47 nextest; clippy clean; wave19 gate 6/6; dojo proof signed.

## Wave 20 — Category config UI — DONE (2026-11-20)
- Proto: UpdateCategory + ExportBackup RPCs (TS synced); global export stamps every category freshness.
- SPA /categories: create, rename, consent-gated delete_after_read flip (typed dialog), membership
  assignment, export-backup button, purge wired to PurgeCategory w/ confirm dialog + server-side
  stale-backup refusal surfaced in UI. tsconfig now extends .svelte-kit (bigint-safe keys: int64
  ids handled as strings), bind:value restricted to plain state (svelte 5 rule) -> oninput handlers.
- Playwright: create/reload/option-presence assertions (server-persistence proof), consent dialog
  accept, backup+purge flow. CSS lesson: input[type="text"] does NOT match inputs w/o explicit type.
- wave20 gate 7/7; full suite 44+; proof signed.

## Wave 21 — Docker packaging — DONE (2026-11-20)
- No docker on the Windows host; Docker 29.7.1 lives ON the pinned device -> the gate builds
  there (deployment target as the build host; ssh-based checks).
- build-once-repackage-many: scripts/build-release.sh stages the docker CONTEXT under
  target/docker-context/ (binary + SPA + Dockerfile) — build outputs never inside the repo
  source tree (lesson: staging into docker/ put generated Svelte chunks in the scan surface).
- Dockerfile: certs stage (bookworm-slim does NOT ship ca-certificates -> apt install) +
  scratch runtime; COPY --chmod=755 (Windows-staged binaries lose the exec bit); ENV db/
  storage under /data volume; server never serves the web UI (separate-origins decision 2).
- compose: koma (127.0.0.1:8080 loopback shipped default) + koma-web (nginx :8081, second
  origin), KOMA_ORIGINS wired to the web origin.
- Proof: image built on device; container run -> 401 without token, 200 with (grpc-web Ping).
- Gate fixes: stale artifact-path checks after relocation; docker-serve-check ssh'd properly.
- Windows handle remnant: docker/web old-staging dir held by a watcher; gitignored, noted.

## Wave 22 — Tauri 2 desktop shell (probation) — DONE (2026-11-23)
- desktop/ crate: tauri 2, window built programmatically from KOMA_WEB_ORIGIN (default
  127.0.0.1:8081 compose web origin) — no sidecar, zero server logic (decision 2 honored);
  KOMA_HEADLESS=1 builds the window invisible for gate runs; icon.ico via magick (tauri-build
  requires it for the Windows resource even with bundle.active=false).
- Launch proof: server (18100) + SPA preview (4173) live, shell headless 6s, process alive,
  clean log. cargo check + build green (13MB debug exe).
- Spec tuple annotations fixed ([string,string] -> string[]) after the svelte-kit tsconfig
  made tests typechecked (pre-existing looseness surfaced; runtime unaffected).
- wave22 gate 5/5; 47/47 nextest; tsc clean; dojo proof 5caa7905.
