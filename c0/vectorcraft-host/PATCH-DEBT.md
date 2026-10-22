# Patch debt: lines of ours inside upstream files

Baseline: `main` at `5b77e422ab3529b4239abcebedd377c4c706b95a` (VectorCraft 0.14.0).
All commits belong on `c0/main`; this executor never pushes. Counts include replacement and
formatting lines, not just net growth. New files are excluded by `--diff-filter=M`.

| File | Lines | What | Why | Offered upstream |
|---|---:|---|---|---|
| `Cargo.lock` | 1 | js-sys edge for vectorcraft-doc | Lock the wasm clock dependency without upgrading packages | yes, as is |
| `apps/vectorcraft-web/Cargo.toml` | 4 | Feature flag and native serde_json dependency | K-41 native feature tests use the same parser | no, host feature |
| `apps/vectorcraft-web/src/main.rs` | 2 | Feature-gated page_control module | Keep upstream browser build unchanged without the feature | no, host seam |
| `apps/vectorcraft-web/src/web.rs` | 4 | Retain runner; attach wrapper after app construction | Wrap the existing browser API and lend an unpainted pass | no, host seam |
| `crates/doc/Cargo.toml` | 3 | wasm-only js-sys dependency | Browser UTC clock for export metadata | yes, as is |
| `crates/doc/src/metadata.rs` | 3 | Finite, bounded browser Unix time and corrected comment | Wasm exports previously omitted dates | yes, as is |
| `crates/engine/src/store.rs` | 1 | Match profile writer's native-only caller guard | Strict wasm Clippy otherwise fails on unused native code | yes, as is |
| `crates/ui-egui/src/canvas.rs` | 3 | Guard Home's Community heading | No empty promotional section when links are hidden | no, host switch |
| `crates/ui-egui/src/chrome.rs` | 2 | Guard Home button and header Discord | Host owns Home and community navigation | no, host switches |
| `crates/ui-egui/src/community.rs` | 3 | Guard shared link list | Cover Home and About together | no, host switch |
| `crates/ui-egui/src/control.rs` | 2 | Single c0. dispatch arm and descriptor filter | Share the control queue; hidden promotions do not leak into host command palettes | no, host seams |
| `crates/ui-egui/src/credits.rs` | 3 | Render contributor names without profile links | Keep credits while suppressing GitHub navigation | no, host switch |
| `crates/ui-egui/src/dialogs/about.rs` | 5 | Non-promotional attribution text | Retain app, authors and licence while hiding ArtCraft promotion | no, host switch |
| `crates/ui-egui/src/lib.rs` | 11 | Eight host-seam lines; three native-only font-poll guards | Host state/lifecycle/app bar; remove unused native fields from wasm | font-poll guards: yes, as is; host seams: no |
| `crates/ui-egui/src/i18n/tests.rs` | 1 | Skip `c0_*` files in the status-message scan | Host control replies are English protocol data like control.rs, not status-bar text | no, host seam |
| `crates/ui-egui/src/menus.rs` | 4 | Filter hidden commands; guard Home draw decision | Native/in-window menus and Home obey session switches | no, host switches |
| `crates/ui-egui/src/palette.rs` | 1 | Host-aware palette match | Promotional commands also disappear from search | no, host switch |
| `crates/ui-egui/src/prefs_dialog.rs` | 1 | Remember native canvas-colour preference in the host adapter | Host palette reapplication must not erase a saved custom canvas colour | no, host seam |
| `crates/ui-egui/src/theme.rs` | 3 | Module, token-read overlay, visuals overlay | Survive theme reapplication and colour egui widgets consistently | no, host seams |

Total: 57 added lines in 19 modified upstream files.

Ratchet: 57

Measurement before commit uses `git diff --numstat --diff-filter=M main -- . ':!c0'`.
Final committed measurement uses exactly:

```sh
git diff --numstat --diff-filter=M main c0/main -- . ':!c0'
```

## New files: no upstream patch debt

- `c0/vectorcraft-host/`: page, Trunk config, build/verification scripts and delivery docs.
- `apps/vectorcraft-web/src/page_control.rs`, `page_control/{bridge,files}.rs`: parser,
  existing-API wrapper, unpainted runner pass and bounded cancellable chooser.
- `crates/ui-egui/src/c0_host.rs`, `c0_host/c0_*.rs`: host state, inspection/navigation,
  settings and file lifecycle.
- `crates/ui-egui/src/theme/host.rs`: complete context-local Color32 registry/overrides.
- `crates/ui-egui/tests/c0_*.rs`: native contract, chrome, palette, rendering, queue and theme tests.

## Couplings to recheck on upstream pulls

| Our code | Upstream coupling | Verification |
|---|---|---|
| `page_control/bridge.rs` | `window.vectorcraft.request` promises, API reply polling, WebRunner event-listener loan | Native feature tests, wasm feature check, Chromium frozen-frame calls |
| `page_control/files.rs` | Services.open_async/inbox and rfd Blob access | Atomic cancellation test and wasm build |
| `c0_files` | Close owns Save Changes; document uids identify the replaced active document; `read_for_open`/`open_read` split validation from activation | Save/Discard/Cancel, background-tab retention, invalid input retention and snapshot round trips |
| `c0_state` | DocState revision/history, layer hierarchy, View.artboard and navigation commands | Inspect and navigation contract tests |
| `Chrome` | Menu shown_state, palette matching, shared community list, credits profile link, Home predicate | Default behaviour and rendered link-suppression tests |
| `theme/host.rs` | Every scalar Color32 field, token access and egui visuals application | Field registry, clearing and light/dark persistence tests |
| Browser clock | `metadata::now_unix` returns bounded Unix seconds; SOURCE_DATE_EPOCH remains upstream | Native metadata suite and wasm checks/build |

"Offered upstream: yes, as is" marks the standalone wasm bug fix ready to offer. This lane
creates no external PRs and makes no upstream feature-parity or roadmap score changes.
