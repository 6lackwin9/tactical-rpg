# ART_UI_STUDIO: Dedicated Artwork, Animation & Mock UI Session Reference

Status: active (user 2026-11-09: Dedicated Artwork/UI Session split & Design Session lock; compacted 2026-11-15 per `CTX-CLEANUP1-ARTUI`)
Skill: `art-ui-session` (`.agents/skills/art-ui-session/SKILL.md`, `.claude/skills/art-ui-session/SKILL.md`)
Agent: `art-ui-studio` (`.claude/agents/art-ui-studio.md`)
Depends on: [SCREEN_ATLAS.md](SCREEN_ATLAS.md), [CAST1_ASSETS.md](CAST1_ASSETS.md), [ART_PROMPT_TEMPLATES.md](ART_PROMPT_TEMPLATES.md), [ui.md](../preferences/ui.md), [assets-world.md](../preferences/assets-world.md)
Historical round logs: [archive/ART_UI_EXPERIMENTS_LOG.md](archive/ART_UI_EXPERIMENTS_LOG.md)

## 1. Purpose & Session Separation (Design Session Lock)

To let Tushar iterate on **Artwork, Character Sprite Animations, VN Stage Choreography, and Screen Layouts/UI** without touching or blocking the **Design Session** (`docs/READY.md`, system specs) or **Build Session** (`docs/FEEDBACK.md`, Rust crates), all visual/UI prototyping runs in the dedicated **Artwork/UI Session** (`/art-ui-session`):

