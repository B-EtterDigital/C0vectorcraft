# C0vector guest-page lane report

Date: 2026-10-22. Branch: `c0/main` (never pushed). Upstream `main`: `5b77e422` (unchanged).
No pushes, external PRs, control-plane changes or telemetry writes.

## Commits

- `38c3849f` feat(c0vector): browser clock for export dates and wasm dead-code guards
- `56b0b028` feat(c0vector): host control, chrome switches and runtime theme tokens
- `fca6dfae` feat(c0vector): VectorCraft guest page and page-control bridge
- this report, committed on top (see `git log`).

## Implementation


- The existing `vectorcraft-web` binary, built by Trunk with `page-control`, a transparent page
  and relative hashed assets. No initial document/demo/sample.
- JSON envelope over `window.vectorcraft.request`, its control queue, and a synchronous unpainted
  WebRunner pass. No duplicate browser API or queue.
- All seven c0 methods, bounded file input, normal Close / Save Changes active-tab replacement,
  read-only native/SVG/PDF snapshots, complete inspection and artboard/layer navigation.
- Default-true session Chrome switches, all requested panel flags, promotion filtering in
  menus/palette/Home/header/About, retained contributor names and licence.
- All 38 scalar Color32 tokens, context-local overrides, null clearing and brightness persistence.
- Browser Unix clock fix for export dates, marked ready to offer upstream as-is.

## Upstream files edited (added lines; Ratchet 57 of 70)

Cargo.lock 1, apps/vectorcraft-web/Cargo.toml 4, src/main.rs 2, src/web.rs 4, crates/doc/Cargo.toml 3,
doc/src/metadata.rs 3, engine/src/store.rs 1, ui-egui: canvas.rs 3, chrome.rs 2, community.rs 3, control.rs 2,
credits.rs 3, dialogs/about.rs 5, i18n/tests.rs 1, lib.rs 11, menus.rs 4, palette.rs 1, prefs_dialog.rs 1, theme.rs 3.
Measured with `git diff --numstat --diff-filter=M main c0/main -- . ':!c0'`: 57 added, 19 files.
Reasons per file: [PATCH-DEBT.md](PATCH-DEBT.md). `i18n/tests.rs` was added this session: the
status-message scan flagged the host's English control-reply strings (17 untranslated); `c0_*` files are now
skipped like `control.rs`.

## Test results

| Gate | Result |
|---|---|
| `cargo test -p vectorcraft-web --features page-control` | 10 passed, 0 failed |
| `cargo test --no-fail-fast -p vectorcraft-ui-egui` | 1815 passed, 0 failed (lib 1790 after the i18n fix, which also passed alone: 61 i18n tests); includes c0_chrome 5, c0_host_contract 14, c0_theme 4, c0_untrusted 2 |
| `cargo check --target wasm32-unknown-unknown -p vectorcraft-web --features page-control` | ok, no warnings |
| clippy `-D warnings` (ui-egui, web, doc, engine; all targets; page-control) | clean; wasm clippy clean (earlier run, same sources apart from the test-only i18n line) |
| `cargo fmt --all --check` | clean |
| engine library suite | 2,660 passed, 2 ignored (first executor's run; not re-run here, engine source only changed by a cfg guard) |
| `cargo xtask ci` | NOT confirmed: the first executor's log `/tmp/c0vector-gates/ci.log` stops mid-run with no verdict, and I did not re-run it (time/disk). |
| Chromium smoke (`c0vector-gates/smoke.cjs`, swiftshader, `?webgl`) | pass: 28 c0.* replies, 0 page errors, starts with 0 documents and transparent body, real SVG opens (width 72), native round trip, unknown keys reported, frozen-frame calls answer; screenshot read, app renders |

## Page size

`dist/c0host`: wasm 68,273,354 bytes (23.8 MB gzip -9), js 178,068, index.html 1,600; total 68,453,022 bytes (66 MB).
Code section 50.9 MB, data 17.2 MB, no debug sections. The docs claim about 17.5 MB; this is 4x that. I did
not build upstream's own page to compare (18 min, disk), so I cannot say whether our patch contributes.

## Load average (`/proc/loadavg`)

- before page build: `11.97 13.56 9.76 7/2094 1075333`
- after page build (18m05s wall, `smoac-slot run heavy`, CARGO_BUILD_JOBS=2): `10.26 15.14 15.00 12/2206 2071500`

## Assumptions I could not confirm

- Full `cargo xtask ci` verdict (see above); upstream suites other than engine/ui-egui/web were not re-run.
- Built without `CRAFT_FONTS_DIR` (no `../craft-fonts`): the web-fonts hook is a no-op, so only the bundled Latin fonts are present.
- Wasm size versus upstream's own build (unmeasured).
- The smoke driver lives in `/tmp/c0vector-gates/smoke.cjs` (outside the repo); `/tmp/c0vector-browser`'s playwright copy is broken, I used `/tmp/c0light-browser/node_modules/playwright`.
- WebGPU not exercised, only WebGL on swiftshader. `dist/` and `target/` are gitignored and left in place (target is shared, not cleaned).
