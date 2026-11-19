# Resume checkpoint — M2 complete, merged to main (2026-11-19)

- main holds M1 + M2 (60 commits), tree clean, 44/44 tests green incl. the 5-test safety chain.
- M2 delivered: DB-backed FIFO download queue + supervised worker (w15); idempotent auto-update
  pass w/ UpdateLibrary RPC (w16); explicit mark-read + trash window with 7d retention record
  (w17); metadata-only backup export + stale-backup-gated purge with per-category user-confirmed
  PurgeCategory RPC (w18) — the stop conditions are enforced in code, not just documented.
- Deferred/open: Suwayomi hardware baseline (physically infeasible on Zero 2 W — recorded);
  Docker packaging + .tachibk import (M3); Tauri shell probation; online sources beyond MangaDex
  (verification sheets required).
- Next milestone: M3 — Publish (staged .tachibk import, category config UI, Docker packaging,
  Tauri shell). Requires owner decisions on repo publication + README positioning.
