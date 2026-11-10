# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to R2 rows (R1 release, R1.1, R2 Waves 1-2, R2 release) | released (`main` @98ab973a, @3cc73f2b) | `main` | Rows moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10, size). |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (`3353e62b`) | `develop` | - |
| R4 release | released: `main` @32ae1ada ([#58](https://github.com/6lackwin9/tactical-rpg/pull/58), merge commit; tree = develop @ad903f92, which adds a whitespace fix for an MD012 lint error from 92a7282b) | `main` | Ready queue empty; next: Section 3 `R3-STAGE-SEED`, `POST-R4-CLEANUP` when Design queues them. |
| WEBREL1 feasibility | done: FB-147 (read-only audit, no code) | - | Waiting on Design: E1 split (Option A) and slice order. |
| PR4-1 `R3-STAGE-SIM` | done: part 1 (`stage=N`, `55ef14d8`), part 2 (AUDIT-07, `sim_golden` run 11 re-recorded, FB-154) | `develop` | - |
| PR4-2 `POST-R4-CLEANUP` | done: phase A (`feadda4d`), phase B re-freeze (`c49194b2`, FB-152; FB-153 suggestion); item 2 stays held | `develop` | - |
| PR4-3 `WEBREL1-E1a` | done (FB-155): wasm32 check clean, `aegis_core::clock`, clock lint | `develop` | - |
| PR4-4 `WEBREL1-E1b` | done (FB-156; FB-157 suggestion): `aegis_core::vfs`, `aegis_save::store`, wasm `cfg`, fs lint | `develop` | - |
| PR4-8 `SKILLCHECK-FOLD` | done (FB-158 suggestion): fold error when nothing else is reported; ranks normalised before the fold | `develop` | - |
| PR4-5 `WEBREL1-E2` | done (FB-159; FB-160 suggestion): `shell::start_web`, async WebGL2 renderer, DPR, audio on first gesture; `web_min` passes in headless Chromium | `develop` | - |
| PR4-6 `WEBREL1-G1` | done (FB-161; FB-162 suggestion): the game runs in a browser (web entry, bundle, `dev_tools`, `fc.sh web`, smoke 40/40, WEBREL1 6.2) | `develop` | - |
| PR4-7 `WEBREL1-P1` | done (FB-163; FB-164 suggestion): advisory `web` CI job, `web.yml` itch.io deploy; GitHub-side acceptance observable at the next release | `develop` | - |
| PR4-9 `WEBOPT1` | parked (FB-165 limit-blocker): the `apt` Binaryen 108 breaks the optimised build; work kept on the branch @`6df0d0e4` | `claude/optimistic-newton-ewkyqt` | Waiting on Design (FB-165). |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-164 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-165. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

## Test build and release build

| Build | Command | Contents | Used for |
| --- | --- | --- | --- |
| Test (`dev_tools` on) | `tools/fc.sh web test` (the default) | Base game plus the Tools menu (Scene Runner, Combat Arena, Forge viewers, Gear Dojo, UI Gallery) and the page probe the smoke test reads | The CI artifact of every release PR; the itch.io deploy until the user calls the game playable |
| Release (`dev_tools` off) | `tools/fc.sh web release` | Base game only: Tools and dev keys compiled out | The deploy once the user calls the game playable |

`dev_tools` is a cargo feature of `emberwake_gui`, on by default, so the native desktop build does not change.

## Build it (Linux, the cloud, or Git Bash on Windows)

One-time setup:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked   # must equal the wasm-bindgen in Cargo.lock
```

Then `tools/fc.sh web test` (or `release`). It builds `emberwake_gui` for `wasm32-unknown-unknown` into its own
`target-wasm/` (so it never evicts the native build), binds it with `wasm-bindgen`, runs `wasm-opt` when Binaryen is
installed (optional), packs `games/emberwake/web_bundle.json` with `aegis bundle` (part 1: data and art, fetched before
the main menu; part 2: audio, fetched while the game runs) and writes `out/web/` (git-ignored): `index.html`,
`emberwake.js`, `emberwake_bg.wasm`, `bundle.json`, `emberwake_1.pak`, `emberwake_2.pak` and `build.json` (variant and
commit). It prints each file's size raw and gzipped.

## Play it locally

- `tools/fc.sh web serve [port]` serves `out/web/` on `http://127.0.0.1:8000/` (Python's `http.server`). Opening
  `index.html` as a file does not work.
- Launch keys go in the URL query, for example `?screen=runner` (the Scene Runner, test build) or `?screen=arena&seed=3`.
- A downloaded `web-build` artifact: unzip it and serve the folder with `python -m http.server 8000`, then open
  `http://localhost:8000`.
- Other devices on the same Wi-Fi: `python -m http.server 8000 --bind 0.0.0.0` in the folder and open
  `http://<PC's IPv4 address>:8000` (allow Python through the Windows firewall on private networks).

## Smoke test

`tools/fc.sh web smoke` runs `tools/web/smoke.js` on a test build in `out/web/`: headless Chromium through Playwright
loads the page and, through the page probe, checks the boot (loading bar, both bundle parts, a main menu with no Quit),
the no-WebGL message, the URL query, (a) Tools > Combat Arena `goblin_skirmish` to the results screen, (b) New game to
the first region screen and (c) save, reload, Continue. Every page must run without a console or page error. It needs
Node with Playwright and its Chromium (or `CHROMIUM=<path>`); it is never in the gate.

## CI artifact (release PRs)

The `web (advisory)` job in `.github/workflows/gate.yml` runs on every PR into `main` (the release PR): it installs the
wasm32 target and `wasm-bindgen-cli` 0.2.129 (cached), runs `tools/fc.sh web test`, uploads `out/web/` as the artifact
`web-build` (kept 14 days), then runs the browser smoke (Playwright Chromium installed on the runner). The job is
advisory: it is not in the `gate` job's `needs` and not a required check, and a smoke failure does not fail the job
(read the smoke step's log). After it has been green on two releases the user decides whether `main` requires it
(WEBREL1 section 5). Download the artifact from the PR's checks (the `gate` workflow run > Artifacts).

## Deploy to itch.io (`web.yml`)

`.github/workflows/web.yml` is manual only: **Actions > web > Run workflow**, Branch **main**, variant **test**
(default) or **release**. The Merge & Release thread runs it when the user says "deploy". It:

1. refuses any ref but `main`, and stops before building with "BUTLER_API_KEY not set" when the secret is missing, or
   with an `ITCH_TARGET` message when the variable is missing or not of the form `<user>/<game>:html5`;
2. builds the chosen variant with `tools/fc.sh web <variant>`;
3. installs itch.io's `butler` for linux-amd64 from `https://broth.itch.zone/butler/linux-amd64/<version>/archive/default`,
   pinned to 15.32.0 and checked against its sha256 (to bump: fetch the new version and its sha256 on a GitHub runner,
   since the cloud proxy blocks `broth.itch.zone`, and change `BUTLER_VERSION` and `BUTLER_SHA256` together);
4. pushes `out/web/` with `butler push out/web $ITCH_TARGET --userversion <variant>-<short sha>`.

## One-time setup by the repo owner (itch.io page, secret, variable)

1. On itch.io: **Upload new project**. Kind of project: **HTML**. Leave the upload empty (CI fills it). Embed options:
   viewport **1280 x 720**, tick **Fullscreen button**, leave **SharedArrayBuffer support** off (the build is
   single-threaded). Visibility: **Restricted**, then set a password under **Password protect**. Save; the URL slug (for
   example `tactical-rpg`) is the project name.
2. itch.io **Settings > API keys > Generate new API key**; copy it.
3. GitHub repo **Settings > Secrets and variables > Actions**: **Secrets > New repository secret** `BUTLER_API_KEY` =
   the key; **Variables > New repository variable** `ITCH_TARGET` = `<itch username>/<project slug>:html5`. Nothing of
   either is in the repository.
4. Run the `web` workflow on `main`. After the first upload, tick **This file will be played in the browser** on the
   `html5` file in the project's edit page and save (one time).

## What differs in the browser

| Area | Behaviour in the browser |
| --- | --- |
| Quit | Hidden on the main menu (a page cannot close its own tab) |
| Saves and settings | In the browser's `localStorage` for the page's origin (keys under `aegis/`); clearing site data wipes them. itch.io serves every game from the shared `itch.zone` origin, so the storage quota may be shared |
| Playtest recordings and F12 screenshots | Off on the web |
| Audio | The second bundle part, fetched after the main menu; starts only after the first click or key press (browser autoplay rule) |
| WebGL missing | The game stops and the page shows a short message instead of drawing |
| Touch / mobile | Runs, but the UI is built for mouse and keyboard |
~~~

### FB-165: PR4-9 `WEBOPT1` parked: the `apt` Binaryen (108) breaks the web build

- **Spec**: READY Section 1 row PR4-9 (`FB-164` Option A); branch `claude/optimistic-newton-ewkyqt` @`6df0d0e4` (not landed)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: built as specified (`binaryen` added to the `apt-get install` of the `web` job in `gate.yml` and of `web.yml`; `tools/fc.sh web` now prints `wasm-opt (<version>): <before> -> <after> bytes`; both workflows parse and pass actionlint + shellcheck). With the `apt` package (Ubuntu's `binaryen 108-1`, the version an `ubuntu-latest` runner would also install) `wasm-opt -O2` shrinks the module 13,518,498 -> 9,788,617 bytes (3.8 -> 3.5 MB gzip), but **the optimised build does not boot**: the smoke fails 25 checks with `RangeError: WebAssembly.Table.grow(): failed to grow table by 4`. Cause (`wasm-dis` before and after): the input has `(table $0 4059 4059 funcref)` and `(table $__wbindgen_externrefs 1024 anyref)` exported as `__wbindgen_externrefs`; after `-O2` the export points at `table $0`, the fixed-size funcref table, so `wasm-bindgen`'s glue grows the wrong table (a Binaryen 108 bug with reference-types modules). The acceptance ("the smoke still passes on the optimised build") cannot be met with the `apt` package. A newer Binaryen release could not be tried from the cloud container (downloading the release binary was refused). `tools/fc.sh` runs `wasm-opt` whenever it is on the PATH, so any machine with an old Binaryen gets a broken web build today too (no machine in the project has one: it was uninstalled here after the test).
- **Recommendation**: **Option A (recommended)**: install a current Binaryen release in both workflows from its GitHub release archive (`WebAssembly/binaryen` `version_12x`, linux x86_64, pinned with a sha256 checked on a runner, like butler), plus a guard in `tools/fc.sh` that skips `wasm-opt` below a minimum version (and says so); Build verifies the smoke on the first release PR. **Option B**: build the web module without reference types (`-C target-feature=-reference-types`, so `wasm-bindgen` uses its JS heap and old `wasm-opt` works); a codegen change that needs the smoke and a size check. **Option C**: drop WEBOPT1 (the `.wasm` stays 3.8 MB gzip) and only add the version guard to `tools/fc.sh`.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
