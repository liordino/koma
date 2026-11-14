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
