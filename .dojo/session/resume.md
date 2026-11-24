# Resume checkpoint — M4 UI overhaul complete, merged, CI green (2026-11-24)

- main holds M1-M4; the Ink & Amber redesign is live: app shell + mobile tabs, cover-panel
  library, live search with add-to-category, manga detail with chapter states, self-driving
  downloads (the M2 queue is now live in the binary), consent-gated category cards,
  reader pill + amber progress strip.
- DESIGN.md + .impeccable/design.json record the built world; detect clean; README
  screenshots regenerated from real live-source data.
- Known gaps (recorded, deliberate): shell first-run token prompt (findings 2026-11-24);
  .tachibk imports render cover placeholders (no cover URLs in the backup format);
  description not persisted (detail hero shows title/source/counts only).
- Test suite: 54 Rust tests + 9 Playwright specs (library, search, detail, reader,
  categories, shots, core-loop device benchmark), all wave gates green.
