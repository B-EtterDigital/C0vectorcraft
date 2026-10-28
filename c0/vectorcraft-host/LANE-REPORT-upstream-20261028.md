# Lane report: upstream 54c4f82638f0 into c0/main (vectorcraft)

Branch `fix`. Merge `f8acfc68` (second parent is upstream `54c4f82638f0dc55120becb41264b11af73e765d`).
Descends from `c0/main` (`fb69cb8a`) and contains upstream. Merge only, no rebase.
Follow-up commits: `d9b74242` (PATCH-DEBT), `b30f581d` (collapse_dock + Id::unique), `e60f4a98`
(test ids), and this report.

## Conflicted files and how each was resolved

| File | Conflict | Resolution |
|---|---|---|
| `.github/workflows/{ci,freebsd,packaging-lint,print,release,windows-arm64,windows7}.yml` | modify/delete: our fork deleted them (`9fa7a671`), upstream modified them | Kept deleted, never resurrected. Upstream's `affinity-corpus`, `nix`, `sonarcloud` workflows were auto-resolved as deletions. |
| `crates/ui-egui/src/canvas.rs` | content (whole file: upstream split it into `canvas/*.rs`) | Took upstream (`== upstream`). Community-heading guard **moved** to `canvas/home.rs` (re-applied, 3 lines). |
| `crates/ui-egui/src/lib.rs` | content (upstream moved most items to `app.rs`, `app/*.rs`) | Took upstream. Re-applied: `pub mod c0_host;` and `pub c0` field (2 lines, stay in lib.rs — crate root and struct must declare them). `run_now` allow → `app/commands.rs` (1). `tick` call and app-bar guard → `app.rs` (1 + 3). Constructor init → `app.rs` (1). |
| `crates/ui-egui/src/menus.rs` | content (upstream split into `menus/*.rs`) | Took upstream. `shown_state` allow → `menus/state.rs` (3). `home_showing` predicate → `menus/bar.rs` (1). |
| `Cargo.lock` | auto-merged (no conflict) | Kept upstream lock plus our one-line js-sys edge. `cargo metadata --locked` exits 0. No hand edits, no re-resolve needed. |

Auto-merged files whose hunks I read in final position (all correct function): `chrome.rs`, `community.rs`,
`control.rs`, `credits.rs`, `dialogs/about.rs`, `i18n/tests.rs`, `palette.rs`, `prefs_dialog.rs`, `theme.rs`,
`apps/vectorcraft-web/{Cargo.toml,src/main.rs,src/web.rs}`, `crates/doc/{Cargo.toml,src/metadata.rs}`.

"Moved out" was not possible for any seam: each hook is a call inside an upstream function (the Community
heading inside `home_content`, the app-bar call inside the frame, `shown_state`, `home_showing`), so the guard
has to sit at that call site. Everything else of ours already lives in new `c0_*` files.

Dropped with proof: `crates/engine/src/store.rs` (1 line, `bb8557fa`: `#[cfg(not(target_arch = "wasm32"))]`
on `store_profile`). Upstream `54c4f826` carries the same guard (line 207) and the fork's file is identical to
upstream. Proof: `git diff 54c4f826 HEAD -- crates/engine/src/store.rs` is empty, and the wasm clippy run
below passes.

Post-merge chore ("keep upstream trademarks and CI out of c0/main"): **not needed**. `git ls-files docs/brand/
.github/workflows/` gives only `docs/brand/LICENSE-brand.txt`, which the rule excludes. No `include_bytes!`
references brand files. So no commit was made.

## PATCH-DEBT

- Before (ledger): 57, listed against baseline `5b77e422`.
- Measured before (fork vs its fork point `bccf4de7`): 53 added lines in 21 files.
- After (`git diff --numstat --diff-filter=M 54c4f826 HEAD -- . ':!c0'`): **53 added lines in 21 files**.
- Ratchet set to 53. Total did not grow.
- Corrections: removed the `store.rs` row (stale). Removed the "three native-only font-poll guards" from the
  `lib.rs` row, because the diff against `bccf4de7` and `54c4f826` does not contain them; I searched the
  `c0/main` history and could not trace them, so their origin is unconfirmed (see below).

## Tests (all pass)

- `cargo test -p vectorcraft-ui-egui`: **2295 passed, 0 failed** (all test binaries).
- `cargo test -p vectorcraft-web`: **12 passed, 0 failed**.
- No c0 crate under `c0/` exists (c0 holds only scripts and docs), so no separate c0host test run.
- Upstream-red: **none**. Nothing failed, so no test had to be checked against pure upstream.

## Build and lint

- `cargo check -p vectorcraft-ui-egui -p vectorcraft-web --all-targets`: exit 0.
- wasm: `cargo check --target wasm32-unknown-unknown -p vectorcraft-web`: exit 0; also with `--features page-control`: exit 0.
- Release wasm size: **not measured**. The page was not built (no trunk run in this lane), so the 73400320-byte cap was not checked.
- clippy native (`-p vectorcraft-ui-egui -p vectorcraft-web -p vectorcraft-doc --all-targets`): exit 0, no warnings.
- clippy wasm (`-p vectorcraft-web --features page-control`): exit 0. One warning, `derivable_impls` in
  `crates/ui-egui/src/updates_ui.rs:25`. That file is upstream's and we did not touch it, so it is upstream's warning, not ours. Not fixed here.
