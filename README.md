# Koma

Koma is a **single-operator, opinionated reference implementation** of a self-hosted manga
server and reader — a ground-up rewrite of Suwayomi-Server's role, not a community Suwayomi
alternative. It is built and maintained for exactly one user: its owner. No feature is made
for an audience; requests for new sources or reader modes are tracked, never committed.

## What it is

- **Backend:** Rust — Axum + Tonic (gRPC-Web, unary in v1) + SQLx/SQLite (FTS5 trigram search).
- **Frontend:** SvelteKit SPA; Tauri desktop shell as the native client. Web and native clients
  are deliberately kept on separate origins.
- **Core loop:** search configured sources with source badges → open a title → add it to a
  category → download a full chapter → read in the windowed vertical-scroll reader → progress
  resumes in ≤ 2 interactions.

## Source policy — the two-tier reality

This project does not pretend all its sources are official public APIs:

- **Tier A — documented, third-party-invited APIs:** **MangaDex only**. Fully supported, with
  At-Home image-delivery compliance from the first source slice.
- **Tier B — undocumented endpoints, personal-risk automation** (e.g. ComicWalker, Webtoon):
  would ship as default-off cargo features, trivially removable. None exist in the public
  repo today; they are personal-use options, not endorsements.
- **MangaDex is the only source in the public shipping list.** Adapters relying on an
  unofficial/internal API are maintained privately, if at all. MANGA Plus is not supported
  publicly, period.
- A new source enters the roadmap only after a verification sheet: endpoint documentation,
  auth, rate limits, ToS review, and image-delivery origin.

## Status

Implemented and gated by tests (Rust suite + Playwright, run on the pinned Raspberry Pi
Zero 2 W): SSRF-guarded outbound fetch with per-source rate budgets; MangaDex adapter
(tier A) with At-Home-compliant chapter download; DB-backed download queue; idempotent
library auto-update; bearer-token auth on every route; path-safe page serving; windowed
vertical reader with resumable page-anchored progress (decision 30: strip-only v1);
category config with consent-gated destructive flags and backup-gated purge; staged
`.tachibk` (Mihon/Tachiyomi) import. SPA pages: library, reader, categories. Search and
adding titles ride the gRPC API (the core loop is proven end-to-end on-device, 25.9s
against a 90s budget); a dedicated search page is future UI work.

## Running it

On the deployment host (Docker):

```bash
scripts/build-release.sh          # stage the image context (binary + SPA)
docker compose up -d             # koma on 127.0.0.1:8080, web UI on 127.0.0.1:8081
```

`KOMA_TOKEN` (required) is set via the environment or `.env`. The compose ships the
loopback-only binding as the default; wider exposure is out of scope for v1 — the
sanctioned path is a private overlay (Tailscale/WireGuard) plus a reverse proxy.

## Development

Cargo workspace (`server`, `desktop`) + SvelteKit app (`web`). Gates:
`cargo nextest run`, `cargo clippy --all-targets -- -D warnings`, `npx tsc --noEmit` in
`web/`, Playwright specs in `web/tests` (need a live seeded server), and
`scripts/dojo-check.sh` for the signed proof artifact. Architecture decisions and their
rationale live in the tracked decision log (`.dojo/CONTEXT.md`).

## Authentication

Koma ships **in-app bearer-token authentication**: every API route (including page images,
served fetch-to-blob) requires a token. The token model is per-user data, so multi-user support
can arrive later without a redesign. v1 is single-user.

## DMCA response posture

Koma itself hosts no copyrighted content: it indexes configured third-party services and caches
pages privately for one operator's reading. If a rights holder nonetheless has a complaint about
this repository (documentation, code, or bundled data), send the standard notice elements
(identification, description of infringed work, claimed infringement, contact) and we will
remove or disable access to the identified material promptly.

## License

MIT — see [LICENSE](LICENSE).

## Desktop shell (probation)

The Tauri 2 shell is a thin separate-origin native client: it opens the web UI origin and
contains no server logic (no sidecar, per the separate-origins posture). It is on probation —
cut at any daily-use failure until ratified.
