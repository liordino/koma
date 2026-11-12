# BRIEF.md — Koma (Self-Hosted Manga Server & Reader)

**Feasibility verdict: challenged (PSD v0.1.0, 2025-11-05).** The panel certified the stack and the niche as feasible; the specification as written is **not load-bearing**. PSD sections 3 and 8 must be read before any slice is executed. The headline failure: "all decisions finalized" failed inspection — five-plus explicit non-decisions and several contradictions — so AI sessions fed the document will silently invent its open decisions. Slice 0 (below) is the fix.

Status: pre-implementation — no code, no repos, no schema.

## What it is

A ground-up rewrite of Suwayomi-Server: a self-hosted, single-user manga server and reader for a homelab Pi-class device running alongside other light services.

- **Backend:** Rust — Axum + Tonic/tonic-web + SQLx/SQLite (FTS5).
- **Frontend:** SvelteKit SPA; Tauri desktop shell as the native client. Web client and native app are deliberately kept on **separate origins** (owner constraint — "must be read from different endpoints"; no sidecar).
- **Transport:** gRPC-Web for structured data; plain HTTP raw binary for page images. Unary-only in v1; full-chapter download before reading.
- **Design philosophy:** behavior in data — categories, sources, and reader settings live as table rows, not hard-coded branches.
- **Methodology:** fully AI-assisted implementation (Crush + GLM-5, Claude fallback) with the owner gating every step via dojo-skills decision gates. Learning (Rust, Svelte, gRPC, data-oriented design) is realized through the owner's gated decisions, not hand-written code.

## Core loop (MVP: the Minimum Readable Path)

