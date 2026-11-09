# ART_UI_STUDIO: Dedicated Artwork, Animation & Mock UI Session Reference

Status: active (user 2026-11-09: Dedicated Artwork/UI Session split & Design Session lock)
Skill: `art-ui-session` (`.agents/skills/art-ui-session/SKILL.md`, `.claude/skills/art-ui-session/SKILL.md`)
Agent: `art-ui-studio` (`.claude/agents/art-ui-studio.md`)
Depends on: [SCREEN_ATLAS.md](SCREEN_ATLAS.md), [CAST1_ASSETS.md](CAST1_ASSETS.md), [ART_PROMPT_TEMPLATES.md](ART_PROMPT_TEMPLATES.md), [ui.md](../preferences/ui.md), [assets-world.md](../preferences/assets-world.md)

## 1. Purpose & Session Separation (Design Session Lock)

To let Tushar iterate on **Artwork, Character Sprite Animations, VN Stage Choreography, and Screen Layouts/UI** without touching or blocking the **Design Session** (`docs/READY.md`, system specs) or **Build Session** (`docs/FEEDBACK.md`, Rust crates), all visual/UI prototyping runs in the dedicated **Artwork/UI Session** (`/art-ui-session`):

| Session | Scope & Owned Paths | Explicitly Locked / Forbidden Paths |
| --- | --- | --- |
| **Artwork/UI Session (`/art-ui-session`)** | `tools/mock_ui/**` (including `tools/mock_ui/anim_lab/**`), `art_src/**` (including `art_src/vn_stage/**`), `games/emberwake/assets/**`, `kits/rpg/assets/**`, `engine/assets/**`, `games/emberwake/docs/specs/ART_UI_STUDIO.md`, `games/emberwake/docs/specs/SCREEN_ATLAS.md`, `games/emberwake/docs/specs/CAST1_ASSETS.md`, `games/emberwake/docs/preferences/{ui,assets-world}.md` | **Never edit**: `docs/READY.md`, `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, or Rust code/data in `engine/crates/**`, `kits/rpg/crates/**`, `games/emberwake/crates/**`, `games/emberwake/data/**`. |
| **Design Session (`/design-session`)** | `docs/READY.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/DECISIONS.md`, `docs/TASKS.md`, `/audit-build` | **Locked out of** live Mock UI rigging (`tools/mock_ui/**`) and sprite/animation iteration (`art_src/vn_stage/**`) unless importing a signed-off UI/art spec into `READY.md`. |
| **Build Session (`/build-session`)** | `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md`, Rust crates (`engine/`, `kits/rpg/`, `games/emberwake/`), `games/emberwake/data/` | **Never edit** `docs/READY.md`, specs, or `tools/mock_ui/**`. |

---

## 2. Mandatory Artwork & Animation Iteration Rules

Every `/art-ui-session` must follow these four standing rules established by Tushar:

1. **Lightweight Standalone Preview / GIF First (Never Rig in Full Mock UI First)**:
   - When iterating on character animations, sprite poses, screen layouts, or visual effects, **do NOT wire experimental changes directly into the full 3.2 MB `emberwake_screen_flow.html` mock UI first**.
   - Always build or update a **focused, reduced standalone artifact** (`python3 -m tools.mock_ui.anim_lab` -> `kaelen_4pose_animation_lab.html`) or generate **animated GIFs** (`art_src/vn_stage/*.gif`) focused strictly on the visual/animation change.
   - Only integrate into `tools/mock_ui/` (`python3 -m tools.mock_ui` -> `emberwake_screen_flow.html`) **after** Tushar reviews and approves the standalone preview or GIF.
2. **Solid `#FF00FF` Magenta Background for Generated Sprites**:
   - Never rely on fake checkerboard "transparency" from image models. Always prompt for a **solid, flat, high-contrast bright magenta (`#FF00FF`) background** with no ground shadow and no text labels, then chroma-key and despill magenta edges in Python.
   - For multi-pose sheets or single-character keyframes, prefer **single-character `1:1` generation** referencing a clean `340×340` `#FF00FF` base image (or ensure wide `#FF00FF` gaps between figures) so adjacent weapons/limbs never overlap.
3. **Mandatory Post-Generation Visual Inspection**:
   - Immediately view every generated or edited image/strip with `view_file` before presenting it. Check against both the prompt and the reference image for:
     - **Identity & Palette Lock** (e.g., Kaelen must always have **shoulder-length golden-blonde hair**, scarred steel breastplate over cream quilted gambeson tunic, spiked brown leather bracers, olive trousers, and dark brown boots—never black hair or generic armor).
     - **Clipping, Seam Gaps, or Stray Neighbor Pixels** (verify bounding boxes and alpha masks across all frames).
4. **Layout & Look Before UX/Behavior**:
   - For any new screen or visual element, align on the **static visual look and layout** first before wiring complex UX/state behavior.

---

## 3. Chronological Log of Options Tried, User Feedback & Settled Decisions

This section records every option explored, Tushar's exact feedback, why rejected options failed, and the settled solution so no future session repeats a rejected approach.

### 3.1 Experiment 1: Character Rigging Approach (`Options A, B, C`)

| Option | Reference File (`art_src/vn_stage/`) | Description | User Feedback & Disposition |
| --- | --- | --- | --- |
| **Option A: 3-View Turnaround** | `exp1_optionA_3views_sheet.jpg` | Front, 3/4, and Side views on a single sheet. | **Rejected**: Useful for modeling, not expressive enough for tactical VN stage states. |
| **Option B: Exploded Paper-Doll Cutout** | `exp1_optionB_exploded_parts.jpg` | Disassembled torso, head, upper/lower arms, thighs, shins, and weapon on one sheet. | **Rejected as primary generation format**: Reassembling AI-exploded parts creates mismatched proportions and ugly joint seams compared to cohesive poses. |
| **Option C: Multi-Pose Stance Sheet** | `exp1_optionC_3poses_sheet.jpg` | Distinct full-body character stances (`Relaxed`, `Sneak`, `Weapons Drawn`). | **Selected (Winner)**: Tushar picked **Option C (Pose Sheet)** as the foundation, later expanded into the **4-Pose (1 Passive + 3 Active) Architecture**. |
| **Hair Drift Defect** | `exp1_rejected_black_hair_drift.jpg` | 3-pose sheet where model drifted Kaelen's hair to black. | **Rejected**: Enforced strict golden-blonde hair lock on all subsequent Kaelen prompts. |

### 3.2 Experiment 2: VN Stage Camera & Background Motion (`Scroll BG` & `Pan Camera`)

- **Goal**: Simulate walking and stage progression during VN story scenes using wide painted stage backdrops (`vn_alley_stage_bg.jpg`, `vn_rooftop_stage_bg.jpg`, `vn_bridge_stage_bg.jpg`, `vn_hearth_stage_bg.jpg`).
- **Options Tried & User Feedback**:
  1. *Initial take*: Characters bounced in place or entered from arbitrary sides while the camera panned.
  2. *Tushar's Feedback & Settled Rule*:
     - In both `🔄 Scroll BG` (party walks in place while background scrolls left) and `🎥 Pan Camera` (camera pans across a wide `148%` stage following the party), **all moving party units enter from the LEFT and leave to the RIGHT**.
     - When the party is supposed to **encounter someone** (e.g., a guard watching a gate while sneaking by, or meeting an NPC waiting ahead), the encountered unit starts **outside the frame on the RIGHT and stays static relative to the background** (`worldX` locked to the backdrop), scrolling naturally into view as the party advances and stopping at encounter distance for dialogue.

### 3.3 Experiment 3: Locomotion Leg Rigging & The 4-Pose (`1 Passive + 3 Active`) Split

- **Iteration 1 (Left Foot Facing Front)**:
  - *Problem*: In Kaelen's original `Relaxed` and `Sneak` 3/4 poses, his left foot pointed straight toward the camera (`front`), while his right foot pointed right (into the walk direction). When walking right, the left foot looked dragged sideways.
  - *User Feedback*: Change the left foot direction when initiating a walk; if needed, mirror/clone the forward-pointing right boot onto the left foot. (For `Weapons Drawn`, keep as-is since it reads as combat strafing and is rarely used for long walks.)
- **Iteration 2 (Lower-Only Leg Cut vs. Upper Thigh Pose)**:
  - *Problem*: Cutting only below the knee (`y >= 262`) disconnected the lower leg from the upper thigh; cutting at the tunic hem (`y >= 206`) on `Relaxed` still felt off during walking because the upper thigh was drawn rested back and the torso faced the camera rather than the direction of travel.
  - *User Feedback & Settled Architecture*: `Sneak` felt natural because Kaelen's body leaned/looked into the direction of movement. We therefore established the **4-Pose Passive + Active Architecture**:
    1. **`1 Passive Pose` (`passive_relaxed`)**: Non-active neutral stop frame for standing idle and dialogue (billhook grounded vertically at his side).
    2. **`3 Active Poses`**:
       - **`active_walk` (`1H Travel Carry` & `2H Patrol Carry`)**: Upright torso, breastplate, shoulders, and head turned right into the walk direction, polearm carried off the floor (`kaelen_active_walk_1h_carry.jpg`, `kaelen_active_walk_2h_carry.jpg`).
       - **`active_sneak` (`Crouched Stealth`)**: Low crouched posture facing right with careful stealth movement (`kaelen_4poses_passive_active_sheet.jpg` pose 2).
       - **`active_drawn` (`Weapons Drawn`)**: Wide braced combat stance (`kaelen_4poses_passive_active_sheet.jpg` pose 3).
- **Iteration 3 (Scissor Leg Crossover, Layer Swapping, Tilted Feet & Stop Bounce)**:
  - *Problem*:
    1. Shearing the legs across each other (`±44px`) and swapping `legR` between front and back layers mid-stride caused visual popping and weird thigh crossover.
    2. In the raw generated `active_walk` sprites (`kaelen_active_walk_1h_carry.jpg`), the back heel was raised high and the front toe was cocked up with stiff straight legs, so the feet never landed flat on the ground.
    3. Transitioning from `Active Walk` (`head top y = 52`) to `Passive Relaxed` (`head top y = 35`) with a stop-squash caused a bouncy shrink-then-enlarge jump.
  - *User Feedback & Settled Fixes*:
    - **Flat-Planted Boots**: Grafted flat-on-the-ground, forward-pointing boots and natural upright trousers (`y = 224..324`) under the `Active Walk` upper bodies (`kaelen_4pose_complete_preview.png`).
    - **Zero Crossover & Fixed Layer Order**: `legL` stays permanently on the back layer and `legR` stays permanently on the front layer; each leg oscillates in its own stride zone without ever crossing over the other.
    - **Exact Height Match (`y = 35..324`)**: Scaled `Active Walk (1H & 2H)` upper bodies so head-to-boot height (`289px`, `y = 35..324`) matches `Passive Relaxed` to the pixel, eliminating the stop-transition bounce.

### 3.4 Experiment 4: `Active Drawn` Facial Expression, Weapon Actions & Keyframe Generation

- **Calm Stoic Face (Threat via Weapon Action, Not Facial Grimace)**:
  - *User Feedback*: Tone down the facial aggression in `Weapons Drawn` (removed the snarling bared teeth in `kaelen_4poses_passive_active_sheet.jpg`). Threat is communicated through **weapon action** (`Poke`, `Cleave`, `Brandish`), while **Expression FX Overlays** (`💢 Threat Vein + Dark Aura`, `✨ Cold Steel Tip Glint`, `❗ Combat Alert`, `💧 Tense Sweat`) handle emotional beats.
- **Cold Steel Weapon Glint Rule**:
  - *Problem*: Initial glint was a large (`±14px`) cross sliding along the shaft in a continuous loop.
  - *User Feedback & Fix*: Limit the weapon glint strictly to the **weapon tip** `(274, 114)` (never sliding along the shaft) and reduce its size (`±5.5px` star) as a **short one-time flash (`~0.4s`)**.
- **Cohesive Body Follow vs. Static Body**:
  - *Problem*: Keeping `drawn_core_body` 100% static while moving `drawn_weapon` caused the shoulder/elbow joints to stretch and disconnect from the torso.
  - *User Feedback & Fix*: Don't keep the body 100% static. Keep the **boots planted at `y = 324`** while the **hips (`(168, 214)`) shear and the torso + attached upper sleeves lean/coil into the strike** (`-7.5°` on windup, `+11°` on cleave, `+11px` lunge on poke) so joints never stretch or detach.
- **Directly Generated Keyframes for Active Poses (`Engine A — Recommended`)**:
  - *User Proposal*: If procedural rigging through code is too finicky for complex active transitions, directly generate the animation frames/GIFs for active poses.
  - *Implemented Keyframe Pipeline*:
    1. **4-Frame Generated Walk Cycle (`walk_1h_kf0..3`, `walk_2h_kf0..3`)**: Extracted the 4 hand-drawn walking leg steps (`Contact A -> Passing A -> Contact B -> Passing B`) from `art_src/vn_stage/kaelen_walk_4frame_sheet.jpg`, leveled the contact front soles flat onto `y = 324`, and locked them under the height-matched (`y = 35..324`) `1H` and `2H` upper bodies.
    2. **Generated Sneak Cycle (`active_sneak` ↔ `sneak_kf_passing`)**: Extracted the crouched passing step from `art_src/vn_stage/kaelen_sneak_4frame_sheet.jpg` and paired it with the crouched contact stride under the locked sneak upper body.
    3. **4 Single-Character `1:1` Drawn Combat Keyframes (`active_drawn`, `drawn_kf_windup`, `drawn_kf_poke`, `drawn_kf_cleave`)**: Generated directly from the single-character `#FF00FF` reference (`art_src/vn_stage/kaelen_drawn_windup_keyframe.jpg`, `kaelen_drawn_poke_keyframe.jpg`, `kaelen_drawn_cleave_keyframe.jpg`) with identical planted boots (`x = [82..261]`).

### 3.5 Experiment 5: Passive stop frames vs active keyframes with code in-betweens (in review, 2026-11-09)

- **Question (Tushar)**: passive poses are stop frames with minimal movement; can active poses be generated as keyframes and filled with in-between frames by code or local generation?
- **Tool**: `python3 -m tools.mock_ui.anim_lab.tween --out <path> [--gif-dir <dir>]` builds a side-by-side lab (`kaelen_tween_lab.html`, about 3 MB) from the aligned keys in `kaelen_4pose_uris.json` and the weapon landmarks in `tools/mock_ui/anim_lab/tween_landmarks.json`. Every frame is snapped to one shared palette with a hard alpha edge and a magenta despill. Methods:
  - `keys`: the generated keyframes only, hard cuts, held for the same total time.
  - `flow`: dense optical flow (OpenCV DIS) between neighbouring keys; each in-between takes the nearer key's warped pixels, filling only 2 px cracks from the other key (a full fill grew a third leg in the walk).
  - `hybrid`: the halberd is lifted out of each key (shaft line fitted to the painted shaft, band ±5.5 px, inpainted), the body is in-betweened by flow, and one canonical halberd (from `active_drawn`) is drawn back as a rigid prop placed by the front-hand grip, its angle interpolated; the hands are laid back over the shaft.
  - `code`: the passive stop frame moved by code only (a 2 px breath over 3.6 s above the shins).
- **Findings so far (agent, awaiting Tushar's review)**:
  - The generated keys disagree on the weapon itself: the halberd's length (214 to 263 px) and blade shape change from key to key, so any in-between morphs the blade. A rigid prop layer fixes this and is the strongest result (poke clip).
  - Flow in-betweens work when the moving part travels a short way (guard, wind-up, poke, recover). They fail when a limb travels far (the cleave's front hand, about 90 px) or when legs swap front and back (walk passing frames break up).
  - Large pose changes (passive to drawn) need a generated breakdown key or local video in-betweening (Wan 2.2 first-last-frame on the PC, untested); code alone doubles the weapon.
- **Masters**: the keyframe masters are JPEGs; their magenta fringe survives keying. Generate future keys as PNG.
- **Breakdown keys through the Drive queue (2026-11-09)**: references are the aligned lab frames at 340 px on `#FF00FF` with a shared palette, rebuilt byte for byte from `kaelen_4pose_uris.json` by `python3 -m tools.mock_ui.anim_lab.tween --export-refs <dir>` (no binary in git: the cloud session's LFS push is refused); prompts in `art_src/vn_stage/prompts/`. `kaelen_drawn_kf_midswing` (wind-up to cleave) was queued on Drive from the cloud session; `kaelen_drawn_kf_halfdraw` (relaxed to drawn) is queued from the PC with `artqueue.py text`, which copies its references into `refs/`. A cloud session can write small text files to Drive reliably, but relaying binary images by hand as base64 corrupted one upload, so references are exported locally and queued with `artqueue.py`.

---

## 4. Screen-by-Screen UI Decisions Summary (`emberwake_screen_flow.html`)

| Screen | Key Settled UI / Visual Decisions | Mock UI Module (`tools/mock_ui/`) |
| --- | --- | --- |
| **1. Main Menu (`MainMenu`)** | Atmospheric Sunk Ward mist backdrop (`title_sunk_ward_mist_v2.jpg`, no random arch flames); 60fps floating ember/ash canvas; text-only menu column (`Continue` lit on open; `peek_room` uses corner-only `ornate` frame and appears on hover/focus). | `js/03_main_menu_and_slots.js` |
| **2. Chronicle Slots (`Load/Save`)** | 2-click Load/Save Chronicle Slots modal; `New Chronicle` boots directly into Sunk Ward (`sunk_ward`). | `js/03_main_menu_and_slots.js` |
| **3. Settings Modal (`Settings`)** | Staged Display Mode / Resolution with 15s revert confirmation popup and unapplied-changes close guard (`requestCloseSettings`). | `js/04_settings.js` |
| **4. VN Story Stage (`StoryStage`)** | Full-body standing sprites on widescreen backdrops; bottom ornate dialogue dock (`TalkBox`); camera modes (`Static`, `🔄 Scroll BG`, `🎥 Pan Camera`) with `Enter Left -> Static-on-BG Right Encounter -> Leave Right` choreography. | `js/05_story_and_region.js` |
| **5. Hub & Region (`RegionScreen`)** | Top `hud_bar` (Scrap, Tallow, Alert tier chip, Red Tithing deadline chip); collapsible `"Here (N)"` legend; `BoardPane` (Notice Board) & `ShopPane` (Aline & Bram). | `js/05_story_and_region.js` |
| **6. Party & Gear (`PartyScreen`)** | 6 Founder tabs (`Q/E`), `Active (4 of 6)` bunk toggle, 5-pip visual Belt Pouch (`<= 5` discrete cap uses visual pips, not `"3/5"` text), `Wounded` badge. | `js/05_story_and_region.js` |
| **7. Tactical Battle (`BattleScreen`)** | Continuous color-coded 6-beat timeline blocks; icon-only `SkillMenu` beside clicked target (1 row per intent, no duplicate block/AP numbers on rows); `RING`/`ARC` stage overlays; `T` full timeline inspector. | `js/06_battle.js` |

---

## 5. Complete Reference Asset & Tooling Index

### 5.1 Repository Tools for the Artwork/UI Session

```bash
# 1. Build the lightweight standalone 60fps Kaelen 4-Pose & Keyframe Animation Lab (590 KB..1.7 MB)
#    Use this FIRST when iterating on character poses, walk/sneak cycles, or weapon actions!
python3 -m tools.mock_ui.anim_lab --out /tmp/kaelen_4pose_animation_lab.html

# 1b. Build the keyframe in-between comparison lab (keys vs optical flow vs rigid-weapon hybrid, about 3 MB)
#     (pip install opencv-python-headless once; add --gif-dir art_src/vn_stage for one GIF per clip and method)
python3 -m tools.mock_ui.anim_lab.tween --out /tmp/kaelen_tween_lab.html

# 2. Build the full playable 1280x720 Emberwake Screen Flow Mock UI (3.2 MB)
#    Use this ONLY after standalone animation/layout changes are signed off by the user!
python3 -m tools.mock_ui --out /tmp/emberwake_screen_flow.html

# 3. Force-rebuild the Mock UI asset bundle from games/emberwake/assets, kits/rpg/assets, and engine/assets
python3 -m tools.mock_ui --rebuild-assets

# 4. Run docs & markdown lint check before committing Artwork/UI docs
bash tools/fc.sh docs
```

### 5.2 Reference Images, Keyframe Sheets & Animated GIFs (`art_src/vn_stage/`)

| File in `art_src/vn_stage/` | Contents & Role |
| --- | --- |
| `kaelen_generated_keyframes_preview.gif` | **Primary Animated GIF**: 4-panel animated preview of Directly Generated Keyframes (`4-Frame Walk`, `Sneak Cycle`, `Drawn Polearm Poke`, `Drawn Billhook Cleave + Tip Glint`). |
| `kaelen_procedural_rig_preview.gif` | **Procedural Rig Animated GIF**: 4-panel animated preview of the non-crossing flat-boot shear walk/sneak and 3-layer cohesive hip+torso+weapon rig. |
| `kaelen_generated_keyframes_sheet.png` | **Aligned `340×340` Keyframe Atlas**: Row 1 = 4-Frame Walk Cycle (`Contact A`, `Passing A`, `Contact B`, `Passing B`); Row 2 = 4 Drawn Combat Keyframes (`Guard`, `High Windup`, `Polearm Poke`, `Billhook Cleave`). |
| `kaelen_4pose_complete_preview.png` | **Aligned `340×340` 5-Variant Pose Sheet**: `1. Passive Relaxed`, `2A. Active Walk (1H Carry)`, `2B. Active Walk (2H Carry)`, `3. Active Sneak`, `4. Active Drawn (Calm Face)`. |
| `kaelen_4poses_passive_active_sheet.jpg` | Raw `#FF00FF` magenta 4-pose sheet with calm stoic closed-mouth `Active Drawn` face. |
| `kaelen_active_walk_1h_carry.jpg` | Raw `#FF00FF` magenta `Active Walk (1H Travel Carry)` sprite facing right. |
| `kaelen_active_walk_2h_carry.jpg` | Raw `#FF00FF` magenta `Active Walk (2H Patrol Carry)` sprite facing right. |
| `kaelen_drawn_steady_body.jpg` | Raw `#FF00FF` magenta unoccluded `Active Drawn` core body (scarred breastplate, belt, tunic skirt, wide braced legs). |
| `kaelen_walk_4frame_sheet.jpg` | Raw `#FF00FF` magenta 4-frame `Active Walk` cycle sprite sheet (`Contact -> Passing A -> Contact -> Passing B`). |
| `kaelen_sneak_4frame_sheet.jpg` | Raw `#FF00FF` magenta `Active Sneak` cycle sprite sheet (`Crouched Passing` & `Crouched Contact`). |
| `kaelen_drawn_windup_keyframe.jpg` | Raw `#FF00FF` magenta single-character `Active Drawn` High Windup / Threat Brandish keyframe (`48°` raised billhook). |
| `kaelen_drawn_poke_keyframe.jpg` | Raw `#FF00FF` magenta single-character `Active Drawn` Polearm Poke / Thrust keyframe. |
| `kaelen_drawn_cleave_keyframe.jpg` | Raw `#FF00FF` magenta single-character `Active Drawn` Downward Billhook Cleave keyframe (`-15°` low slash). |
| `exp1_optionA_3views_sheet.jpg` | Historical Option A (3-View Turnaround) for reference. |
| `exp1_optionB_exploded_parts.jpg` | Historical Option B (Exploded Paper-Doll Parts) for reference. |
| `exp1_optionC_3poses_sheet.jpg` | Historical Option C (Initial 3-Pose Sheet, selected by user). |
| `exp1_rejected_black_hair_drift.jpg` | Historical rejected take showing black-hair drift (kept as negative example). |
| `title_sunk_ward_mist_v2.jpg` | Signed-off Sunk Ward Main Menu mist backdrop (no arch flames). |
| `vn_{alley,rooftop,bridge,hearth}_stage_bg.jpg` | Signed-off widescreen VN Story Stage backdrops (`Alley`, `Rooftop`, `Canal Bridge`, `Tallow Hearth`). |
| `vn_{kaelen,vesper,maren,aline,vane,bailiff}_fullbody.jpg` | Full-body standing VN cast sprites (`Kaelen`, `Vesper`, `Maren`, `Sister Aline`, `Captain Vane`, `Castellan Bailiff`). |
| `tools/mock_ui/anim_lab/kaelen_4pose_uris.json` | Aligned `340×340` base64 PNG data URI bundle containing all passive/active poses, procedural layers (`drawn_legs`, `drawn_torso`, `drawn_weapon`), and generated keyframes (`walk_1h_kf0..3`, `walk_2h_kf0..3`, `sneak_kf_passing`, `drawn_kf_{windup,poke,cleave}`). |
