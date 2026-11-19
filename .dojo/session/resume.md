# Resume checkpoint — M1 waves 9–12 done, wave ceiling reached (2026-11-19)

- Branch plan/koma-m1 (plan/koma-slice-0 merged to main last session). Clean tree.
- Done this cycle: 9 outbound-fetch+SSRF+rate-limiter · 10 MangaDex adapter core ·
  11 At-Home flow + download worker · 12 library/category/progress RPCs.
- 31/31 nextest, clippy clean, dojo proof signed at close.
- Next wave 13 (step RED ready in session file): SvelteKit scaffold (`npx sv create`),
  windowed vertical reader (preload ±2–3, fixed placeholders), page-anchored progress RPC,
  fetch-to-blob pages; Playwright gate: read 10 pages → reload → resume ≤2 interactions.
- Cross-cutting notes: tonic services must share ONE Routes (fallback clash); temp test dirs
  use atomic serials; cargo invocations sequential (concurrent runs race the target dir).
- Wave 14 after 13: core-loop benchmark on device (decision 16 gate).
