# Resume checkpoint — M1 wave 13 done, ceiling pause (2026-11-19)

- Branch plan/koma-m1. Clean tree. 34 tests green incl. the Playwright reader gate (7/7 PASS).
- Done: waves 9–13 (fetch foundation, MangaDex adapter, At-Home + download worker, library RPCs,
  SvelteKit windowed reader with resumable progress).
- Next wave 14: core-loop benchmark on Pi Zero 2 W (cold start → find → add → read 10 pages →
  close → resume ≤ 90 s). Needs: release binary rebuilt (cross recipe in .dojo/findings.md),
  SPA static build hosted for the device API origin, benchmark harness extension, MangaDex live
  search on device (real network, At-Home compliance on the device is exercised here).
- Then: M1 completion review → possible merge decision (owner), and M2 (download queue,
  auto-updates, delete-after-read safety design).
- Dev-run recipe (host): export KOMA_DB_URL/KOMA_STORAGE/KOMA_TOKEN/KOMA_ADDR=127.0.0.1:18100,
  bench-seed, koma-server, `npx playwright test` in web/. Ports note: 18080 fell into Windows'
  forbidden socket range (os error 5/10013) — 18100 chosen; update spec + gate together.
