# SOW consistency: VectorCraft tool modes and foldable right dock

Date: 2026-10-23. Branch `c0/main` (never pushed). Upstream `main` unchanged.

## Tool-mode decision

VectorCraft already has everything PhotoCraft needed a new module for, so the modes are
Illustrator's Tools panel states, expressed as two booleans on the app's own state:

| Mode | `toolbar_double` | `toolbar_floating` |
|---|---|---|
| docked, one column | false | false |
| docked, two columns | true | false |
| floating, one column | false | true |
| floating, two columns | true | true |

- `toolbar_double` = `UiState.toolbar_double` (the double arrow at the top of the panel, `window.toolbarColumns`).
- `toolbar_floating` = `UiState.toolbar_pos.is_some()`: the app's own floating palette (title bar, grip,
  drag to move, drag to the left edge to dock). Turning it on keeps a position the user already dragged
  to, else uses the app's own default corner (60, 120); turning it off docks it.
- `toolbar` (existing) still shows or hides the panel in every mode.
- `right_dock_collapsed` = `UiState.dock_collapsed`, set through the app's `dock::set_collapsed`, with the `dock`
  switch restored so the two keys do not depend on their order. Folded, the group becomes the icon
  column; each icon still pops its panel out; unfolding while one is popped out shows that tab.
  The change is immediate (the app has no animation), only the panel layout resizes the canvas area.
- All drawn by the app's own toolbar and dock code. No drawing code of ours.

## Commits

- `267189d` feat(c0vector): tool modes and a foldable right dock in c0.set panels
- this report, committed on top (see `git log`).

Changed: `crates/ui-egui/src/c0_host/c0_panels.rs` (ours), `crates/ui-egui/tests/c0_tool_modes.rs` (new),
README (c0.set keys), PATCH-DEBT.md. **Upstream lines edited: 0. Ratchet stays 57 of 70.** (The local
shallow clone has no `main` ref, so the numstat command cannot run here; the commit's file list shows no
upstream file.)

## Tests

| Gate | Result |
|---|---|
| `c0_tool_modes.rs` (new) | 8 passed: defaults equal upstream, set/read-back of all four modes, kept position, rendered palette per mode (wider with two columns), session-only (prefs unchanged), dock fold/unfold and `dock` independence, icon panel on unfold, unknown/invalid keys reported |
| c0 host suites (c0_chrome 5, c0_host_contract 14, c0_theme 4, c0_untrusted 2) | 25 passed (c0_tool_modes run in the earlier run too) |
| `cargo test -p vectorcraft-web --features page-control` | 10 passed |
| `cargo xtask ci` (fmt, clippy, full tests, layers, assets, wasm, ...) | all 10 steps passed; 8,699 tests passed, 0 failed |
| wasm check, clippy (native and wasm32, `-D warnings`) | clean |
| Page build (trunk, c0/vectorcraft-host) | ok |
| Headless Chromium smoke (swiftshader, `?webgl`) | pass: defaults read false; all four modes set and read back with nothing unknown; fold and unfold read back; unknown key reported; 0 page errors. Screenshots (floating two columns; dock folded) read and correct |

## Wasm size

`vectorcraft-web_bg.wasm` 68,274,303 bytes (23.8 MB gzip -9); previous lane 68,273,354 (+949 bytes).
I found no pin cap in the repo, so I cannot compare against it (see assumptions).

## Load average (`/proc/loadavg`)

- before the lane's builds: `24.77 17.26 16.28 7/2019 1095641`
- before the page build: `13.50 10.37 12.60 9/1600 1118325`
- after the page build: `14.66 12.16 12.08 9/1824 1427548`

## Assumptions I could not confirm

- "The pin cap" for wasm size: no cap is defined in the repo, the docs or the prior report, so I recorded the size only.
- Not saved to prefs: the keys write app UI state (`toolbar_double`, `toolbar_pos`, `dock_collapsed`), which
  upstream saves with workspaces, as the existing `toolbar` and `dock` keys already do. They write nothing to
  preferences (tested). Whether the web page persists UI state across reloads I did not check.
- No layout-jump measurement of the canvas content on fold; the canvas simply gets the freed width.
- `FLOAT_AT` copies upstream's private `floating::TOOLS_AT`; it can drift on an upstream pull (listed in PATCH-DEBT).
- WebGPU not exercised; the smoke driver (`/srv/smoac/build/c0vec-smoke/`) lives outside the repo.
- The cargo target directory (13 GB) is left in `sow-consistency-vectorcraft/target`.
