# CONTEXT.md (seed) — Koma

## Glossary

- **Koma** — self-hosted, single-user manga server + reader; ground-up rewrite of Suwayomi-Server.
- **Readable MVP (M1)** — collapsed slices 1–4: badge search → MangaDex title → category add → full-chapter download → windowed vertical read with resumable progress.
- **Slice 0 (M0)** — mandatory prerequisite spike + decision pass: schema + migrations, transport spike, security baseline, FTS5 verification, codegen contract, version pins, model calibration, benchmark device + Suwayomi baseline, spec-conflict sweep + decision log.
- **Decision log** — repo-committed record replacing the "all decisions finalized" claim; per-item status (decided/deferred/open) with rationale; reopening requires a written, timeboxed spike result committed to the repo.
- **Spec-conflict sweep** — one documented edit reconciling the input's internal contradictions (three category-seed variants, search short-circuit, the official-API premise, stale learning text) before the document is re-fed to AI sessions.
- **Tier A / Tier B sources** — the two-tier legal model replacing "official public APIs only": tier A = documented, third-party-invited APIs (MangaDex only); tier B = undocumented endpoints used as personal-risk automation (ComicWalker, Webtoon), default-off.
- **MANGA Plus restriction** — dropped from the public shipping list; a public repo advertising an internal Shueisha API is the plan's highest-probability takedown trigger; private/local branches remain the owner's personal-use option.
- **At-Home flow** — MangaDex image delivery requiring report-key-signed URLs, per-session tokens, and documented rate limits on both API planes; compliance belongs in readable-MVP acceptance criteria, not the adapter backlog.
- **delete_after_read** — destructive category flag deleting a chapter after reading; default-on on the primary category is the plan's highest data-risk configuration; ships only with the full safety design.
- **Read predicate** — the unspecified event that counts as "read" and triggers deletion; the owner's "read for X time" is a compressed single-sentence lean-in, not a spec.
- **Windowed rendering** — reader virtualization: preload ~±2–3 pages, fixed-size placeholders, unmount far pages; mandatory for 50+ page chapters.
- **Page-anchored progress** — progress stored as a page-anchored scroll ratio so resume survives reload and device changes.
- **Path-safe page serving** — `/pages` resolution via DB-keyed chapter lookup, bounded `page_n`, canonicalize + prefix check; traversal regression test from day one (frequent LLM-codegen bug class).
- **SQLite baseline** — WAL, synchronous=NORMAL, foreign_keys=ON, busy_timeout set in one connection-init module; single-writer discipline for four concurrent writers; numbered sqlx migrations with `sqlx prepare --check` in CI.
- **Trigram FTS** — trigram tokenizer on external-content FTS5, verified against the sqlx-linked SQLite build in slice 0; makes CJK and substring title search work ("piece" matches "One Piece").
- **buf + connect-es** — proto toolchain + `@connectrpc/connect-web` as the browser/Tauri gRPC-Web client generating TypeScript from the same `.proto`; adopted before the first `.proto` (the client choice was "Dunno").
- **Unary-only contract** — v1 RPCs are unary; the owner's full-chapter-download-then-read instinct ratified conditionally on a measured chapter-size envelope + confirmed At-Home URL expiry; streaming cut permanently if both conditions pass.

- **BLOB switch** — compressed page BLOBs enter the schema only if a measured benchmark on the pinned device favors them; otherwise filesystem bytes with the DB as index.
- **Source verification sheet** — required per-source record (endpoint documentation, auth, rate limits, ToS clause, image-delivery origin) before any source beyond MangaDex enters the roadmap.
- **Committed-evidence gate** — the one enforcement the owner accepted: a merged slice without committed harness output (test results + benchmark numbers) does not count as done.
- **Decision-gated AI building** — Crush + GLM-5 implementation with the owner gating every step via dojo-skills; learning rides on the owner's gated decisions (the resolution of the learning-vs-AI contradiction; the source document's stale Section 1 text must be updated to reflect it).
- **Model calibration** — slice 0 runs the same Rust/tonic/SQLx task through GLM-5 and the Claude fallback with pre-set switch criteria; the declared fallback targets the wrong failure mode.
- **Pinned device** — the not-real "Pi 2 Zero W" replaced by an exact product (Pi 2 vs Pi Zero W) with co-tenant services declared and the ARM/TLS build proven; Suwayomi baseline captured on the same hardware.
- **Persona A** — the owner; the single user every trade-off resolves to.

## Non-Goals

- Reader modes beyond vertical scroll — decide webtoon-strip-only deliberately and state it in the UI, or budget shared rendering primitives (open decision)
- Multi-user features; auth beyond the single chosen posture; in-process remote access
- Self-built TLS
- Source breadth beyond verified sources (verification sheet required per source)
- Tauri Mobile (the PWA-on-Android experiment is the short-term path and gates slice 11)
- Offline client page cache in v1 — cut with documented degradation, or designed as its own slice
- Auto-updater in v1 unless explicitly designed (signing keys + hosted manifest are an unbudgeted sub-project)
- Plugin architecture — post-cutover only; explicitly deferred in writing, not by omission
- Public Suwayomi-Server API compatibility
- MANGA Plus in the public shipping list
- Enforced session caps / two-strikes re-scope — proposed, declined by the owner ("on my own pace"); advisory only, momentum risk owner-borne
- Streaming RPCs in v1 (per the unary-only contract; cut permanently if the ratification conditions pass)

## Decisions

