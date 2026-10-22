# C0vector: VectorCraft as a C0VIBE guest page

This page builds the existing `apps/vectorcraft-web` binary with the opt-in `page-control`
feature. It starts with no document, demo or sample. Its application bar, Home and community
links start hidden; editing tools, the dock, document tabs and file commands remain available.
The background is transparent until VectorCraft paints. Each guest view owns its own document
session; no parent bridge, network service or other Craft application is required.

## Build

From this directory run `./build.sh`, or, under an admitted heavy slot:

```sh
CARGO_BUILD_JOBS=2 NO_COLOR=true trunk build --release
```

Trunk writes the finished, hashed site to `dist/c0host/` at the repository root with relative
asset URLs. The upstream `cargo xtask web-fonts` post-build hook is retained. Set
`CRAFT_FONTS_DIR` to an existing absolute craft-fonts directory to copy optional web fonts.
Without it, the app still builds and uses its built-in fallback fonts; optional script coverage
depends on fonts available to the application. The build script unsets a missing directory.
Serve the directory over HTTP(S); `?webgl` selects WebGL2 and `?lang=de` selects the interface
language, as in upstream. No generated assets or fonts are committed here.

## Page function and envelopes

```js
const reply = JSON.parse(await window.__c0hostRequest(JSON.stringify({
  method: "c0.inspect", params: {}
})));
```

`window.__c0hostRequest = (json) => window.__vectorcraftControl(json)` is the page alias.
Wait for `window.__vectorcraftControl` to exist before calling. The bridge wraps the existing
`window.vectorcraft.request(method, params)` API and its control queue. It resolves to JSON
text: `{"ok":true,"result":...}` or `{"ok":false,"error":"..."}`. Malformed JSON, invalid
parameters and unknown methods return error envelopes. Params may be omitted or null (an empty
object); other non-object params fail. Existing `window.vectorcraft` APIs retain their contracts.

Requests are bounded to 1 MiB, except `c0.*` requests, which allow 100 MiB of JSON for a 72 MiB
file encoded as base64 plus envelope overhead (the GridCraft limits). `ui.click.count` is at most 16 and `ui.drag.steps`
at most 32. The app also enforces its own bounded synthetic-input queue.

## When a request is answered

The call synchronously dispatches a private EventTarget event, lending the retained WebRunner
one unpainted egui logic pass. The app drains its existing control queue and settles the existing
API promise in that pass. Host methods need no presented frame or requestAnimationFrame.
Synthetic input gets up to 40 extra logic passes; its reply acknowledges queueing, and widget
input still needs layout passes. `ui.screenshot` deliberately waits for a presented canvas frame.

File replacement can answer `{"pending":"saveChanges"}`. It is then awaiting the normal
Save Changes dialog, not silently accepting a discard. Answer with `ui.dialog.confirm` to save,
or `ui.dialog.set {field:"discard",value:true}` then confirm to discard, or
`ui.dialog.cancel {}` to cancel. Only the active document is replaced; other tabs stay open.
A cancelled or failed save keeps the active document. Poll `c0.inspect.pendingFile` and `fileResult` for completion.
Browser save options/downloads use upstream's asynchronous UI; starting a download alone does
not authorize the host to discard unsaved work. `fileResult` is the latest replacement result,
an error envelope or `{cancelled:true}`. It resets when another replacement begins.

## Host methods

| Method | Params | Result |
|---|---|---|
| `c0.inspect` | `{}` | Mirror described below |
| `c0.goTo` | `{artboard?:index,layer?:id}` | `{artboard,layer}`; at least one target required |
| `c0.documentBytes` | `{format?:"vectorcraft"\|"svg"\|"pdf"}` | `{name,format,bytes,revision,warnings}`; bytes are standard base64 |
| `c0.openFile` | `{name,bytes}` | Upstream `document.open` result or `{pending:"saveChanges"}` |
| `c0.newDocument` | `{template?:presetName}` | Upstream `file.new` result or `{pending:"saveChanges"}` |
| `c0.cancelFileDialog` | `{}` | `{cancelled:boolean}` |
| `c0.set` | `{themeTokens?,panels?}` | `{applied:string[],unknown:string[],invalid:string[]}` |

`c0.inspect` includes `title`, `revision`, `dirty`, `canUndo`, `canRedo`, `panes` (the panel-key
map), `theme:{dark,overrides}`, `artboards:[{index,name,width,height,active}]`,
`layers:[{id,name,visible,locked,depth}]` (top-level layers and one sublayer level),
`selectionCount`, `activeArtboard` (null without a document),
`documents:[{title,dirty,active}]`, `fileDialog`, `pickerPending`, `pendingFile`, `fileResult`.
Revision is the active document's upstream revision, not a cross-document generation counter;
also compare the document list/title when switching documents.

Navigation validates both targets before changing either. It uses `view.goToArtboard` to
select/fit the board, and `layer.setCurrent`/`layer.locate` to highlight/reveal a layer in the
Layers panel. Out-of-range indices and non-layer ids fail rather than being clamped.

Snapshots encode the current document without changing its path, dirty state,
revision, history, Redo, view or active tool draft. Native VectorCraft is the fidelity format;
SVG/PDF use upstream encoders and return approximation warnings. No document is an error.
Snapshotting does not confirm Save Changes or mark the document saved.

