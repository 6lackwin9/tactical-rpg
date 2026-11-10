# Web build (Aegis WebAssembly, itch.io)

The Aegis game runs in a browser since `PR4-6` `WEBREL1-G1` (2026-11-10; spec `docs/specs/WEBREL1.md`, decision "Post-R4 queue and web release (user 2026-11-09)"). Hosting: a password-protected itch.io page; every release PR builds the web bundle in CI and the upload to itch.io happens only when the user says "deploy" (the CI job and `web.yml` landed with `PR4-7` `WEBREL1-P1`, 2026-11-10).

## Build, serve, smoke (Aegis)

- Needs the `wasm32-unknown-unknown` target and `cargo install wasm-bindgen-cli --version 0.2.129 --locked`; `wasm-opt` is used if installed.
- Build: `tools/fc.sh web test` (Tools menu, dev tools and the `aegisProbe()` test probe) or `tools/fc.sh web release` (no Tools, no dev keys) writes the gitignored `out/web/`: `index.html`, `emberwake.js`, `emberwake_bg.wasm`, `bundle.json`, `emberwake_1.pak` (game, kit and engine files without audio), `emberwake_2.pak` (audio, fetched while the game runs), `build.json`. The bundle list is `games/emberwake/web_bundle.json`, packed by `aegis bundle`.
- Serve: `tools/fc.sh web serve [port]`, then open http://127.0.0.1:8000/. URL keys are the CLI's, for example `?screen=arena&encounter=goblin_skirmish&seed=3`; desktop-only keys (`port`, `headless`, `data`, `saves` and the like) are ignored with a console warning.
- Smoke: `tools/fc.sh web test && tools/fc.sh web smoke` (Node with Playwright Chromium; not in the gate): boot to the main menu, Combat Arena to victory, New game to the region map, save, reload and Continue.
- Other ways to play: a downloaded `web-build` artifact can be unzipped and served with `python -m http.server 8000` (then `http://localhost:8000`); other devices on the same Wi-Fi reach `python -m http.server 8000 --bind 0.0.0.0` at `http://<PC's IPv4 address>:8000` (allow Python through the Windows firewall on private networks). Opening `index.html` as a file does not work.

## Test build and release build

| Build | Command | Contents | Used for |
| --- | --- | --- | --- |
| Test (`dev_tools` on) | `tools/fc.sh web test` (the default) | Base game plus the Tools menu (Scene Runner, Combat Arena, Forge viewers, Gear Dojo, UI Gallery) and the page probe the smoke test reads | The CI artifact of every release PR; the itch.io deploy until the user calls the game playable |
| Release (`dev_tools` off) | `tools/fc.sh web release` | Base game only: Tools and dev keys compiled out | The deploy once the user calls the game playable |

`dev_tools` is a cargo feature of `emberwake_gui`, on by default, so the native desktop build does not change.

## CI artifact (release PRs)

The `web (advisory)` job in `.github/workflows/gate.yml` runs on every PR into `main` (the release PR): it installs the wasm32 target and `wasm-bindgen-cli` 0.2.129 (cached), runs `tools/fc.sh web test`, uploads `out/web/` as the artifact `web-build` (kept 14 days), then runs the browser smoke (Playwright Chromium installed on the runner). The job is advisory: it is not in the `gate` job's `needs` and not a required check, and a smoke failure does not fail the job (read the smoke step's log). After it has been green on two releases the user decides whether `main` requires it (WEBREL1 section 5). Download the artifact from the PR's checks (the `gate` workflow run > Artifacts).

## Deploy to itch.io (`web.yml`)

`.github/workflows/web.yml` is manual only: **Actions > web > Run workflow**, Branch **main**, variant **test** (default) or **release**. The Merge & Release thread runs it when the user says "deploy". It:

1. refuses any ref but `main`, and stops before building with "BUTLER_API_KEY not set" when the secret is missing, or with an `ITCH_TARGET` message when the variable is missing or not of the form `<user>/<game>:html5`;
2. builds the chosen variant with `tools/fc.sh web <variant>`;
3. installs itch.io's `butler` for linux-amd64 from `https://broth.itch.zone/butler/linux-amd64/<version>/archive/default`, pinned to 15.32.0 and checked against its sha256 (to bump: fetch the new version and its sha256 on a GitHub runner, since the cloud proxy blocks `broth.itch.zone`, and change `BUTLER_VERSION` and `BUTLER_SHA256` together; the pin is carried over from the Godot-era workflow, so the first real deploy also checks it, `FB-164`);
4. pushes `out/web/` with `butler push out/web $ITCH_TARGET --userversion <variant>-<short sha>`.

