---
name: art-ui-session
description: Bootstrap and operate the dedicated Artwork, Animation, and Mock UI Session (/art-ui-session, "start art-ui session", "start artwork session", "start UI session") isolated from the locked Design and Build Sessions. Loads games/emberwake/docs/specs/ART_UI_STUDIO.md, SCREEN_ATLAS.md, tools/mock_ui, tools/mock_ui/anim_lab, and art_src/vn_stage so UI layout, VN stage choreography, and character pose/keyframe animations iterate lightweight-first without touching READY.md or Rust crates.
---

# Artwork, Animation & Mock UI Session (`/art-ui-session`)

Operate as the **Dedicated Artwork, Animation & Mock UI Session** (`art-ui-session`), completely isolated from the **Design Session** (`docs/READY.md`, gameplay specs) and **Build Session** (`docs/FEEDBACK.md`, Rust crates).

## 1. Ownership & Session Lock Boundaries

- **Owned Paths (Allowed to Edit)**:
  - `tools/mock_ui/**` (playable 1280×720 Mock UI builder `python3 -m tools.mock_ui` and standalone 60fps Animation Lab `python3 -m tools.mock_ui.anim_lab`)
  - `art_src/**` (including `art_src/vn_stage/**` reference sprite sheets, `#FF00FF` magenta keyframes, stage backdrops, and animated GIFs)
  - `games/emberwake/assets/**`, `kits/rpg/assets/**`, `engine/assets/**`
  - `games/emberwake/docs/specs/ART_UI_STUDIO.md`, `games/emberwake/docs/specs/SCREEN_ATLAS.md`, `games/emberwake/docs/specs/CAST1_ASSETS.md`, `games/emberwake/docs/specs/ART_PROMPT_TEMPLATES.md`
  - `games/emberwake/docs/preferences/ui.md`, `games/emberwake/docs/preferences/assets-world.md`
- **Strictly Locked / Forbidden Paths (Never Edit in Artwork/UI Session)**:
  - **Design Session Queue & Governance Docs**: `docs/READY.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/TASKS.md`, `docs/DECISIONS.md`, `docs/HISTORY.md`
  - **Build Session Output & Code/Data**: `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md`, `engine/crates/**`, `kits/rpg/crates/**`, `games/emberwake/crates/**`, `games/emberwake/data/**`

## 2. Startup Sequence (Run on Session Start)

1. Sync with `origin/develop` and read the canonical Artwork/UI history & reference index:
   - Read `games/emberwake/docs/specs/ART_UI_STUDIO.md` (contains all prior options tried, Tushar's feedback, settled rules, and the complete asset/tool catalog).
   - Read `games/emberwake/docs/specs/SCREEN_ATLAS.md` and `games/emberwake/docs/preferences/{ui,assets-world}.md` when working on screen layouts or visual chrome.
2. Verify available preview builders:
   - **Standalone Animation Lab (`~600 KB..1.7 MB`)**: `python3 -m tools.mock_ui.anim_lab --out <artifact_dir>/kaelen_4pose_animation_lab.html`
   - **Full Screen-Flow Mock UI (`~3.2 MB`)**: `python3 -m tools.mock_ui --out <artifact_dir>/emberwake_screen_flow.html`

## 3. Mandatory Iteration Rules (Never Repeat Past Issues)

1. **Lightweight Standalone Preview / GIF First**:
   - **Never** wire experimental animations, sprite rigs, or layout tests directly into the full 3.2 MB `emberwake_screen_flow.html` first.
   - Always iterate in a **focused standalone HTML artifact** (`tools/mock_ui/anim_lab`) or generate **animated GIFs** (`art_src/vn_stage/*.gif`) first. Integrate into `tools/mock_ui/` (`emberwake_screen_flow.html`) **only after** Tushar reviews and approves the animation or screen look.
2. **Look/Layout First, UX/Behavior Second**:
   - For screen layouts or artwork, get sign-off on the static visual layout/look before wiring state or UX interactions.
3. **4-Pose (`1 Passive + 3 Active`) Character Architecture**:
   - **`1 Passive` (`passive_relaxed`)**: Neutral stop/dialogue frame (`y = 35..324`, weapon grounded vertically).
   - **`3 Active`**:
     - `active_walk` (`1H Travel Carry` & `2H Patrol Carry`): Torso and head turned into the walk direction, height-matched to `passive_relaxed` (`y = 35..324`, zero stop-transition bounce), flat-planted boots, non-crossing fixed-layer legs.
     - `active_sneak`: Crouched stealth posture facing the travel direction, non-crossing fixed-layer stride.
     - `active_drawn`: **Calm stoic closed-mouth face** (no snarling teeth; threat is shown through weapon action `Poke` / `Cleave` / `Brandish`, with emotion via Expression FX overlays).
   - **Dual Animation Engines (`tools/mock_ui/anim_lab`)**:
     - **Engine A (Recommended — Directly Generated Keyframes)**: Uses hand-drawn `#FF00FF` magenta keyframe sprites (`walk_1h_kf0..3`, `sneak_kf_passing`, `drawn_kf_{windup,poke,cleave}`) aligned to `340×340`.
     - **Engine B (Procedural Shear / Cohesive Kinetic Chain)**: Non-crossing fixed-layer leg shear + 3-layer Drawn kinetic chain (`drawn_legs` shearing from planted boots `y = 324` + `drawn_torso` with attached upper sleeves pivoting from hips `(168, 214)` + `drawn_weapon` extending from bracer cuffs).
   - **Cold Steel Weapon Glint Rule**: Pinned strictly to the **weapon tip** `(274, 114)` (never sliding along the shaft), compact size (`±5.5px`), triggered as a **short one-time flash (`~0.4s`)**.
4. **VN Stage Camera & Encounter Choreography (`05_story_and_region.js`)**:
   - In `🔄 Scroll BG` and `🎥 Pan Camera`, all moving units **enter from the LEFT and leave to the RIGHT**.
   - Any unit encountered along the way starts **outside the frame on the RIGHT and remains static relative to the background** (`worldX` locked to the backdrop), scrolling into view as the party advances.
5. **Image Generation Hygiene**:
   - Always generate sprites on a **solid flat `#FF00FF` bright magenta background** (prefer single-character `1:1` generation from a `340×340` `#FF00FF` reference so figures never overlap).
   - Immediately inspect every generated image with `view_file` to verify character identity (e.g., Kaelen's golden-blonde hair), weapon details, and unclipped bounds before presenting.
   - Run `bash tools/fc.sh docs` before committing any documentation or asset updates in `games/emberwake/docs/specs/ART_UI_STUDIO.md`.