1. **Stack freeze stands** — Rust/Axum/Tonic/tonic-web/SQLx/SQLite/FTS5/SvelteKit SPA/Tauri 2; each individually mature and trend-aligned; no panel objection.
2. **Separate repos, containers, origins** — owner constraint, deliberate ("must be read from different endpoints — web page, app, native stuff"); no sidecar; the five-specialist single-binary/monorepo recommendation recorded as rejected; its cost chain designed deliberately (auth mechanism, CORS, cross-repo proto contract).
3. **Search shows all configured sources with source badges** — owner decision superseding the input's first-hit short-circuit; the document must be edited to match before re-feed.
4. **Two-tier source policy replaces "official public APIs only"** — tier A: MangaDex only; tier B: ComicWalker/Webtoon as default-off cargo features with runtime registration (one-flag removal); the README describes the reality, not the false premise.
5. **MANGA Plus dropped from the public shipping list** — stop condition, reproduced verbatim in BRIEF.md.
6. **Network trust model** — loopback shipped-default binding; bearer-token middleware + Host/Origin validation; beyond-LAN only via private overlay (Tailscale/WireGuard); no self-built TLS; in-process remote unsupported. "Maybe no need for auth" resolves to exactly one written posture — loopback-only + Tailscale with no in-app auth is the option consistent with the owner's answer, with the copyright note that non-loopback exposure moves the operator from private copying toward apparent distribution.
7. **`/pages` auth redesign** — cookie-based schemes are dead under separate origins; mechanism is fetch-to-blob with Authorization headers (paired with windowed rendering) or short-lived HMAC-signed page URLs; decided before the first `.proto`.
8. **SQLite baseline before the first migration** — WAL, synchronous=NORMAL, foreign_keys=ON, busy_timeout in one connection-init module; single-writer discipline; numbered sqlx migrations + `sqlx prepare --check` in CI.
9. **Destructive-cleanup safety design required** — trash directory with retention window; cleanup blocked without a fresh backup; permanent purge explicit, per-category, user-confirmed — never a seeded default on the primary category (stop condition). The owner's "X time" delay reshapes the trigger; read predicate and retention remain open.
10. **Reader architecture** — windowed rendering + page-anchored progress are mandatory, not optional.
11. **Unary-only v1 RPC contract with full-chapter-download-then-read** — conditionally ratified; streaming cut permanently if the chapter-size envelope and At-Home URL expiry conditions pass, else it is designed as its own slice.
12. **BLOB storage switch** — compressed page BLOBs only on a measured benchmark win on the pinned device; default is filesystem bytes + DB-as-index.
13. **Source admission switch** — per-source verification sheet required before any source beyond MangaDex enters the roadmap; tools or sources that fail verification do not enter the roadmap (stop condition).
14. **Import reclassified** — the `.tachibk`/Mihon backup schema is public, Apache-2.0, license-clean with notice preservation; the task is mapping Suwayomi-authored export data (verified with the owner's real file early), not reverse-engineering; staged import adopted (dry-run report → `VACUUM INTO` snapshot → single-transaction commit with external-ID upserts).
15. **Decision governance** — decision log replaces the "finalized" claim before any re-feed; reopening only via written, timeboxed, repo-committed spike results; schema churn controlled by versioned migrations.
16. **Evidence gate** — committed harness output (test results + benchmark numbers) accompanies every merged slice; owner-accepted, the only enforcement mechanism.
17. **Tooling commitments before slice 1** — buf + connect-es codegen with a freshness CI check; version pins + Cargo.lock + cargo-deny/Dependabot; named test harness (cargo-nextest, wiremock, tower oneshot, Playwright, tonic reflection/grpcurl) recorded in the handover doc; slice-0 model calibration with pre-set switch criteria; stale scaffolding commands replaced (`npx sv create`, not `npm create svelte`).
18. **At-Home compliance in readable-MVP acceptance criteria** — report-key token flow + dual-plane rate limiting from the first source slice, not later.
19. **Learning resolution recorded** — learning goals are realized through decision-gated AI building (dojo-skills gates); the source document's Section 1 must be updated so the resolution survives future re-feeds.
20. **Plugin architecture explicitly deferred** until after cutover, in writing — not by omission (it is a scope bomb: dynamic loading vs WASM vs out-of-process, ABI stability, sandboxing, update channels).
21. **Version gates before repo creation** — LICENSE chosen (Apache-2.0/MPL-2.0/MIT-compatible, with notice preservation for Apache-2.0 imports), DMCA response posture pre-committed, single-operator README positioning written before the first star.
22. **Commit style: conventional** (`type(scope): summary`) — consented at session 1 kickoff.
23. **Application log sink: text file (JSONL)** — code depends on a `LogSink` interface, never a destination; remote sinks (if ever added) require local fallback. Consented at session 1 kickoff.
24. **In-app auth posture: bearer-token middleware** — supersedes decision 6's "no in-app auth" resolution (owner pivot at session 1 kickoff: multi-user is intended, post-v1). BRIEF.md's loopback-only-no-auth line is superseded and edited in wave 1. Multi-*user features* remain a v1 non-goal; v1 ships the mechanism (token middleware + users table, one seeded user) so multi-user arrives later without redesign.
25. **LICENSE: MIT** — meets decision 21's compatibility bar; Apache-2.0 notice preservation applies to Mihon imports. Consented session 1 kickoff.
26. **`/pages` auth: fetch-to-blob with Authorization headers** — closes decision 7's two-option fork; one auth mechanism for the whole API, pairs with windowed rendering; HMAC-signed URLs recorded as a revisit-later optimization.
27. **Tracking posture: track-all** — the full `.dojo/` footprint travels in git; concurrent mid-wave edits across machines conflict at pull and the human picks one machine's truth (accepted). Only standard build-hygiene (target/, node_modules, .env) is ignored.