- Cargo `[patch]` "not used in the crate graph" warnings for `egui_demo_lib`, `egui_inspection`,
  `egui_system_font_provider` come from the workspace manifest, which this lane did not change.

## Fixes made during the lane

- `b30f581d`: upstream removed `dock::set_collapsed`. `fold_dock` now calls `panel_docking::collapse_dock(app,
  Some(folded))` and still restores `app.ui.dock` afterwards. Also `egui::Id::new` → `egui::Id::unique` in
  `c0_files.rs`, `theme/host.rs`. `Id::unique` hashes with fixed seeds (checked in egui source), so keys stay deterministic across frames.
- `e60f4a98`: the two c0 tests now use `egui::Id::unique` to match production (`about_tab`, `toolbar-floating`).

## New upstream commands / menu items (for the Core Agent verb mapping)

Set difference of quoted dotted ids in `crates/**` at `54c4f826` minus `bccf4de7`: **70 new ids**.

`artboard.deleteEmpty`, `brush.addFromLibrary`, `brush.libraries`, `brush.library`, `brush.loadLibrary`,
`brush.saveLibrary`, `help.checkForUpdates`, `imageTrace.preview`, `object.pathfinder.options`,
`text.convertListToText`, `text.reshapePath`, `text.startList`, `text.thread.break`, `type.convertListToText`,
`view.goto11` … `view.goto25` (15), `window.brushLibrary`, `window.brushLibrary.other`, `window.panel.minimize`,
`window.userBrushLibrary`, `window.userBrushLibrary1` … `window.userBrushLibrary10`.

Changed meaning, not new id: `window.collapseDock {collapsed?}` now collapses every dock column (or expands the
outer one), toggles when `collapsed` is omitted, and returns the new state. Our `fold_dock` calls the new function.

## What upstream added, in plain words

- 364 non-merge commits since `bccf4de7`. Most are **refactors** that split large files into modules (`lib.rs`,
  `canvas.rs`, `menus.rs`, `panels/*`, `widgets.rs`, `dialogs`, engine `lib.rs`). These drive our conflicts.
- **Undo fixes**: a canceled undo scope no longer drops journal entries from open undo groups; canceling an undo
  group keeps a batch's steps in other documents; Delete Empty Artboards is one undo step.
- Image Trace: changes made while Preview is off stay with their object; Document Library traces use the Swatches color group.
- Text: threaded overflow draws the loaded-text cursor; a recovered document keeps continued paragraphs in threaded frames.
- Tabs: tab overflow list scrolls to every open document.
- Menu: "Dropdowns: menus grow back after a search with no matches (#4337)".
- I read commit **subjects only**, not diffs. Upstream does not use `feat:`/`perf:`/`fix:` prefixes consistently, so I
  could not reliably list crash, freeze and save fixes. Treat the above as a partial summary.

## Clashes with our UI

- **Home**: upstream's Home screen moved to `canvas/home.rs`. The Community guard (`community_links`) is re-applied there. The Home button predicate (`menus/bar.rs` `home_showing`) and the app-bar switch (`app.rs`) are re-applied.
- **Menus**: menus split into `menus/{bar,state,run,…}.rs`. Our hidden-command filter is in `menus/state.rs`. Re-check the native menu tests.
- **Panels/dock**: `window.collapseDock` semantics changed (column-based, toggling, transactional when docking is on). Our `fold_dock` now goes through it. The host path where `ui.docking` is `Some` applies per-column `SetIconic`, which our legacy path did not; not exercised beyond the existing tool-mode tests.
- **Theme**: `theme.rs` overlay lines auto-merged. Compile and tests pass, but I did not verify the overlay against upstream's theme changes visually.
- **Community links / About / credits**: auto-merged guards pass tests. Not checked against upstream's new About text.
- **Agent server / control**: `control_server.rs` tests moved to `control_server/tests.rs`. Our `c0.` dispatch in `control.rs` is intact and compiles.

## Assumptions I could not confirm

1. The font-poll guard lines in the old `lib.rs` ledger row: could not trace in `c0/main` history; removed from the ledger as not present in the diff.
2. Whether upstream's `collapse_dock` transactional path (docking enabled) behaves identically to our previous `set_collapsed` for the host's fold command. Covered only by `c0_tool_modes`, which passes.
3. The release wasm size against the 73400320-byte cap (page not built).
4. Whether upstream theme changes interact with `theme/host.rs` overlay beyond what the tests cover.
5. The PATCH-DEBT measurement uses `main` = upstream `54c4f826`; the ledger's old baseline `5b77e422` gave 57 with the phantom lines above.
6. `cargo check`/`test` ran under `smoac-slot run gate` with the host's cap (4). The skill named in the brief (`sow-upstream-pull`) was not available here; rules were taken from the brief.
