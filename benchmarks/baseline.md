# Benchmarks — Koma vs Suwayomi baseline on the pinned device

Device: **Raspberry Pi Zero 2 W** (quad-core Cortex-A53, 512 MB)
Co-tenants during *every* measurement (decision 28): **FreeLLMApi, MicroBin, Miniflux**
Build target: `aarch64-unknown-linux-musl` (fully static; `armv7-unknown-linux-musleabihf` fallback)
Harness: `benchmarks/run-benchmarks.sh` (JSONL results)

> Wave 8b: the numbers below are placeholders until run on the device. A metric with
> `PENDING` is not measured; do not treat this file as evidence until 8b is real
> (decision 16: committed harness output does not count until the device ran it).

## Metrics

| Metric | Koma | Suwayomi baseline | Target / gate |
|---|---|---|---|
| Idle RSS | PENDING | PENDING | < 100 MB |
| Cold start (drop caches → first 200) | PENDING | PENDING | < 2 s |
| Page p95 (localhost, 20 pulls) | PENDING | PENDING | < 50–200 ms LAN |
| Core loop: cold→find→add→read 10 pages→close→resume | PENDING | — | ≤ 90 s |

## Procedure

1. On the device, with all co-tenants running as usual: capture Suwayomi baseline first
   (see below), then build `./koma-server` for aarch64 (see §Cross-build) and run the harness.
2. Run the harness **3 times**, record the median.
3. Paste each harness JSONL (verbatim) into §Raw results and transcribe medians into the table.

## Cross-build (from any machine)

```bash
rustup target add aarch64-unknown-linux-musl
# cc/ar via zig (see proven working command shape in .dojo/progress.md — ccz wrapper):
CC_aarch64_unknown_linux_musl="ZIG_CC_WRAPPER_PATH" AR_aarch64_unknown_linux_musl="zig ar" \
CCZ_TARGET=aarch64-linux-musl cargo build --release --target aarch64-unknown-linux-musl -p koma-server
scp target/aarch64-unknown-linux-musl/release/koma-server user_at_zero2w:
```

Windows note (found in the spike): cc-rs emits a rust-flavored `--target=` that zig 0.14 cannot
parse; the wrapper filters it (kept in tools, not the repo — see .dojo/findings.md).

## Suwayomi baseline capture

Run Suwayomi-Server (current release) on the same device with the same co-tenants; record
idle RSS (VmRSS), cold start (drop caches → first successful web call), and page p95 with
`hyperfine`-style repetition. Record below before any Koma run — the baseline is what makes
the Koma numbers meaningful (BRIEF.md success criteria are comparative).

## Raw results

(appended verbatim JSONL from each harness run; wave 8b)