Search all configured sources with **source badges** (owner decision — supersedes the input's first-hit short-circuit; the document must be edited to match) → open a MangaDex title → add it to a category → chapter **fully downloaded, then read** in the windowed vertical-scroll reader → progress persisted and resumable in ≤ 2 interactions.

MangaDex only for the MVP — with the At-Home report-key image flow and dual-plane rate limiter implemented from this slice, not deferred. Windowed rendering (preload ±2–3 pages, fixed-size placeholders, unmount far pages) and page-anchored scroll-ratio progress are mandatory, not optional: a naive continuous strip will not hold 50+ page chapters (memory growth, scroll jank, lost position).

## Main user

Exactly one: **the owner** (Persona A). Every trade-off resolves to their taste — vertical scroll only, dark-first, minimal clutter. The public repo states its position in the README before the first star: an opinionated single-operator reference implementation, **not** a community Suwayomi alternative; source requests tracked, never committed. The single-operator statement also keeps the GDPR household exemption accurate.

## Success metric

All currently **asserted, not measured** — slice 0 converts each into recorded evidence:

- idle RSS < 100 MB; cold start < 2 s; LAN page p95 < 50–200 ms — each against a recorded Suwayomi baseline (`benchmarks/baseline.md`) on identical hardware, on a **pinned, real** device. The input's "Pi 2 Zero W" is not a real product; pin Pi 2 (ARMv7) vs Pi Zero W (ARMv6), declare co-tenant services, and prove tonic + its TLS stack builds on it.
- **MVP signal:** one chapter read end-to-end with progress restored after reload, on the pinned device.
- **Core-loop benchmark (proposal):** cold start → find → add → read 10 pages → close → resume ≤ 90 s on localhost.
- **Cutover:** metadata-only migration satisfies the owner; the panel's proposed 14-day exclusive-use gate is recorded as proposed, not accepted.
- **Enforcement:** the owner declined per-slice caps and pace rules; momentum risk is owner-borne. The one accepted gate: **a merged slice without committed harness output (test results + benchmark numbers) does not count as done.**

## Hard constraints

- **Two-tier source policy** (restated because the input's "official public APIs only" is factually false for 3 of 4 sources): tier A = documented, third-party-invited APIs — **MangaDex only**; tier B = undocumented endpoints, personal-risk automation (ComicWalker, Webtoon) — default-off cargo features with runtime registration, one-flag removal. The README describes this reality, not the false premise.
- **Per-source verification sheet** (endpoint docs, auth, rate limits, ToS clause, image-delivery origin) required before any source beyond MangaDex enters the roadmap.
- **Network baseline:** loopback shipped-default binding; bearer-token middleware + Host/Origin validation; central outbound-fetch module with per-adapter domain allowlists (SSRF); image-decode limits pre-decode; cargo-audit + cargo-deny + gitleaks in CI before slice 1.
- **Path-safe page serving:** `/pages` resolves `chapter_id` via DB lookup, bounded `page_n`, canonicalize + prefix check; traversal regression test from day one (frequent LLM-codegen bug class).
- **SQLite operational baseline before the first migration:** one connection-init module setting WAL, synchronous=NORMAL, foreign_keys=ON, busy_timeout; single-writer discipline for the four concurrent writers. SQLite ships with FK enforcement **off** and the schema sketch has none — orphaned rows are the exact corruption class SQLite was chosen to prevent. Numbered sqlx migrations; `sqlx prepare --check` in CI.
- **Destructive-cleanup safety:** `delete_after_read` is default-on for the primary category while the only backup is a metadata-only JSON dump — as seeded, it destroys the only copy of consumed chapters. Required design: trash directory with retention window; cleanup gated on a fresh backup; permanent purge explicit and user-confirmed. The owner's "delete after read for X time" reshapes the trigger into a delayed one; read predicate and retention remain open ("X time" is an owner lean-in, not a spec).
- **Separate-origins consequence chain (decided before the first `.proto`):** the owner rejected same-origin serving; the five-specialist monorepo/single-binary recommendation is recorded as rejected. Consequences designed deliberately, not inherited: SameSite cookie auth is dead under this topology, so `/pages` auth is either fetch-to-blob with Authorization headers (paired with windowed rendering) or short-lived HMAC-signed page URLs [resolved 2025-11-12 by session decision 26: fetch-to-blob]; CORS restricted to the deployed UI origins; buf keeps the `.proto` the single cross-repo source of truth. "Maybe no need for auth at all?" resolves to exactly one written posture — in-app bearer-token auth [SUPERSEDED 2025-11-12 by session decision 24; the earlier loopback-only-no-in-app-auth posture is replaced and multi-user features remain post-v1 — see .dojo/CONTEXT.md], plus the copyright line that non-loopback exposure moves the operator from private copying toward apparent distribution.
- **Governance:** the "all decisions finalized" claim is replaced by a repo-committed **decision log** (per-item decided/deferred/open + rationale); reopening requires a written, timeboxed spike result committed to the repo. Crush + GLM-5 is uncalibrated on this stack's hardest error classes (SQLx compile-time verification, tonic/prost macros — not borrow-checker errors); slice 0 doubles as model calibration with pre-set switch criteria. Stale scaffolding commands replaced (`npx sv create`, not `npm create svelte`).
- **Stack freeze stands:** Rust/Axum/Tonic/tonic-web/SQLx/SQLite/FTS5/SvelteKit/Tauri — each found individually mature and trend-aligned.
- **Before repo creation:** LICENSE chosen (Apache-2.0/MPL-2.0/MIT-compatible — keeps Mihon/Apache-2.0 imports clean, with notice preservation) and the DMCA response posture pre-committed.

### Stop conditions (as stated — never softened)

- "Drop MANGA Plus from the public shipping list — the author can maintain such adapters in a private/local branch for personal use without publishing them." (— Legal Compliance Checker)
- "If a tool or source can't pass verification, it doesn't enter the roadmap." (— Tool Evaluator)
- "Remote | Not supported in-process | Reverse proxy + TLS mandatory — only documented path for internet exposure." (— Application Security Engineer)
- "Beyond-LAN access is out of scope for v1; the sanctioned path is a private overlay (Tailscale/WireGuard), not self-built TLS." (— Security Architect)
- "Cleanup blocked without a fresh backup; permanent purge as an explicit, per-category, user-confirmed action — not a seeded default on the primary category." (— Data Engineer)
- "A merged slice without committed harness output (test results + benchmark numbers) does not count as done." (— Reality Checker)

## Explicit exclusions (v1)

- Reader modes beyond vertical scroll (decide deliberately webtoon-strip-only and state it, or budget shared rendering primitives — open, see PSD question list)
- Multi-user features or auth beyond the one chosen posture; in-process remote access
- Self-built TLS
- Source breadth beyond verified sources
- Tauri Mobile; offline client page cache in v1 (cut with documented degradation, or designed as its own slice)
- Auto-updater in v1 unless explicitly designed (signing keys + hosted manifest = unbudgeted sub-project)
- Plugin architecture — post-cutover only, deferred **explicitly in writing**, not by omission
- Public Suwayomi-Server API compatibility
- MANGA Plus from the public shipping list
- Streaming RPCs — cut permanently if the full-chapter-download contract is ratified on a measured chapter-size envelope + confirmed At-Home URL expiry

## Build-order intent

1. **M0 — Slice 0 (mandatory decision pass: nothing ships until it exists).** SQLite connection-init + schema draft with committed migrations; transport spike (Axum + tonic-web, one unary gRPC-Web call in a browser and a Tauri webview); security baseline commit; FTS5-trigram verification against the sqlx-linked build; buf + connect-es codegen contract; version pins + Cargo.lock + cargo-deny; model calibration (GLM-5 vs Claude on the same Rust/tonic/SQLx task); pinned benchmark device + ARM/TLS build spike + Suwayomi baseline; spec-conflict sweep + decision log. The auth/page-serving mechanism **precedes the first `.proto`**.
2. **M1 — Readable MVP** (panel's strong recommendation: collapse original slices 1–4 — the existing backlog hides the product's core value behind three scaffolding slices). MangaDex adapter (tier A, At-Home-compliant, per-adapter allowlist), all-source badge search, detail, library CRUD + categories, path-safe binary page serving, windowed vertical reader with page-anchored progress.
3. **M2 — Server habits (safety rides in the same milestone, not after).** Download queue (single-writer, queue sequence, worker supervision), auto-updates (external-ID upserts, idempotent), delete-after-read (read predicate + retention + trash window + re-read affordance + fresh-backup gate).
4. **M3 — Publish.** Staged `.tachibk` import (Mihon schema is public and Apache-2.0-licensed — mapping risk, not reverse-engineering; tested with the owner's real export early), category config UI (cheap pull-forward), Docker packaging (loopback shipped default), Tauri desktop shell on probation until M1/M2 survive ~a month of daily use; auto-updater cut or explicitly budgeted.
5. **Post-cutover, deliberately deferred.** Plugin architecture (in writing), Tauri Mobile (PWA-on-Android experiment gates slice 11), streaming RPCs (conditional permanent cut), additional sources (verification sheet per source).