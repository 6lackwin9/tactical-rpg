# Emberwake UI Prototype (`tools/mock_ui`)

Modular builder for the self-contained playable HTML5 UI & game-loop preview (`emberwake_screen_flow.html`), modeled on `games/emberwake/crates/emberwake_gui/src/*.rs` and `engine/crates/aegis_ui/src/*.rs`.

## Usage

```bash
# Build HTML using cached asset bundle (regenerating cache automatically if missing)
python3 -m tools.mock_ui

# Force-rebuild the asset bundle from games/emberwake/assets, kits/rpg/assets, and engine/assets
python3 -m tools.mock_ui --rebuild-assets

# Write to a custom output path
python3 -m tools.mock_ui --out /path/to/emberwake_screen_flow.html

# Build the lightweight standalone 60fps Kaelen 4-Pose & Keyframe Animation Lab (use FIRST before full mock UI)
python3 -m tools.mock_ui.anim_lab --out /path/to/kaelen_4pose_animation_lab.html
```

See [ART_UI_STUDIO.md](../../games/emberwake/docs/specs/ART_UI_STUDIO.md) and skill `/art-ui-session` for the complete Artwork/UI Session workflow, chronological options/feedback log, and reference asset index (`art_src/vn_stage/`).

## Module Layout

- `anim_lab/` — Standalone 60fps 4-pose (`1 Passive + 3 Active`) and weapon-action animation sandbox (`kaelen_4pose_uris.json` + `template.html` + `__main__.py`), supporting both **Directly Generated Keyframe Sprites** and **Procedural Shear/Kinetic Chain** rigging for rapid lightweight iteration outside the 3.2 MB full mock UI.
- `anim_lab/tween.py` — Keyframe in-between comparison lab (`python3 -m tools.mock_ui.anim_lab.tween --out <path>`): the same clips as generated keys only, optical-flow in-betweens, and a hybrid with one rigid halberd placed by `tween_landmarks.json`, on one shared palette (`tween_template.html`; needs `opencv-python-headless`). ART_UI_STUDIO Experiment 5.
- `assets.py` — Reads layered assets (`games/emberwake/assets`, `kits/rpg/assets`, `engine/assets`), extracts 9-slice pieces for both `ui/frames/ornate.png` (Reduced Corner-Only Frame) and `ui/frames/ornate_full.png` (Full Ornate Frame with borders and medallions), applies unit palette swaps, and encodes fonts/images as data URIs.
- `build.py` / `__main__.py` — Combines `shell.html`, `styles.css`, the asset bundle, and `js/*.js` into a single standalone `.html` file.
- `shell.html` — Document structure, top Design Session inspector bar (`#design-bar`), 1280×720 viewport container (`#viewport-wrap`), and bottom decision bar (`#arch-bar`).
- `styles.css` — Shared CSS variables, widget primitives (`.rust-panel`, `.rust-card`, `.rust-btn`, `.rust-btn-720`), full & corner-only ornate frames (`.ornate-box`, `.ornate-box-640`, `.ornate-corner-box`, `.ornate-corner-box-640`), close `[✕]` buttons (`.close-x-720`, `.close-x-640`), and title-screen atmospheric FX.
- `js/01_state_and_data.js` — Global `STATE` plus catalog data (`SKILLS`, `ENCOUNTERS`, `STORY_DATA`, `REGIONS`, `DIALOGUES`, `WORLD_NODES`, `BOARD_QUESTS`, `SHOPS`).
- `js/02_widgets_and_nav.js` — Shared ornate frame helpers (`ornateBox*`, `ornateCornerBox*`), close button helpers (`closeBtn720`, `closeBtn640`), unified `uiBack()` handler (`Escape` / Right-Click / `✕`), top HUD bar, and screen/quest navigation helpers.
- `js/03_main_menu_and_slots.js` — 1280×720 Main Menu, 60fps title particle canvas, New Chronicle setup popup, and 2-click Load/Save Chronicle Slots screen.
- `js/04_settings.js` — Fullscreen & In-Game Settings modal, staged Display Mode / Resolution with 15s revert confirmation, and unapplied Display changes close guard (`requestCloseSettings`).
- `js/05_story_and_region.js` — `SceneRunner`, `StoryStage`, `RegionScreen` (Hub & Street maps + `TalkBox`), `BoardPane`, `ShopPane`, `PartyScreen` (6 tabs), and `WorldMap`.
- `js/06_battle.js` — `BattleScreen` (tactical grid, intent planner, `SkillMenu`, enemy intel), `Timeline` full overlay, and `BattleResults` modal.
- `js/07_bootstrap.js` — Main `render()` dispatcher, screen-specific inspector bar toggles, `fitViewportScale()`, and keyboard / right-click (`contextmenu`) listeners.
