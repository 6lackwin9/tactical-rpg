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

## 6. Retired Inbox Completions Log (2026-11-15, Phase 2–3)

- **`AI-03` (Story Music Tracks — retired)**: All 5 Flow Lyria context tracks landed in Phase 1–2 (`[GENERATED]`); the 4 story tracks (`story_opening`, `prologue_tension`, `story_escape`, `story_goodbye`) ship in `games/emberwake/assets/audio/music/` as `[PROVISIONAL synth placeholder]` loops generated via `tools/gen_music.py` (catalogued in `games/emberwake/assets/audio/music/README.md`).
- **`AI-10` (M5 Slag Gate CG — retired)**: `games/emberwake/assets/backgrounds/story/vg_m5_gate.png` (`1120x472`) landed (`[GENERATED]`).
- **`AI-12` (Icons & Sunk Ward NPC/Common Portraits — retired)**:
  - `families_1`, `gear_1` (incl. `items/{heavy,medium,light}/helm.png`), `hud_chips_2`, `kits/rpg/assets/statuses/{steadfast,stealth}.png`, all `14` `cit_<zone>` citizen portraits, and `portraits/npcs/{bailiff_corvo,skerrit_bone_wright}.png` landed cleanly with zero duplicate placeholders (`[GENERATED]`).
  - `art_src/prompts/cut_ai_sheets.py` updated with `skills_1..4` grid specs and rounded-corner outer-triangle alpha cleanup (`_slice_rounded_grid`); all `52` Sunk Ward skill icons cut to `games/emberwake/assets/skills/<id>.png` (`64x64` RGBA `[GENERATED]`).
  - Missing common portraits placed at `games/emberwake/assets/portraits/common/{warrior_1h_buckler,ranger_wenn_tallow}.png` (`240x240` RGBA `[PLACEHOLDER]` composites from `kits/rpg/assets/portraits/common/warrior_base.png` and `games/emberwake/assets/portraits/npcs/wenn_tallow.png`), clearing Sunk Ward unit portrait fallback warnings.
- **`AI-13` (Sunk Ward Map Art, Interior Walls & `_side` Props — retired)**:
  - `art_src/region_maps/sluice_undercroft.json` synced (`npc_c` at `(16, 6)`), `kits/rpg/assets/tilesets/common/ground/floorboards/v0.png` (`256x256`) and `tilesets/sunk_ward/boundary/{blind_tenements,drain_brick,gorge_rock}/` landed (`[GENERATED]`).
  - `games/emberwake/assets/tilesets/interior_{plaster,stone}/{wall (128x96), wall_face (128x48)}/v0.png` processed seamlessly from `art_src/sunk_ward/tiles/walls_interior_*.png` (`[GENERATED]`, zero edge seams).
  - Remaining `9` missing vertical N–S `<art>_side` props (`crypt_bars_side 64x128`, `iron_ladder_side 48x96`, `culvert_gate_side 48x96`, `barred_gate_side 48x96`, `door_side 48x96`, `goods_shelf_side 48x96`, `weapon_rack_side 48x96`, `mission_board_side 48x96`, `pen_fence_side 32x112`) placed in `games/emberwake/assets/props/sunk_ward/<art>_side/v0.png` (`[PLACEHOLDER]` deterministic Sunk Ward pixel art matching the existing `7` `_side` pieces, 1px dark outline, zero cast floor shadows).

## 7. Phase 4 Provisional Skill, Item & Graft Placeholders (2026-11-15, `PR5-62b` / `PR5-62c`)