| Session | Scope & Owned Paths | Explicitly Locked / Forbidden Paths |
| --- | --- | --- |
| **Artwork/UI Session (`/art-ui-session`)** | `tools/mock_ui/**` (including `tools/mock_ui/anim_lab/**`), `art_src/**`, `games/emberwake/assets/**`, `kits/rpg/assets/**`, `engine/assets/**`, `games/emberwake/docs/specs/ART_UI_STUDIO.md`, `SCREEN_ATLAS.md`, `CAST1_ASSETS.md`, `games/emberwake/docs/preferences/{ui,assets-world}.md` | **Never edit**: `docs/READY.md`, `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, Rust crates, or `games/emberwake/data/**`. |
| **Design Session (`/design-session`)** | `docs/READY.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/DECISIONS.md`, `docs/TASKS.md`, `/audit-build` | **Locked out of** live Mock UI rigging and sprite/animation iteration unless importing a signed-off spec into `READY.md`. |
| **Build Session (`/build-session`)** | `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md`, Rust crates, `games/emberwake/data/` | **Never edit** `docs/READY.md`, specs, or `tools/mock_ui/**`. |

## 2. Mandatory Artwork & Animation Iteration Rules

1. **Lightweight Standalone Preview / GIF First**: Iterate in reduced previews (`py -3.12 -m tools.mock_ui.anim_lab` or `art_src/vn_stage/*.gif`), never inside full `emberwake_screen_flow.html` first; promote into `tools/mock_ui/` only after user sign-off.
2. **Solid `#FF00FF` Magenta Key for Transparent Sprites**: Always prompt on flat `#FF00FF` with zero cast ground shadow, zero labels, and clear gaps; chroma-key and despill in Python (clear RGB on transparent pixels before scaling).
3. **Mandatory Post-Generation Visual Inspection**: View every generated or cut asset directly (`read_file`) to verify identity/palette lock (e.g., Kaelen's shoulder-length golden-blonde hair, cream gambeson, scarred breastplate), zero clipping, and zero pink halo/gutter pixels.
4. **Name References by File Name & Stage Cleanly**: List references by exact filename in attach order (`--stage-refs` / `queue_sunk_ward.py`) and verify files are real assets rather than Git LFS pointers.
5. **Layout & Look Before UX/Behavior**: Lock static look and framing before wiring complex runtime state.
6. **External Paid API Keys Banned Without Explicit Approval**: Never read `.env` or run paid API modes unless explicitly instructed; use Drive web queue, Flow, local deterministic tools, or authorized built-in `image-generator`.

## 3. Active Contracts & Settled Pipeline Decisions

### 3.1 4-Pose Character Rig & Keyframe / Video Animation Contract

- **Architecture (`1 Passive + 3 Active`)**: `passive_relaxed` (`y = 35..324` grounded vertical polearm), `active_walk` (`1H Travel` & `2H Patrol`), `active_sneak` (`Crouched Stealth`), and `active_drawn` (`Weapons Drawn` calm stoic face + weapon action & short tip glint at `(274, 114)`).
- **Grounding & Layer Lock**: Flat-planted boots at `y = 324`, height-matched (`y = 35..324`), fixed `legL` back / `legR` front layer with zero stride crossover.
- **Video-to-Sprite Pipeline (`tools/mock_ui/anim_lab/tween.py`)**:
  - Cycles (walk/sneak): 360p start frame -> Veo -> `--video-loop` (two steps + overlap blend after ease-in -> transparent sheet + `.json`, 24 fps).
  - Attacks (poke/slash): 720p tall start (`--export-video-start-tall`) for overhead headroom -> `--video-action [--tall] [--no-trail]` -> non-looping `.png` sheet + `.json`.
  - Prompting rule for gap frames (`halfdraw_r2` / `lift_r2`): edit image 1 part-way, state pose relative to both references, avoid "copy frame 1 closely" on multi-frame cycles, and render masters as PNG (not JPEG).

### 3.2 VN Story Stage & Flow Character CG Contract

- **Choreography (`StoryStage`)**: Moving party units enter from the **LEFT** and exit **RIGHT** in both `Scroll BG` and `Pan Camera`; encountered NPCs lock static to backdrop `worldX` off-frame **RIGHT** and scroll into stopping distance.
- **Flow Characters (`3.7`)**: Register per-character portrait + 3-view turnaround (`@Character`, `@Vesper` style guide for muted grime/palette), name image-left/image-right explicitly, and enforce `"no text, no caption, no rain"` in CG prompts.

### 3.3 Region Maps, Props & Terrain Tiles Contract (`art_src/region_maps/README.md`)

- **Props View & Grounding (Rounds 4–5 / AI-13)**: **Straight front-on top-down view** (south face + top band flat to viewer, vertical/horizontal edges square to the grid; never attach isometric landmarks to prop prompts). **NO cast oval shadows under props** — draw only a tight dark base contact line (`cut_props.py` + `render_map.py`).
- **Dual Profiles**: Wall-facing/asymmetric props ship `<art>` (front / north wall / free-standing) and `<art>_side` (west wall facing north-south; flipped east).
- **Grounds & Boundaries (`tools/tiles/{cut,seam,flatten,groundcheck}.py`)**: Ground tiles are straight-down `256x256` (`ground/<id>/v0.png`), dark and muted toward `grounds.json` target hex (`CELLS` tile span, `GREY` desaturation where needed), seamless (`seam.py fix`), and quiet under small figures (`groundcheck.py`); water never warps at transitions (stone rim instead). Boundary sets ship `wall/v0.png` (`32x32`) and `wall_face/v0.png` (`32x16`).

### 3.4 Music Pipeline Contract (`MUSIC1`, `art_src/music/prompts/`)

- **Flow Music (`tools/mock_ui/music.py`)**: Lyria 3.5 Compose with house seed `455684` (`menu_theme`, `sunk_ward_town` via `--gap 2.5`; `sunk_ward_combat*` stem mixes via `--rotate`). Never regenerate or overwrite finalized Flow `.ogg` tracks via `gen_music.py`.
- **Circular-Buffer Synth Placeholders (`tools/gen_music.py`)**: Story (`story_opening`, `prologue_tension`, `story_escape`, `story_goodbye`) and remaining Sunk Ward contexts (`sunk_ward_danger`, `sunk_ward_boss`, `sunk_ward_shop`, `sunk_ward_travel`) render as seamless `44.1 kHz` stereo Ogg Vorbis (`-q:a 5`) via `tools/gen_music.py`.

### 3.5 Experiment History Index (`archive/ART_UI_EXPERIMENTS_LOG.md`)

| Section | Date | Summary & Key Artifact | Archive Link |
| --- | --- | --- | --- |
| `3.1`–`3.4` | 2026-11-08..09 | Rigging (`Option C` 4-Pose), VN Scroll/Pan choreography, leg/boot lock, combat glint | [Experiments 1–4](archive/ART_UI_EXPERIMENTS_LOG.md#31-experiment-1-character-rigging-approach-options-a-b-c) |
| `3.5` | 2026-11-09 | Keyframes vs Optical Flow vs hybrid rigid-weapon (`tween.py`, 6-frame/8-frame sheets) | [Experiment 5](archive/ART_UI_EXPERIMENTS_LOG.md#35-experiment-5-passive-stop-frames-vs-active-keyframes-with-code-in-betweens-in-review-2026-11-09) |
| `3.6` | 2026-11-09..10 | Google Veo 360p/720p video loops & `--no-trail` attack cuts | [Experiment 6](archive/ART_UI_EXPERIMENTS_LOG.md#36-experiment-6-image-to-video-for-the-actions-google-veo-in-setup-2026-11-09) |
| `3.7`–`3.8` | 2026-11-10 | Flow character registration, prologue CGs, Kaelen baseline handoff | [Experiments 7–8](archive/ART_UI_EXPERIMENTS_LOG.md#37-experiment-7-story-cgs-from-google-flow-character-references-in-setup-2026-11-10) |
| `3.9` | 2026-11-11 | Flow Music Producer vs Compose seed `455684`, stem splitting & `--gap`/`--rotate` | [Experiment 8 (Music)](archive/ART_UI_EXPERIMENTS_LOG.md#39-experiment-8-music-from-flow-music-in-setup-2026-11-11) |
| `3.10` | 2026-11-12..15 | UI feedback rounds, Asset Sweep (`ASSET_PLAN.md`), Sunk Ward tile/prop/hub rounds 1–5 | [Inbox Log 3.10](archive/ART_UI_EXPERIMENTS_LOG.md#310-request-inbox-log-2026-11-12) |

## 4. Screen-by-Screen UI Decisions Summary (`emberwake_screen_flow.html`)

| Screen | Key Settled UI / Visual Decisions | Mock UI Module (`tools/mock_ui/`) |
| --- | --- | --- |
| **1. Main Menu (`MainMenu`)** | Atmospheric Sunk Ward mist backdrop (`title_sunk_ward_mist_v2.jpg`); 60fps floating ember/ash canvas; text-only menu column (`Continue` lit on open; corner-only `ornate` `peek_room` on hover/focus). | `js/03_main_menu_and_slots.js` |
| **2. Chronicle Slots (`Load/Save`)** | 2-click Load/Save modal; `New Chronicle` boots directly into `sunk_ward`. | `js/03_main_menu_and_slots.js` |
| **3. Settings Modal (`Settings`)** | Staged Display Mode / Resolution with 15s revert confirmation and unapplied-changes close guard. | `js/04_settings.js` |
| **4. VN Story Stage (`StoryStage`)** | Full-body standing sprites on widescreen backdrops; bottom ornate dialogue dock (`TalkBox`); camera modes (`Static`, `Scroll BG`, `Pan Camera`). | `js/05_story_and_region.js` |
| **5. Hub & Region (`RegionScreen`)** | Top `hud_bar` (Scrap, Tallow, Alert, Red Tithing); collapsible `"Here (N)"`; `BoardPane` & `ShopPane`. | `js/05_story_and_region.js` |
| **6. Party & Gear (`PartyScreen`)** | 6 Founder tabs (`Q/E`), `Active (4 of 6)` bunk toggle, 5-pip visual Belt Pouch, `Wounded` badge. | `js/05_story_and_region.js` |
| **7. Tactical Battle (`BattleScreen`)** | Continuous color-coded 6-beat timeline blocks; icon-only `SkillMenu` beside clicked target; `RING`/`ARC` overlays; `T` timeline inspector. | `js/06_battle.js` |

## 5. Reference Tooling & Active Asset Index

- **Standalone Animation Lab**: `py -3.12 -m tools.mock_ui.anim_lab --out <path>`
- **Tween Comparison Lab**: `py -3.12 -m tools.mock_ui.anim_lab.tween --out <path> [--gif-dir art_src/vn_stage]`
- **Full Screen Flow Mock UI**: `py -3.12 -m tools.mock_ui --out <path> [--rebuild-assets]`
- **Sunk Ward Prop & Tile Cutters**: `art_src/prompts/cut_props.py`, `art_src/prompts/cut_ai_sheets.py`, `art_src/region_maps/render_map.py`, `tools/tiles/{cut,seam,flatten,groundcheck}.py`
- **Docs Lint Gate**: `tools/fc.sh docs`
- **Staging & Retired Probe Index**: [art_src/ARCHIVE_INDEX.md](../../../art_src/ARCHIVE_INDEX.md) (lists active sheets vs superseded first takes / probes kept cleanly on disk so script paths stay intact).