Open accepts a nonempty basename of at most 255 UTF-8 bytes, with no path separators or NUL.
It rejects files larger than 72 MiB and validates/decodes the file with the upstream loader
before any document closes. This byte API opens documents; installation/library/import-option
dialogs remain on the ordinary upstream File Open path. For PDF it uses upstream loader defaults;
password-protected files fail instead of opening a password dialog. New accepts an upstream
New Document preset name (for example `A4`); omitted means the default blank document.
Both replace the active document through Close and its unsaved-changes path, preserving other tabs.
They reject concurrent dialogs/replacements. Picker cancellation invalidates late browser
answers; native picker receivers are dropped so late results cannot run their original action.
It cannot dismiss operating-system chooser UI. It also cancels a pending host replacement.

## Chrome settings

`panels` accepts booleans for `app_bar`, `start_screen`, `community_links`, `toolbar`,
`control_bar`, `task_bar`, `dock`, `status_bar`, `rulers`. The three new Chrome switches default
to true in the upstream app, are not serialized, and do not modify saved preferences.
The page sets the first three false. Existing UI flags keep upstream defaults.

`start_screen:false` prevents Home on startup, after closing, and via `app.home`, for this session.
`community_links:false` removes Discord, ArtCraft, getartcraft and project GitHub promotions
from the header, Home, menus and About; contributor names remain, with profile links disabled.
About still names VectorCraft, contributors and its MIT OR Apache-2.0 licence. Direct promotional
commands are refused while the switch is off. Defaults retain upstream links and behaviour.

## Runtime theme tokens

`themeTokens` accepts every scalar Color32 field in `theme::Tokens`:

```text
app_bar panel panel_darker input input_border border divider text text_dim text_disabled
icon hover tool_active accent accent_strong row_selected pasteboard ruler ruler_tick
smart_guide guide measure_bg button text_strong tab_strip button_border selection
anchor_selected flyout_active caption_close caption_close_text highlight bleed error
warning axis_x axis_y axis_z
```

Values are `#RRGGBB` or `#RRGGBBAA`; null clears one override back to the current palette.
Colours apply to custom widgets and egui visuals, persist across brightness/accent/scheme
changes, and are scoped to the guest context. Metrics (`radius`, `font_size`) and `dark` are not
colour overrides. Unknown/invalid keys are reported as `themeTokens.<key>` or `panels.<key>`;
bad section types are reported by section name. Valid keys still apply in a mixed request.

## Ordinary VectorCraft control methods

The page exposes the existing [control protocol](../../docs/control-protocol.md):
`engine.execute`, `engine.commands`, `document.inspect`, `ui.inspect`, `ui.menu.list`,
`ui.contextMenu.list`, `ui.tool.select`, `ui.tool.list`, `ui.pointer`, `ui.key`, `ui.text`,
`ui.move`, `ui.click`, `ui.drag`, `ui.set`, `ui.dialog.set`, `ui.dialog.confirm`,
`ui.dialog.cancel`, `ui.resize`, `ui.focus`, `ui.screenshot`, `ui.render`, `app.open`,
`app.save`, `app.export`, `app.quit`, and engine command ids. Browser hosts should prefer the
byte methods over native filesystem paths. `app.export` without a path returns `dataBase64`.
The host owns page lifetime and browser sizing. Existing API helpers (`execute`, `commands`,
`inspect`, `screenshot`, `openFile`, `captureDownloads`, `files`, `readFile`) remain available;
use `c0.openFile` for replacement with unsaved-change protection rather than the additive API.

## Branches, tests and upstream pulls

Only `c0/main` receives commits; never push from this executor. All our implementation is in
new `c0_*`/`page_control*` files. [PATCH-DEBT.md](PATCH-DEBT.md) lists upstream seams and the
measured added-line ratchet. No upstream feature-parity score changes.

Run Cargo under `smoac-slot run heavy 2 3600 -- env CARGO_BUILD_JOBS=2` (fallback to gate only
if heavy admission is refused):

```sh
cargo test -p vectorcraft-web --features page-control
cargo test -p vectorcraft-ui-egui --test c0_host_contract --test c0_chrome --test c0_theme --test c0_untrusted
cargo check --target wasm32-unknown-unknown -p vectorcraft-web --features page-control
cargo clippy -p vectorcraft-ui-egui -p vectorcraft-web -p vectorcraft-doc --all-targets --features page-control -- -D warnings
cargo xtask ci
```

`smoac-slot run heavy 2 3600 -- c0/vectorcraft-host/verify.sh` runs the full sequence, including
the page build, in one slot and writes separate stage logs under `/tmp/c0vector-gates/` (override
with `C0VECTOR_LOG_DIR`). All Cargo invocations share the lane's own target directory.
To run a local Playwright driver under the same admission, place it at
`$C0VECTOR_LOG_DIR/smoke.cjs` or set `C0VECTOR_SMOKE_SCRIPT`. The script starts an HTTP server
on an allocated loopback port and passes the driver `C0VECTOR_SMOKE_URL` and
`C0VECTOR_SMOKE_SCREENSHOT`. Browser tooling stays outside the Rust-only repository.

[LANE-REPORT.md](LANE-REPORT.md) records the final gates, browser smoke, artifact sizes and
unconfirmed assumptions. Recheck the API/runner, loader, Save Changes, theme and menu seams on
every upstream pull.
