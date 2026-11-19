# Koma — .dojo/TASKS.md (waves are single verifiable outcomes)

Plan: plan/koma-slice-0 (Slice 0 / M0 — mandatory decision pass). Sources: BRIEF.md + .dojo/CONTEXT.md decisions.
Every merged wave carries committed harness output (decision 16 evidence gate).

## Wave 1 — Repo birth
status: done
- Outcome: git repo initialized; MIT LICENSE file; README stating single-operator positioning,
  two-tier source reality (tier A MangaDex only; tier B default-off, private-adapters note,
  no MANGA Plus), DMCA response posture, decision-24 supersession edit applied to the BRIEF's
  no-in-app-auth line; decision log = tracked .dojo/CONTEXT.md (decision log requirement satisfied by
  tracked artifact under track-all posture).
- Check: HEAD contains LICENSE + README + edited docs; DMCA section present; `git log` shows
  conventional commits.

## Wave 2 — SQLite operational baseline
status: done
- Outcome: cargo workspace; connection-init module (WAL, synchronous=NORMAL, foreign_keys=ON,
  busy_timeout); numbered sqlx migrations; draft schema WITH foreign keys (manga, chapter, category,
  source, user, page-progress tables); LogSink interface (decision 23, text-file JSONL impl).
- Check: integration test proves WAL on, FK enforcement on, orphan insert rejected.

## Wave 3 — Bearer-token auth + transport spike
status: done
- Outcome: Axum server with bearer-token middleware (users table, one seeded user) + tonic/tonic-web
  unary gRPC-Web route; CORS restricted to deployed UI origins.
- Check: integration test — RPC without token → 401; with token → unary response over gRPC-Web.

## Wave 4 — FTS5 trigram search verification
status: done
- Outcome: trigram tokenizer on external-content FTS5 table, verified against the sqlx-linked build.
- Check: test — substring query "piece" matches "One Piece" on the linked SQLite.

## Wave 5 — Codegen contract (buf + connect-es)
status: done
- Outcome: buf + `@connectrpc/connect-web` generating TypeScript from the repo's `.proto`.
- Check: generated TS compiles in the web workspace.

## Wave 6 — CI & dependency hygiene
status: done
- Outcome: CI config (cargo-audit, cargo-deny, gitleaks, `sqlx prepare --check`, nextest) + Cargo.lock + version pins.
- Check: dojo-check green locally; CI config committed.

## Wave 7 — Path-safe page serving pre-proto
status: done
- Outcome: `/pages` HTTP route resolving chapter_id via DB lookup, bounded page_n, canonicalize +
  prefix check; fetch-to-blob auth (decision 26).
- Check: traversal regression test (e.g. `../../etc/passwd`, encoded traversal) rejected — fails now, passes after.

## Wave 8 — Benchmarks on pinned device (Pi Zero 2 W)
status: done
- Wave 8a (local, agent-executable): ARM cross-build spike (cargo check for aarch64/armv7 musl),
  benchmark harness script + runbook committed, benchmarks/baseline.md scaffold with co-tenant
  declaration (FreeLLMApi, MicroBin, Miniflux) and Suwayomi baseline capture steps.
- Check: cargo check passes for both musl targets; runbook + harness committed.
- Wave 8b (on-device, owner-run): capture idle RSS, cold start, page p95, Suwayomi baseline;
  real numbers fill baseline.md; BLOB-vs-fs decision (decision 12) measured from the results.
- Check 8b: baseline.md contains real measured numbers from the device.

## Notes (post-execution)
- Wave 8b executed by the agent over SSH (owner granted device access mid-session).
- Suwayomi comparative baseline deferred with reason recorded in benchmarks/baseline.md — owner pace.

## M1 — Readable MVP (plan branch plan/koma-m1)

## Wave 9 — Outbound-fetch foundation (SSRF + dual-plane rate limiter)
status: done
- Outcome: central outbound-fetch module with per-adapter domain allowlists (decision-network
  baseline) and the two-plane rate limiter (API plane vs At-Home image plane) implementing the
  At-Home compliance requirement (decision 18).
- Check: unit tests — request to a non-allowlisted domain is rejected before any connection;
  request to an allowed domain passes; rate limiter enforces per-plane budgets.

## Wave 10 — MangaDex adapter core (search/manga/chapters)
status: done
- Outcome: tier-A source adapter: search, manga detail, chapter list via MangaDex API;
  source registry as table rows (behavior in data); search across all sources with badges in RPCs.
- Check: adapter tests against a local mock conforming to recorded MangaDex responses.

## Wave 11 — At-Home image flow + chapter download
status: done
- Outcome: At-Home report-key flow (signed image URLs from the report-token plane), chapter
  download worker storing files under storage root (single-writer discipline).
- Check: integration test — mock At-Home flow yields a downloaded chapter resolvable by /pages.

## Wave 12 — Library/category RPCs
status: done
- Outcome: gRPC RPCs for library CRUD, category membership, manga detail, progress upsert.
- Check: integration tests over gRPC-Web RPCs with the bearer middleware.

## Wave 13 — Web UI scaffold + windowed reader with resumable progress
status: done
- Outcome: SvelteKit SPA (`npx sv create`), windowed vertical reader (preload ±2–3 pages,
  fixed placeholders, unmount far pages), page-anchored progress persisted via RPC; fetch-to-blob
  page loading.
- Check: Playwright passes — read 10 pages, reload, progress restored in ≤2 interactions.

## Wave 14 — Core-loop benchmark on device (proposal, decision 16)
status: done
- Outcome: cold start → find → add → read 10 pages → close → resume ≤ 90 s measured on the Pi Zero 2 W.
- Check: median ≤ 90 s recorded in benchmarks/baseline.md.

## M2 — Server habits (plan branch plan/koma-m2; safety rides in the same milestone)

## Wave 15 — Download manager: queue + supervised single-writer worker
status: pending
- Outcome: chapter download queue (DB rows as queue state, FIFO), one worker that processes
  serially, survives failures (retry/skip + error state), supervision restart on next item.
- Check: integration tests — out-of-order enqueue is processed in FIFO; a failing page mid-task
  leaves the item failed, the worker moves to the next; queue drained correctly.

## Wave 16 — Auto-update checker
status: pending
- Outcome: update pass over library rows using external_id upserts, idempotent, triggered
  manually in v1.
- Check: mocked adapter returning newer chapters → rows added idempotently; re-running the
  pass twice yields no duplicates.

## Wave 17 — Read tracking + trash window
status: pending
- Outcome: explicit mark-read RPC + reading_completed event; after read, chapter files move to
  a trash directory with a retention window (timer recorded, not purged).
- Check: tests — mark read → files relocated to trash dir, retention timestamp recorded,
  originals gone from serving path, re-download available.

## Wave 18 — delete_after_read safety: backup gate + explicit purge
status: pending
- Outcome: metadata-only JSON backup export; a cleanup pass REFUSES to run without a fresh
  backup (freshness = newer than last read event being purged); permanent purge is an explicit
  per-category, user-confirmed action; never a seeded default on the primary category.
- Check: tests — cleanup blocked without fresh backup; runs after backup; purge is opt-in only.

## Open owner decision (parked, recorded in CONTEXT Decisions on answer)
- Reader mode: webtoon-strip-only (state it in UI) vs budget shared rendering primitives.
  Agent recommendation: webtoon-strip-only v1 — matches leanest-device goal; long-strips already
  render acceptably as single tall pages; revisit only if reading UX suffers.
