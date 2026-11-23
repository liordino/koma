# Findings — Koma (discoveries and halt diagnostics; append only)

## 2026-11-13 — Windows→ARM musl cross-build recipe (wave 8a spike)
- cc-rs emits `--target=<rust-triple>` that zig cannot parse ("UnknownOperatingSystem": zig
  triples are arch-os-abi, no vendor slot). musl.cc toolchain binaries are Linux ELF — not
  executable on Windows. zig cc is the only workable local cross-C path.
- Recipe: zig 0.14.1 (0.17's parser rejects the query shape entirely) + cmd wrapper
  `C:/tools/zig/ccz.cmd` that strips any `--target*` arg (batch `for %%a in (%*)` splits on `=`,
  cc passes args quoted → filter must strip on the quote-stripped value, `!a:~0,8!=="--target"`),
  plus `AR_<target>=zig.exe ar`. Env: CC_<target>="C:/tools/zig/ccz.cmd", CCZ_TARGET=<zig triple>.
- Both `cargo check --target aarch64-unknown-linux-musl` and `armv7-unknown-linux-musleabihf`
  green. Release-link step not yet exercised (needs full build; do at 8b).

## 2026-11-19 — Test flake root cause (fixed in wave 15)
- Symptom: full-suite failures (UNIQUE constraint on source.id / "unable to open db") that never
  reproduced solo, across random tests.
- Root cause: Windows pid reuse — test temp dirs named koma-<wave>-{pid}-{serial} collided with
  stale DBs from prior runs when a new process inherited a recycled pid; rwc then opened a stale
  DB with old rows.
- Fix: db::temp_test_dir(prefix) — nanos-timestamped dir (cannot collide); create_dir_all before
  connecting (new subdirs don't auto-exist). All seven suites migrated. 8 consecutive full-run
  passes incl. the wave gate and dojo proof.

## 2026-11-20 — lens false positive: generated Svelte chunk flagged for innerHTML
- docker/web/_app/immutable/chunks/*.js are vite-generated Svelte hydration runtime (build
  artifacts staged by scripts/build-release.sh; docker/ is gitignored — verified via
  git check-ignore). innerHTML there is Svelte's own runtime, not hand-written source;
  recorded to keep the scan history honest.
- Windows handle note: docker/web (old staging location) is held by a watcher process and
  cannot be deleted this session; it is gitignored and superseded by target/docker-context.