## One-time setup by the repo owner (itch.io page, secret, variable)

1. On itch.io: **Upload new project**. Kind of project: **HTML**. Leave the upload empty (CI fills it). Embed options: viewport **1280 x 720**, tick **Fullscreen button**, leave **SharedArrayBuffer support** off (the build is single-threaded). Visibility: **Restricted**, then set a password under **Password protect**. Save; the URL slug (for example `tactical-rpg`) is the project name.
2. itch.io **Settings > API keys > Generate new API key**; copy it.
3. GitHub repo **Settings > Secrets and variables > Actions**: **Secrets > New repository secret** `BUTLER_API_KEY` = the key; **Variables > New repository variable** `ITCH_TARGET` = `<itch username>/<project slug>:html5`. Nothing of either is in the repository.
4. Run the `web` workflow on `main`. After the first upload, tick **This file will be played in the browser** on the `html5` file in the project's edit page and save (one time).

## What differs in the browser

| Area | Behaviour in the browser |
| --- | --- |
| Quit | Hidden on the main menu (a page cannot close its own tab) |
| Saves and settings | In the browser's `localStorage` for the page's origin (keys under `aegis/`); clearing site data wipes them. itch.io serves every game from the shared `itch.zone` origin, so the storage quota may be shared |
| Playtest recordings and F12 screenshots | Off on the web |
| Dev keys (F6, F7, F9) | Need a debug build with the dev tools, so neither web build has them (`FB-162`) |
| Audio | The second bundle part, fetched after the main menu; starts only after the first click or key press (browser autoplay rule) |
| WebGL2 missing | The game stops and the page shows "The game stopped: ..." instead of drawing |
| Touch / mobile | Runs, but the UI is built for mouse and keyboard |

## Godot web export (retired 2026-10-19, kept as the record)

**Retired 2026-10-19 with the Godot build (transition step 2).** The rest of this page describes the Godot web export: the preset
(`export_presets.cfg`) and `.github/workflows/web.yml` are gone from `main` and stay on the `legacy` branch. Deploys were already paused
(decision "Web deploy paused (user 2026-10-17)"). Kept as the record of how the Godot build was published.

A browser build for cross-platform testing and porting checks (user 2026-10-17). It is a release build and less debug-friendly than the desktop runner; playtesting and debugging stay on desktop.

## How it is built

- Preset `Web` in `export_presets.cfg`: single-threaded templates (`variant/thread_support=false`), so the page needs no cross-origin isolation headers (`COOP`/`COEP`) and no `SharedArrayBuffer`. GitHub Pages cannot set those headers, so the threaded templates are not used. The preset adds `games/emberwake/data/*.json` to the pack (the catalogs are read with `FileAccess`, not as resources).
- Renderer: the web always uses Compatibility (WebGL 2); desktop uses Forward+.
- Entry scene (WEB3, user 2026-10-17): the browser build opens on the Main Menu (`application/run/main_scene.web` = `scenes/main_menu.tscn`, which installs the fallback fonts and hides Quit, since a page cannot close its tab); the desktop keeps the Scene Runner (`run/main_scene`). Debug web exports still reach the tools through the menu's Tools button.
- Workflow `.github/workflows/web.yml` (user 2026-10-17: once a day and on request, to cut CI time; user 2026-10-17: deploy after each release), PAUSED until a playable build (user 2026-10-17): no after-release or 19:07 UTC runs for now; a manual run exports the build, uploads it as the `web-build` artifact (14 days) and deploys it (itch.io below; GitHub Pages when the repository variable `WEB_PAGES` is `true`); the `check` job that skips a scheduled run when `main` is unchanged stays in place for when the schedule returns. A manual run (**Actions > web > Run workflow** on `main`, or ask in the project chat: "deploy") always builds and deploys. Pushes to `main` no longer build; PRs into `main` build (no deploy) only when they change `export_presets.cfg` or `web.yml`.
- Pack contents (REVIEW-F4): the preset's `exclude_filter` drops `build/*`, `scripts/tests/*` and `games/emberwake/data/seeds/*` (tests and seed scripts are dev-only; nothing shipped preloads them, `asset_scan.gd` only names the paths as strings). Re-check with `strings build/web/index.pck | grep -c "scripts/tests/"` (expect 0) after adding references.
- CI hardening (REVIEW-F4): the concurrency group cancels in-progress runs only for PRs (a running deploy is never cancelled); the `itch` job has a 15-minute timeout. Butler is pinned to 15.32.0 (`BUTLER_VERSION` + `BUTLER_SHA256` in the `itch` job, checked with `sha256sum -c` before unzip); to bump, fetch the new version and its sha256 from a GitHub runner, since the cloud proxy blocks broth.itch.zone.
- Templates come from the GitHub release `.tpz` (downloads.godotengine.org is blocked in the cloud); only `web_nothreads_*.zip` and `version.txt` are kept, cached per Godot version.