- **`U2_*` Missing Icons (`110` `[PLACEHOLDER]` PNGs, `0` pre-existing files modified)**: Deterministically rendered in 16-bit Sunk Ward style (`64x64` RGBA rounded dark-slate tiles for skills; `96x96` RGBA octagonal dark-slate tiles for items/grafts; zero paid API calls). Pending replacement with Flow sheets in a post-`PR5-62c` Art/UI pass:
  - **Off-Hand (`5` in `games/emberwake/assets/skills/*.png`)**: `brace`, `reload`, `fade_step`, `catalyst_pulse`, `cite_passage`.
  - **Class Deck Actives & Basics (`14` in `skills/*.png`)**: `hold_line`, `intercept_guard`, `brawler_clinch`, `slag_charge`, `disengage_step`, `mark_quarry`, `snare_wire`, `focus`, `flank_cut`, `backflip`, `soot_feint`, `tend_wounds`, `cinder_spark`, `choke_cloud`.
  - **Weapon Mastery `1–3` / Masterwork `4th` / Cold Steel Keystones (`52` in `skills/*.png`)**:
    - `polearm`: `hook_drag`, `pike_skew`, `check_advance`, `halberd_sweep`, `polearm_cordon`
    - `greatsword`: `wide_cleave`, `rending_arc`, `gallow_sweep`, `gsword_sever`
    - `sword`: `pommel_check`, `bind_parry`, `hookbill_draw`, `sword_duelist`
    - `hammer`: `earth_shaker`, `skull_ring`, `slag_driver`, `hammer_shatter`
    - `gauntlet`: `knuckle_jolt`, `foundry_drive`, `gauntlet_breaker`
    - `fist`: `palm_check`, `liver_hook`, `rapid_combo`, `fist_tempest`
    - `crossbow`: `bodkin_punch`, `crossbow_siege`
    - `bow`: `wing_shot`, `bow_rain`
    - `knife`: `hamstring_cut`, `gut_hook`, `knife_sever`
    - `dagger`: `bodkin_twist`, `dagger_shadow`
    - `censer`: `chime_swing`, `anoint_flare`, `censer_pyre`
    - `bone_saw`: `amputate_cut`, `nerve_nick`, `rib_spread`, `vivisect`, `saw_dissect`
    - `scourge`: `snap_lash`, `hook_snatch`, `flay_arc`, `penance_bite`, `scourge_flay`
    - `sprayer`: `caustic_spritz`, `pitch_cone`, `pressure_stream`, `flash_nozzle`, `sprayer_deluge`
    - `staff`: `staff_cataclysm`
  - **`PK1`/`PK2` Slab Graft Actives (`18` in `skills/*.png`)**: `marrow_lash`, `marrow_snare`, `marrow_shriek`, `marrow_spray`, `marrow_bolt`, `marrow_rend`, `marrow_bulwark`, `marrow_pummel`, `marrow_sight`, `marrow_step`, `marrow_litany`, `marrow_caustic`, `ks_iron_marrow`, `ks_bone_eruption`, `ks_marrow_needle`, `ks_nerve_sever`, `purge_marrow`, `ks_geode_nova`.
  - **Items, Materials & Graft Pins (`21` in `games/emberwake/assets/items/<folder>/<id>.png`)**: `gauntlet/riveted_gauntlets`, `material/sinew_braid`, `material/ossified_bellows`, and `18` `graft/*` pins (`graft_optic`, `graft_sinew`, `graft_bellows`, `graft_bezoar`, `graft_valve`, `graft_core`, `graft_war_ritual`, `graft_bra_ritual`, `graft_rng_ritual`, `graft_rog_ritual`, `graft_litany`, `graft_mag_ritual`, `graft_ks_war`, `graft_ks_bra`, `graft_ks_rng`, `graft_ks_rog`, `graft_purge`, `graft_ks_mag`).

## 8. Wave 2 `AI-15` Provisional Placeholders (2026-11-16, `PR6-3`..`PR6-5`)

- **Wave 2 `R5-Finish` `[PLACEHOLDER]` Art Pack (`29` PNGs, `0` pre-existing assets modified)**: Generated deterministically (`96x96` RGBA octagonal dark-slate items; `240x240` RGBA from `120x120` 2x nearest-neighbor Cohort portraits matching `warrior_1h_buckler.png`; zero `cargo` / paid API keys). Pending Flow replacement alongside `AI-14`:
  - **`PR6-3` Consumable (`1` in `games/emberwake/assets/items/consumable/*.png`)**: `smelling_salts.png`.
  - **`PR6-4` Procedural Keepsake Templates (`6` in `games/emberwake/assets/items/accessory/*.png`)**: `keepsake_tpl_warrior.png`, `keepsake_tpl_ranger.png`, `keepsake_tpl_rogue.png`, `keepsake_tpl_priest.png`, `keepsake_tpl_brawler.png`, `keepsake_tpl_mage.png`.
  - **`PR6-5` `MASTERY_BOOK` Manuals (`16` in `games/emberwake/assets/items/manual/*.png`)**: `base.png` + `book_polearm.png`, `book_greatsword.png`, `book_sword.png`, `book_hammer.png`, `book_gauntlet.png`, `book_fist.png`, `book_crossbow.png`, `book_bow.png`, `book_knife.png`, `book_dagger.png`, `book_censer.png`, `book_bone_saw.png`, `book_scourge.png`, `book_sprayer.png`, `book_staff.png`.
  - **`PR6-4` Unsealed Cohort Portraits (`6` visored/cowled `240x240` RGBA busts in `games/emberwake/assets/portraits/common/*.png`)**: `warrior_cohort_levy.png`, `ranger_cohort_bolt.png`, `rogue_cohort_flue.png`, `priest_cohort_novice.png`, `brawler_cohort_breaker.png`, `mage_cohort_vat.png`.
