# Benchmarks — Koma vs Suwayomi baseline on the pinned device

Device: **Raspberry Pi Zero 2 W** (quad-core Cortex-A53, 512 MB) — `pi` SSH host
Co-tenants during *every* measurement (decision 28): **FreeLLMApi, MicroBin, Miniflux** (as running)
Build target: `aarch64-unknown-linux-musl` — fully static, stripped, 5.1 MB binary
Harness: `benchmarks/run-benchmarks.sh` (JSONL); raw runs in `benchmarks/raw/`

## Measured (wave 8b, 2026-11-18 — 3 runs, medians)

| Metric | Koma (median; runs) | Target / gate | Verdict |
|---|---|---|---|
| Idle RSS | **4.3 MB** (4304 / 4308 / 4324 KB) | < 100 MB | PASS — 23× headroom |
| Cold start | **768 ms** genuine cold (root `drop_caches`; warm-restart runs 451 / 612 ms) | < 2 s | PASS |
| Page p95 (localhost, 20 pulls) | **63 ms** (62 / 63 / 65) | < 50–200 ms | PASS (within envelope; LAN adds RTT only) |

Cold-start caveats kept honest: runs 1–2 restarted without cache drop (no root); run 3 held root
for a true drop. The 768 ms is the number to quote.

## Suwayomi comparative baseline — deferred, reason recorded

No JVM on the device; a JVM plus Suwayomi-Server alongside the three co-tenants on 190 MB
available RAM risks OOM-skewing the comparisons it exists to make. Recorded as deferred for a
calmer moment or a RAM-headroom plan, owner's pace. The Koma absolute gates above are met
without it; the *comparative* claim stays unproven until this exists.

## Storage decision from this measurement (decision 12, .dojo/CONTEXT.md)

Filesystem-served pages measure 62–65 ms p95 on the pinned device — comfortably inside the
50–200 ms envelope. Decision 12's condition ("BLOBs only on a measured win") is therefore NOT
met: filesystem bytes + DB-as-index stand, and compressed page BLOBs are rejected as unneeded
complexity. Revisit only if a measured regression appears.

## Raw results

Verbatim JSONL from each of the three 2026-11-18 runs lives in `benchmarks/raw/`
(results-20261118T164816Z / T165000Z / T165339Z). Cross-build recipe: `.dojo/findings.md`
(zig 0.14.1 + ccz.cmd filter + zig ar; `link-self-contained=no` so zig owns CRT/libc).