## Build it locally (cloud or Linux)

```bash
v=4.7.2; d=~/.local/share/godot/export_templates/$v.stable; mkdir -p "$d"
curl -fsSL -o /tmp/t.tpz "https://github.com/godotengine/godot/releases/download/$v-stable/Godot_v$v-stable_export_templates.tpz"
unzip -o -j -q /tmp/t.tpz 'templates/web_nothreads_*' 'templates/version.txt' -d "$d"
mkdir -p build/web && godot --headless --export-release "Web" build/web/index.html
cd build/web && python3 -m http.server 8000   # then open http://localhost:8000
```

On Windows the editor's Export dialog uses the same preset once its export templates are installed. `build/` is git-ignored. To run a downloaded `web-build` artifact, unzip it and serve the folder the same way (opening `index.html` as a file does not work).

## Playing it on the Windows PC (current hosting, user 2026-10-17)

- From the editor: install the export templates once (**Editor > Manage Export Templates > Download and Install**), open the project, then click the **Run in Browser** button (the browser icon, top right). The editor exports the `Web` preset and opens it at `http://localhost:8060`.
- From a CI build: on the latest `web` run in the Actions tab, download `web-build`, unzip it, and in that folder run `python -m http.server 8000`, then open `http://localhost:8000`.
- Other devices on the same Wi-Fi: run the server with `python -m http.server 8000 --bind 0.0.0.0` and open `http://<PC's IPv4 address>:8000` on the phone or tablet (Windows may ask to allow Python through the firewall: allow it on private networks).
- Cloud drives (Google Drive, OneDrive, Dropbox) do not serve web pages: they download or preview files and do not run the game.

## Private itch.io page (any device, user 2026-10-17)

The `itch` job in `web.yml` uploads each daily or manual run on `main` with butler once these are set (one time, by the repo owner):

1. On itch.io: **Upload new project**. Kind of project: **HTML**. Leave the upload empty (CI fills it). Embed options: viewport 1280 x 720, tick **Fullscreen button**; leave **SharedArrayBuffer support** off (the build is single-threaded). Visibility: **Restricted**, then set a password under **Password protect**. Save; the URL slug (for example `tactical-rpg`) is the project name.
2. itch.io **Settings > API keys > Generate new API key**; copy it.
3. GitHub repo **Settings > Secrets and variables > Actions**: **Secrets > New repository secret** `BUTLER_API_KEY` = the key; **Variables > New repository variable** `ITCH_TARGET` = `<itch username>/<project slug>:html5`.
4. Run the `web` workflow on `main` (**Actions > web > Run workflow**). After the first upload, tick **This file will be played in the browser** on the `html5` file in the project's edit page and save (one time).

## Turning on GitHub Pages (once, by the repo owner)

1. Pages on a private repository needs a paid plan (GitHub Pro or higher), and the published site is public to anyone with the URL.
2. Repository **Settings > Pages > Build and deployment > Source**: choose **GitHub Actions**.
3. **Settings > Secrets and variables > Actions > Variables > New repository variable**: name `WEB_PAGES`, value `true`.
4. Run the `web` workflow on `main` (**Actions > web > Run workflow**). The site is at `https://6lackwin9.github.io/tactical-rpg/`.

## What does not work in the web build

| Area | Behaviour in the browser |
| --- | --- |
| Scene Runner developer panel | Reload scripts, Restart runner and the Arena/Dojo suite buttons do nothing useful: the browser cannot start processes and the pack holds no editable script sources |
| Saves and prefs (`user://`) | Work, stored in the browser's IndexedDB for that site; clearing site data wipes them |
| Emoji icons in labels and buttons | Render from bundled fallback fonts (`games/emberwake/assets/fonts`, web only): monochrome instead of the desktop's color emoji; they add about 2.5 MB to the download (Noto Emoji is the full font so new emoji work without re-subsetting) |
| Keys | F5 may reload the browser tab instead of the scene; use the Menu pill buttons |
| Debug output | Release build: no debug-only logs (`combat_last.log` is skipped) and no remote debugger |
| Audio | Starts only after the first click or key press (browser autoplay rule) |
| Touch / mobile | Runs, but the UI is built for mouse and keyboard |
