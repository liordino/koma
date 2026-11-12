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
  shipped as default-off build features, usable privately, trivially removable. They are not
  endorsements.
- **MangaDex is the only source in the public shipping list.** Adapters relying on an
  unofficial/internal API are maintained privately, if at all. MANGA Plus is not supported
  publicly, period.
- A new source enters the roadmap only after a verification sheet: endpoint documentation,
  auth, rate limits, ToS review, and image-delivery origin.

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
