# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Exactly one: the owner (Persona A). Reads manga on a phone, alternating portrait/landscape
freely, served from a Raspberry Pi Zero 2 W homelab (512 MB, co-tenants FreeLLMApi, MicroBin,
Miniflux). Dark-first, minimal clutter; every trade-off resolves to their taste.

## Product Purpose

Single-operator self-hosted manga server and reader: find titles on MangaDex, add them to
categories, download full chapters, read in a windowed vertical-strip reader with resumable
page-anchored progress. Success: the core loop end-to-end — measured 25.9 s against a 90 s
budget on the pinned device — and daily reading that never loses position.

## Positioning

An opinionated single-operator reference implementation — the leanest possible server
(4.3 MB RSS, fully static binary) — explicitly not a community Suwayomi alternative; no
source beyond verified MangaDex ships publicly.

## Operating Context

SPA on one origin (nginx :8081), API on another (koma :8080), loopback by default; the
Tauri shell wraps the web origin (no sidecar, zero server logic). Bearer-token auth on
every route, including page and cover bytes. Browsing happens on the phone too — the UI
is mobile-first by usage reality, desktop second.

## Capabilities and Constraints

Thirty-two recorded decisions in `.dojo/CONTEXT.md` are binding. The load-bearing ones for
UI work: webtoon-strip-only reader (30), separate origins (2), fetch-to-blob page loading
(26), in-app bearer auth (24), two-tier source policy with MangaDex as the only public
source (4), delete-after-read safety chain with consent-gated flags and backup-gated
purge (9), unary-only RPC (32), SQLite with FK enforcement (8). Search currently rides the
RPC API — no search page exists; closing that gap is part of the current UI overhaul.

## Brand Commitments

koma (コマ) is the manga panel — the frame the story is told in. Shipped identity: the
manga-page-layout logo (ink ground #0b0f1a, amber accent #f59e0b), wordmark, and app icon.
Dark-first always. Lean over loud: cover art does the visual lifting, never decoration.
MIT license.

## Evidence on Hand

Real `.tachibk` import (the owner's actual backup: 26 titles, 6,589 chapters) with
idempotency proof; on-device benchmark numbers in `benchmarks/baseline.md`; real
screensshots in `.github/assets/`; 47-test Rust suite plus Playwright gates; CI green on
the public repo (github.com/liordino/koma).

## Product Principles

1. Leanest possible — no feature exists for an audience; the single operator's taste decides.
2. Behavior in data — sources, categories, settings live as rows, never hard-coded branches.
3. Safety ships with the feature — destructive flows are consent-gated and backup-gated.
4. Proven on the pinned device — numbers, not claims.
5. The panel is the unit — koma (コマ): the frame the story is told in.

## Accessibility & Inclusion

Phone-primary: every target reachable one-handed. Contrast ≥ 4.5:1 for body text on the
dark ground. Keyboard-navigable controls; state is never signaled by color alone.
