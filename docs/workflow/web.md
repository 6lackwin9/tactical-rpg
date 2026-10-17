# Web build (HTML5, GitHub Pages)

A browser build for cross-platform testing and porting checks (user 2026-10-17). It is a release build and less debug-friendly than the desktop runner; playtesting and debugging stay on desktop.

## How it is built

- Preset `Web` in `export_presets.cfg`: single-threaded templates (`variant/thread_support=false`), so the page needs no cross-origin isolation headers (`COOP`/`COEP`) and no `SharedArrayBuffer`. GitHub Pages cannot set those headers, so the threaded templates are not used. The preset adds `data/*.json` to the pack (the catalogs are read with `FileAccess`, not as resources).
- Renderer: the web always uses Compatibility (WebGL 2); desktop uses Forward+.
- Workflow `.github/workflows/web.yml` (user 2026-10-17: once a day and on request, to cut CI time): a daily run at 19:07 UTC on `main` exports the build, uploads it as the `web-build` artifact (14 days) and deploys it (itch.io below; GitHub Pages when the repository variable `WEB_PAGES` is `true`); it skips everything when `main` is still at the commit of the last successful daily or manual run. A manual run (**Actions > web > Run workflow** on `main`, or ask in the project chat: "deploy") always builds and deploys. Pushes to `main` no longer build; PRs into `main` build (no deploy) only when they change `export_presets.cfg` or `web.yml`.
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
| Content Forge and World Forge saves | `res://` is read-only: edits work for the session but saving to the catalog fails |
| Saves and prefs (`user://`) | Work, stored in the browser's IndexedDB for that site; clearing site data wipes them |
| Emoji icons in labels and buttons | Render from bundled fallback fonts (`assets/fonts`, web only): monochrome instead of the desktop's color emoji; they add about 2.5 MB to the download (Noto Emoji is the full font so new emoji work without re-subsetting) |
| Keys | F5 may reload the browser tab instead of the scene; use the Menu pill buttons |
| Debug output | Release build: no debug-only logs (`combat_last.log` is skipped) and no remote debugger |
| Audio | Starts only after the first click or key press (browser autoplay rule) |
| Touch / mobile | Runs, but the UI is built for mouse and keyboard |
