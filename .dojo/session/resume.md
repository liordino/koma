# Resume checkpoint — 8a done, 8b blocked on owner (2026-11-13)

- Branch plan/koma-slice-0, clean tree. Session: wave 8, step HALT.
- 8b needs owner: on the Pi Zero 2 W (co-tenants running), build the release binary per
  benchmarks/baseline.md §Cross-build, set KOMA_BENCH_TOKEN, run benchmarks/run-benchmarks.sh,
  paste JSONL into baseline.md. Then: BLOB-vs-fs decision (12) + Suwayomi baseline capture.
- After 8b: slice-0 complete → merge decision for plan branch.
- Toolchain: zig 0.14.1 at C:/tools/zig; wrapper ccz.cmd; recipe in .dojo/findings.md.
