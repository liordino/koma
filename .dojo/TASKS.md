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
status: pending
- Outcome: cargo workspace; connection-init module (WAL, synchronous=NORMAL, foreign_keys=ON,
  busy_timeout); numbered sqlx migrations; draft schema WITH foreign keys (manga, chapter, category,
  source, user, page-progress tables); LogSink interface (decision 23, text-file JSONL impl).
- Check: integration test proves WAL on, FK enforcement on, orphan insert rejected.

## Wave 3 — Bearer-token auth + transport spike
status: pending
- Outcome: Axum server with bearer-token middleware (users table, one seeded user) + tonic/tonic-web
  unary gRPC-Web route; CORS restricted to deployed UI origins.
- Check: integration test — RPC without token → 401; with token → unary response over gRPC-Web.

## Wave 4 — FTS5 trigram search verification
status: pending
- Outcome: trigram tokenizer on external-content FTS5 table, verified against the sqlx-linked build.
- Check: test — substring query "piece" matches "One Piece" on the linked SQLite.

## Wave 5 — Codegen contract (buf + connect-es)
status: pending
- Outcome: buf + `@connectrpc/connect-web` generating TypeScript from the repo's `.proto`.
- Check: generated TS compiles in the web workspace.

## Wave 6 — CI & dependency hygiene
status: pending
- Outcome: CI config (cargo-audit, cargo-deny, gitleaks, `sqlx prepare --check`, nextest) + Cargo.lock + version pins.
- Check: dojo-check green locally; CI config committed.

## Wave 7 — Path-safe page serving pre-proto
status: pending
- Outcome: `/pages` HTTP route resolving chapter_id via DB lookup, bounded page_n, canonicalize +
  prefix check; fetch-to-blob auth (decision 26).
- Check: traversal regression test (e.g. `../../etc/passwd`, encoded traversal) rejected — fails now, passes after.

## Wave 8 — Benchmarks on pinned device (needs owner hardware)
status: pending-blocked
- Outcome: pinned device chosen, ARM/Tonic/TLS build spike, Suwayomi baseline in benchmarks/baseline.md,
  BLOB-vs-fs storage decision measured (decision 12).
- Check: baseline.md committed with real numbers on the pinned device.
