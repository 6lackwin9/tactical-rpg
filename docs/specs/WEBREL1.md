# WEBREL1: Web (WASM) build of the game, deployed to itch.io

Status: `.1` design (user answers 2026-11-09); awaiting the Build Session `feasibility-check` (`docs/READY.md` Section 2)
Depends on: none (R1-R4 shipped; Post-R4 track, `games/emberwake/docs/MILESTONES.md`)
Layers: engine (`aegis_ui`, `aegis_save`, `aegis_core`: a web platform seam), game (`emberwake_gui`: the web entry and its bundle), platform (tools and CI)
Touches: `engine/crates/{aegis_ui,aegis_save,aegis_core}`, `games/emberwake/crates/emberwake_gui`, `tools/`, `.github/workflows/`, `docs/workflow/web.md` (new)

## Goal

Ship the game to a browser. The **target is the base game** (the `emberwake` binary's main menu: New game, Continue / Load, Settings) as a WebAssembly build on a password-protected itch.io page. **For testing, the build may keep the Scene Runner** (Tools: Combat Arena, Forges, Dojo, UI Gallery) reachable, so web breakage is found before the player-facing UI is finished (user 2026-11-09).

This replaces the Godot web export retired on 2026-10-19 (`docs/HISTORY.md`: "none yet (a WebAssembly build of the engine would be a new task)") and revives the Godot-era hosting decision ("Web build (user 2026-10-17)") on the Aegis engine.

## User answers (2026-11-09)

| Question | Answer |
| --- | --- |
| Q1 Who opens the itch.io page | **Restricted**: password-protected page, as in "Web build (user 2026-10-17)"; going public later is one itch.io setting |
| Q2 When it builds and deploys | **CI build, manual deploy**: every `develop -> main` release PR builds the web bundle (build only, artifact kept); upload to itch.io only when the user says "deploy", until the user calls the build playable; then after each release (the paused 2026-10-17 "Deploy after release" rule comes back on the user's word) |
| Q3 What the first build contains | **Base game is the target; the Scene Runner may be used for testing** (the dev tools stay in test builds and are compiled out of the release build, section 4) |

## 1. What already holds, and what blocks a WASM target

The engine is pure Rust on `winit 0.30` + `pixels 0.15` (wgpu) + `fontdue` + `rodio 0.20` + `arboard 3.7` + `png`; rules, sim and content code are already platform-free and deterministic (`aegis_core` RNG, ordered maps). The blockers are all at the platform edge. Measured on `develop @ab2f2e0f` by grep (non-test `src/` files; the feasibility check confirms each site):

| Edge | Today | Count / sites | Web plan |
| --- | --- | --- | --- |
| File reads | `std::fs` on `data/`, `assets/`, fonts (`CARGO_MANIFEST_DIR`, `current_exe` in `emberwake_gui/src/main.rs`) | ~40 files | One read seam (section 2.1): native keeps `std::fs`; web reads a preloaded bundle |
| User files | `aegis_save::user_dir` = `%APPDATA%/Aegis/<game id>` (saves, settings, recordings) | `aegis_save/src/lib.rs` | Web store on `localStorage` behind the same API (section 2.2) |
| Clock | `std::time::Instant::now` / `SystemTime` (panics on `wasm32-unknown-unknown`) | ~24 files, mostly `aegis_ui` (`shell`, `audio`, `tuning`, `vignette`, `ground`, `playtest`) | One clock seam (`web-time` or an engine wrapper) |
| Threads / TCP | agent link `aegis_ui/src/bridge.rs`, `shell/headless.rs`, `aegis_cli/src/scenario.rs` | 4 files | Not built on web (`cfg`); the web build is single-threaded, so itch.io needs no `SharedArrayBuffer` / COOP-COEP headers |
| Processes / env | `std::process`, `std::env` (CLI args, `APPDATA`, tools) | ~9 / ~14 files | Web reads its options from the page URL query (`?screen=arena&seed=3`), same keys as the CLI |
| Window loop | `winit` desktop event loop, Windows-only work-area query (`windows-sys`, DISPLAY2) | `aegis_ui/src/shell.rs` | winit web backend on a `<canvas>`; the 1280 x 720 canvas (LAYOUT720) scales to the browser window by device pixel ratio |
| Renderer | `pixels` synchronous init | `shell.rs` | `pixels` async init on wasm (wgpu WebGL2 backend; WebGPU when available) |
| Audio | `rodio` on the desktop device | `aegis_ui/src/audio.rs` | `rodio`/`cpal` web-audio backend, started on the first click or key (browser autoplay rule); a silent stub is an acceptable first step |
| Clipboard | `arboard` (PLAYUI1-T text editor) | `aegis_ui` | Native only; on web the editor keeps an in-app clipboard |
| Screenshots, playtest recordings | F12 to disk, recordings in the user folder | `aegis_ui` | Off on web in the release build; test builds may keep recordings in the web store |

Bundle size today: `games/emberwake/assets` 22 MB, `kits/rpg/assets` 7.9 MB, `games/emberwake/data` 1.1 MB (about 31 MB before compression).

## 2. Engine: the web platform seam (layer engine)

The engine stays game-agnostic: nothing in `engine/` names `emberwake`; the game passes its id, bundle name and entry screen.

### 2.1 Reads

- One engine read API (for example `aegis_core::vfs` or `aegis_ui::files`) that every runtime asset, data and font read goes through. Native: `std::fs`, unchanged behaviour and paths. Web: an in-memory table filled before the first frame from **one fetched bundle file** (a pack or zip of the game's `data/`, `assets/` and the kit assets the game uses), with a loading bar.
- The bundle is fetched, not compiled into the `.wasm` (`include_bytes!` on 31 MB would bloat the module and every rebuild). One bundle file keeps the upload far under itch.io's per-project file-count cap.
- Test-only reads (`CARGO_MANIFEST_DIR` fonts in unit tests) stay as they are; the seam covers the runtime paths only.

### 2.2 User store

- `aegis_save` keeps its API (`user_dir`, slots, settings, backups). Web backend: `localStorage` keys `aegis/<game id>/<relative path>`; a write that exceeds the browser quota returns the existing error type (no panic) and the UI shows "save failed".
- The save format and its byte-for-byte resave golden do not change; web saves are the same bytes.

### 2.3 Clock, loop, renderer, audio, input

- One clock type used by the engine (wall-clock reads only through it).
- `Shell` gains a web start path (winit `EventLoopExtWebSys` / `spawn_app`, canvas lookup by id, async `pixels` init) beside the native `run`; frame pacing on `requestAnimationFrame`.
- Audio starts on the first user gesture; until then sounds are dropped, not queued.
- Keyboard and mouse map to the same `Key` / pointer events; right click (go back) needs the canvas context menu suppressed.
- Dev keys (F6 map zoom, F7 view, F9 window size, F12 screenshot) are debug-build or dev-tools only, as on desktop.

### 2.4 Determinism

Rules, sim and RNG code do not branch on target. No golden is re-recorded by WEBREL1; any golden diff on native is a bug in the seam.

## 3. Game: the web entry and bundle (layer game)

- `emberwake_gui` builds for `wasm32-unknown-unknown`: a web entry (`wasm-bindgen` start function) that reads the URL query (same keys as the CLI: `screen=menu|arena|runner`, `encounter=`, `ruleset=`, `seed=`), starts the shell on the page canvas, and opens the main menu by default.
- Not built on web: the agent link (`port=`), `headless=`, `data=` / `saves=` folder options, the Windows `.exe` icon (`build.rs`, `res/`).
- The bundle list (which `data/` and asset folders ship) lives in a game-owned manifest the build tool reads; the kit assets the game uses are included by that list.
- One `index.html` + JS glue + `.wasm` + bundle, served from a single folder (itch.io HTML5 zip, or `localhost` for the user's PC).

## 4. Test build vs release build

| Build | Who | Contents |
| --- | --- | --- |
| **Test** (`dev_tools` on) | CI artifact on every release PR; manual "deploy" to the restricted itch.io page until the user calls the game playable | Base game plus Tools (Scene Runner: Combat Arena, Content and World Forge viewers, Gear Dojo, UI Gallery); playtest recordings to the web store allowed |
| **Release** (`dev_tools` off) | The default deploy once the user calls the build playable (Post-R4 game UI polish) | Base game only: Tools menu and dev keys compiled out; same bundle |

`dev_tools` is a cargo feature on `emberwake_gui` (default on for native, so desktop behaviour does not change). The native desktop build is unaffected by this split.

## 5. Platform: build script, CI, deploy (layer platform)

- `tools/fc.sh web [test|release]` (or `tools/web_build.sh`): `cargo build --target wasm32-unknown-unknown --release -p emberwake_gui`, `wasm-bindgen` (CLI version pinned to the crate version), optional `wasm-opt`, pack the bundle, write `out/web/` (gitignored). `tools/fc.sh web serve` serves it on `localhost` for the user's PC.
- CI: `gate.yml` gains a `web` job on PRs into `main` (the release PR): build only, upload `web-build` as an artifact. It is **not** a required check at first (advisory until it has been green on two releases; then the user decides whether `main` requires it).
- Deploy: a new `web.yml`, `workflow_dispatch` only, on `main`: builds the chosen variant and uploads with itch.io's `butler` to the restricted page. Needs a repo secret `BUTLER_API_KEY` and a repo variable `ITCH_TARGET` (`<user>/<game>:html5`), both set by the user. The Merge & Release thread runs it when the user says "deploy".
- Smoke test (CI and local): headless Chromium (the pre-installed Playwright) loads the test build, waits for the main menu, and (a) opens Tools > Combat Arena and plays scripted turns of `goblin_skirmish` to the results screen; (b) New game reaches the first region screen; (c) save, reload the page, Continue restores the slot.
- `docs/workflow/web.md` (Design writes it from Build's docs delta): build, serve, deploy, the itch.io settings (HTML5, viewport 1280 x 720, fullscreen button, password).

## 6. Slices (queued after the feasibility check)

| Slice | Layer | Scope | Acceptance |
| --- | --- | --- | --- |
| `WEBREL1-E1` | engine | Read seam (2.1), user store (2.2), clock seam, `cfg` out threads/TCP/process/clipboard on wasm | `cargo build --target wasm32-unknown-unknown -p aegis_core -p aegis_save -p aegis_ui` compiles with no warnings; native `tools/fc.sh gate` green with **0 golden diffs**; unit tests for the web store's key mapping and quota error (native-testable through the store trait) |
| `WEBREL1-E2` | engine | Shell web start path, async renderer init, DPR scaling, audio on first gesture | A minimal engine example (no game) renders a frame and takes input in headless Chromium |
| `WEBREL1-G1` | game | `emberwake_gui` web entry, URL query options, bundle manifest, `dev_tools` feature | Test build boots to the main menu in headless Chromium; smoke test (a)-(c) passes; native gate green, 0 golden diffs |
| `WEBREL1-P1` | platform | `tools/fc.sh web`, `web` CI job (advisory), `web.yml` deploy with `butler` | CI job uploads `web-build` on a release PR; `web.yml` dry run (no secret) stops with a clear "BUTLER_API_KEY not set" message; with the secret, a manual run uploads to the restricted page |

## 7. Feasibility check (Build Session, read-only)

Report in `docs/FEEDBACK.md` as one `FB-nn` (`Kind: feasibility`):

1. Confirm or correct every row of section 1's table with file:line sites, and list any blocker the table misses (for example a crate in the `emberwake_gui` dependency tree that does not compile for `wasm32-unknown-unknown`).
2. `pixels 0.15` / `winit 0.30` / `rodio 0.20` web support at these versions: what each needs (features, async init), and whether a version bump is needed (a bump that moves a golden is a `limit-blocker`).
3. The read seam: how many runtime read sites, and whether one API can cover them without touching rules/sim code.
4. Bundle: packed size (raw and gzip/brotli), first-load time estimate, and whether any asset folder can be left out of the web bundle.
5. Save sizes in `localStorage` (largest slot today vs a 5 MB quota).
6. Cost estimate per slice and a recommended order; flag any slice that cannot keep **0 golden diffs**.

## 8. Out of scope

Mobile/touch controls; WebGPU-only features; offline/PWA install; save export/import to a file (a later UI task); a public page (Q1: one setting later); Steam or desktop installers.
