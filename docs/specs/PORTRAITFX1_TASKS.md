# PORTRAITFX1 tasks: checks, assets, rows, sequencing, docs delta

Status: proposed (spec PORTRAITFX1, 2026-10-25). Companion of [PORTRAITFX1.md](PORTRAITFX1.md); section numbers in
the Rule columns point there.

## 1. Acceptance checks (rule-derived; each task's builder writes these tests)

### 1.1 PORTRAITFX1-E (engine: unit tests in `aegis_core/src/assets.rs`; `aegis_ui/tests/portrait_fx.rs` new)

| Id | Check | Rule |
| --- | --- | --- |
| E1 | The sidecar example of 4.1 parses; each rule broken once (an eye box outside the inner rect, three eye boxes, `eye_glow` idle with no eyes, `veins` idle with no root, an unknown key, a bad colour, `border` above `size / 8`) gives its own message; `size` 200 on a 240 px PNG gives the scale warning; `face_file("portraits/npcs/tess.png")` is `portraits/npcs/tess.face.json` | 4.1 |
| E2 | `Art::first_rel` returns the first chain path a layer holds; `Art::face` reads the sidecar from that file's layer, caches it, and gives None without one | 4.1 |
| E3 | The same key and `now` give identical pixels twice; two keys give different blink times; the seed is FNV-1a 64 of the key | 4.5 |
| E4 | Breath: at `t` = period / 2 an inner top-row pixel sits `amp` device px higher; the inner bottom row and every border pixel are unchanged; none under `still`, in the reduced tier or under reduce motion | 4.2, 4.6, 4.7 |
| E5 | Rim: a pixel of the key colour changes value over time within 1 +- 0.12; a grey pixel and the border never change; steady under reduce motion; no key, no flicker | 4.2, 4.7 |
| E6 | Blink: the first starts at `2600 + 3400 h(200)` ms; in the closed phase every eye-box pixel is the lid colour (sampled 1 px above the box when `lid` is ""); a seed with `h(300) < 0.12` blinks twice 220 ms apart; none with `blink` false or no eyes; still blinks under reduce motion | 4.2, 4.7 |
| E7 | Overlays: `glow` with no anchor draws at (0.5, 0.7) of the inner rect; `veins` draws nothing at its start and its full length at 1800 ms, and no pixel beyond `len` from the root; an `eye_glow` pixel shows during a closed blink; `ash` draws min(32, round(18 i)) specks; `ash` and `smoke` draw nothing under reduce motion | 4.2, 4.3, 4.7 |
| E8 | States: `desaturate` 1.0 turns a pixel into its luminance; `tint` follows the formula of 4.2; `fade` 1.0 leaves alpha 0.3; a held cue is 0 at its hold, full at 250 ms, and 0 again 250 ms after it leaves the set | 4.2, 4.4 |
| E9 | Reactions: `flash` alpha is 200 i at its start and 0 at `ms`; a `jolt` `hit` offset never exceeds 2 px, is (0, 0) after 180 ms and never moves y; a weaker jolt does not replace a stronger running one; a 9 px jolt is clamped to 4; no jolt under reduce motion | 4.2, 4.4, 4.7 |
| E10 | Combination: holds of `desaturate` 0.3 and 0.5 give 0.5; two `face` cues: the higher priority wins, equal priority the later one; `expression` is "" with no `face` cue | 4.4 |
| E11 | Tiers: sides 96, 64 and 40 device px give full, reduced and minimal with exactly the effects of 4.6 | 4.6 |
| E12 | Performance: a portrait with nothing time-varying composes once over 100 frames; an idle portrait composes at most 30 times over 1000 clock ms drawn at 144 frames; a 400 px rect composes at 256; with `Budget` the 9th animated portrait of a frame does not recompose | 4.5, 4.8 |
| E13 | Order: no effect but `jolt` changes a border pixel; `flash` covers an overlay pixel; `fade` multiplies overlay pixels too | 4.3 |
| E14 | `Cue::parse` gives None for an unknown effect or jolt preset; `EFFECTS` and `PORTRAIT_JOLTS` hold the names of 4.2 in that order | 4.2, 4.4 |
| E15 | Kit widgets: the Portraits page has every control of 4.10; `kit.portrait.jolt.crit` moves a portrait pixel and no pixel outside the portrait rect; size 16 shows tier minimal in the foot line; `state` lists the live effects | 4.10, 4.11 |

### 1.2 PORTRAITFX1-S (kit: scratch-data tests in `rpgkit_content/tests/portrait_cues.rs` and `rpgkit_battle/tests/portrait.rs`)

| Id | Check | Rule |
| --- | --- | --- |
| S1 | Each validation of 5.1 refuses its bad row (missing `event` on an event row, `status` not in the table, `below` on a `down` row, both `below` and `drop_pct`, `ms` 0 on an event `flash`, `ms` above 0 on a state row, `color` on a `fade`, `preset` missing on a `jolt`, an `expression` outside `EXPRESSIONS`, `surfaces` outside the four); `trigger` `meter` gives "unit meters: CAMPAIGN1-DOC"; a `condition` that is not a kit condition warns; two rows with the same trigger, match and effect are errors on both | 5.1 |
| S2 | The loader reads `portrait_cues.json` beside `CATALOG_KINDS`; `aegis content upsert portrait_cues` writes a row with only its non-default keys; load, write and load again give the same rows | 5.1 |
| S3 | `holds`: a status row matches only with the status; HP 49 of 100 matches `below` 50 and HP 50 does not; `max_hp` 0 matches no `hp` row; `drop_pct` 25 matches 75 against 100 and not 76; `derived` `below` 20 matches 19; a row with `surfaces` `["panel"]` is left out on `dialogue`; a row with `rulesets` `["timeline"]` is left out under `classic` | 5.2 |
| S4 | `reactions` returns the event rows in event order, then row order | 5.2 |
| S5 | `step_events`: HP fell with a " CRIT" line gives `crit` only; guarded gives `guard_hit`; HP rose gives `heal`; a `[Break]` line gives `break`; a unit untouched gives nothing; the session's value before and after the call is equal | 5.3 |
| S6 | `face_chain`: hero `kaelen`, `hurt` gives `heroes/faces/kaelen/hurt.png`, `heroes/faces/kaelen/neutral.png`, then `portrait_chain`; "" gives the `neutral` frame then `portrait_chain`; a common tier gives `portrait_chain` only; `face_sidecars` of a frame lists its own sidecar, then the base's | 5.4 |
| S7 | A11 and A12 on scratch asset layers: a bad sidecar, a sidecar without its PNG, a frame named `smirk`, a faces folder for an id without a portrait, a frame without a sidecar under a base with eyes: each warns once with its text | 5.5 |
| S8 | The gate's parity suites pass unchanged (`trace_parity`, `phasefold`, `checks_parity`, `data_source`) | 8 |

### 1.3 PORTRAITFX1-T (tools: `tools/portraits/face.py`; fixtures `tools/portraits/fixtures/face/`)

Commands (standard library and Pillow only, portable to Rust):

- `face.py grid PNG OUT`: the portrait at 4x with a line every 8 image px and a label every 40, so coordinates read off.
- `face.py set PNG [--eyes x,y,w,h ...] [--no-blink] [--lid #rrggbb] [--glow x,y,r ...] [--veins x,y,deg,len ...] [--rim #rrggbb] [--border n] [--idle effect[:#rrggbb[:intensity]] ...]`: writes the sidecar beside the PNG, `size` from its width.
- `face.py show PNG OUT`: the anchors drawn over the 4x portrait.
- `face.py check [ROOT ...]`: the A11 rules over every sidecar.
- `face.py contact OUT [PNG ...]`: every portrait with its anchors on one review sheet.

| Id | Check | Rule |
| --- | --- | --- |
| T1 | `set` writes a sidecar that the engine parses; a box outside the inner rect is refused with A11's text and writes nothing | 4.1 |
| T2 | `check` prints the same messages as the kit's A11 on the fixture set (the expected list is committed beside the fixtures; the kit test S7 asserts the same list) | 5.5 |
| T3 | `grid`, `show` and `contact` write images 4x the portrait size with the anchors in the right image px | - |

### 1.4 PORTRAITFX1-G (game: unit tests in `portraits.rs` and `gallery.rs`)

| Id | Check | Rule |
| --- | --- | --- |
| G1 | `look_of_unit` carries the unit's statuses and its member's conditions; `look_of_member` carries the saved HP and no statuses; `look_of_character` has no HP (no `hp` or `down` row matches) | 5.2, 6.1 |
| G2 | A character whose sidecar holds `eye_glow` draws it with no cue rows at all | 6.1 |
| G3 | A character with no portrait file draws today's initial letter | 6.1 |
| G4 | Under `skip_anim` `react` fires nothing; state holds still apply | 6.1 |
| G5 | The kit's copy of `EFFECTS` equals `aegis_ui::portrait_fx::EFFECTS` | 5.1 |
| G6 | The Gallery page lists every portrait PNG of the game and kit layers (42 today) and one button per cue row; pressing an event row's button changes a portrait pixel; a state row toggles | 6.3 |
| G7 | A sidecar with `rim` "" flickers on Tallow Amber `#d97706` pixels | 6.4 |

### 1.5 PORTRAITFX1-D (engine `aegis_ui/tests/vignette.rs`; game `region.rs` tests)

| Id | Check | Rule |
| --- | --- | --- |
| D1 | The talk box portrait changes pixels over 4 s by default (breath, rim); under reduce motion only blink frames differ | 4.7, 6.2 |
| D2 | A speaker who is a wounded party member draws `desaturate` 0.35 (with A's rows); a world character never matches an `hp` row | 5.2, 6.2 |
| D3 | A vignette `line` card draws through `PortraitFx` with its sidecar's `idle`; a card with no portrait file draws today's empty frame; a skipped reel draws no portrait | 6.2 |
| D4 | The TEXTFIT1 audit of the talk box and a line card is clean at every text size | 6.2 |
| D5 | The region map's `state` lists the speaker's portrait while a conversation is open | 4.11 |

### 1.6 PORTRAITFX1-R, -P, -TL (game; `party.rs`, `hub.rs`, `battle.rs` tests; engine `timeline` tests)

| Id | Check | Rule |
| --- | --- | --- |
| R1 | The Character screen portrait is 48 px inside the frame and overlaps no widget of the screen (a rect test over the frame's widgets); the hub list rows draw 20 px portraits and the names fit 22 px shorter | 6.2 |
| R2 | A wounded member's portrait is desaturated on both; cycling members (Q, E) draws the new member's portrait and drops the old one's `PortraitFx` | 6.2, 4.8 |
| R3 | TEXTFIT1 clean on both screens at every text size | 6.2 |
| P1 | The hero and intel panels draw 32 px portraits at the top-left (inset 4); the lines beside them start 40 px in; TEXTFIT1 clean | 6.2 |
| P2 | A hit flashes and jolts the target's panel portrait at the impact (rule I of FX1), not at the pop; a crit uses `crit`; a guarded hit uses the guard rows; a heal glows gold | 5.3, 6.2 |
| P3 | A unit with `stealth` draws at alpha 0.58 (`fade` 0.6); below 25 % HP `desaturate` 0.5; down: `still` and `desaturate` 1.0 | 4.2, 5.2; rows of 2.3 here |
| P4 | Hit-stop pauses the portrait clock; `anim_speed` 0.5 halves its pace | 4.5 |
| P5 | Determinism: the session, the battle's RNG state and the trace after a scripted battle are equal with portraits drawn and not drawn; a replayed recording draws the same portrait pixels for the same frames | 8 |
| TL1 | `Track.icon` reserves a row-height square left of the label and pushes `<id>.icon.<key>`; tracks without it draw as before (the existing timeline tests pass); the Kit widgets Timeline page has an icons toggle | 6.5 |
| TL2 | Strip rows draw portraits in the minimal tier: no jolt, no breath; a stealthed unit's row fades and a struck unit's row flashes | 4.6, 6.2 |

### 1.7 PORTRAITFX1-X, -XT (B)

| Id | Check | Rule |
| --- | --- | --- |
| X1 | `expression` on a dialogue node and on a `line` card: "" and the four names pass, `smirk` is refused; the writer stores it only when not "" | 7 |
| X2 | A node with `angry` draws `faces/<id>/angry.png` when it exists, else the `neutral` frame, else the unique portrait | 5.4, 7 |
| X3 | A `line` card resolves its portrait chain with its `expression` (the kit's card resolution) | 7 |
| X4 | A `face` cue `hurt` for 900 ms shows the `hurt` frame and then the frame before it | 4.4 |
| X5 | The gate's parity suites pass unchanged | 8 |
| XT1 | `generate.py` reads a prompts block with a `reference:` line, sends that image, writes `art_src/<id>_faces_gen<time>_<n>.png` and its record (`"kind": "faces"`, the reference, the full prompt) | 7 |
| XT2 | `sheets.py faces <sheet> <id> <tier> [--base] [--crop l,t,r,b] [--contact out.png]` clears the mark on the whole sheet, cuts four equal quadrants, runs each through `pixelize` with one crop, writes four 240 px frames to `faces/<id>/`; `--base` also writes the `neutral` cell as `<id>.png`; `--contact` writes the four frames beside the current portrait | 7 |

### 1.8 Asset tasks (`tools/fc.sh content check` clean, A11 and A12 clean)

| Id | Check | Rule |
| --- | --- | --- |
| A1 | Every portrait PNG of the game and kit layers (42) has a sidecar; the anchors match the procedure of 2.1; `face.py check` is clean | 2.1 |
| A2 | The idle overlays equal table 2.2 | 2.2 |
| A3 | The cue rows equal table 2.3 | 2.3 |
| XA1 | Each image-API run was approved by the user before it ran (the planner's ask names the characters, the image count and the model) | 7 |
| XA2 | Each of the 13 has its picked frames and their sidecars; a prompt record beside every sheet in `art_src/` | 2.4 |
| XA3 | The face cue rows equal table 2.4.3; the slice's lines carry expressions by the rule of 2.4.4 | 2.4 |

## 2. Assets

### 2.1 Sidecar procedure (PORTRAITFX1-A)

1. `face.py grid` for each portrait; read the coordinates off the grid at 4x.
2. Eye boxes: each visible eye's white and iris pixels plus 1 px around, in image px; a three-quarter face's far eye
   gets its own (smaller) box; no box for an eye hidden by hair, a mask or shadow. `blink` false for gas masks,
   lidless creatures and empty sockets. `lid` "" unless the sampled pixel above the box is hair or a hood (then the
   face's skin-light hue, for humans `#a47a5f`).
3. Glow anchors and vein roots where table 2.2 places them; veins point across skin, away from the eyes.
4. `border` 8 for the 240 px portraits; `rim` "" (the game's Tallow Amber).
5. `face.py set`, then `face.py contact` of the batch; the Gallery page (6.3) is the user's review.

### 2.2 Idle overlays (sidecar `idle`; every other portrait: anchors only, no `idle`)

| Portrait (under `portraits/`) | `idle` | Anchors to place |
| --- | --- | --- |
| `elites/calcified_husk` | `veins` `#9333ea` 0.8; `eye_glow` `#c084fc` 0.5 | 2 vein roots on the cheek and the neck at the black bone; eye boxes on the empty sockets, `blink` false |
| `elites/inquisitorial_purifier` | `eye_glow` `#d97706` 0.6; `smoke` `#6b7280` 0.5 | eye boxes on the mask's lenses, `blink` false |
| `elites/ash_warchief` | `ash` `#6b7280` 0.6 | - |
| `elites/gruk_smokecaller` | `smoke` `#6b7280` 0.7 | - |
| `elites/nok_flamebow` | `ash` `#6b7280` 0.4 | - |
| `bosses/proctor_quell` | `veins` `#9333ea` 0.7; `glow` `#9333ea` 0.6 | 1 glow anchor on the neck valve (r 18); 2 vein roots beside it |
| `bosses/inquisitor_prelate_nemeth` | `eye_glow` `#c084fc` 0.6; `glow` `#9333ea` 0.4 | 1 glow anchor on the infusion (chest or neck, where the picture shows it; r 24) |
| `bosses/guild_master_orlov` | `smoke` `#6b7280` 0.4 | - |
| `bosses/grakka_ashmaw` | `ash` `#6b7280` 0.6; `eye_glow` `#e11d48` 0.5 | - |
| `common/beast_render_beast` | `eye_glow` `#e11d48` 0.7 | eye boxes; `blink` true |
| `common/crawler_ash_crawler` | `glow` `#9333ea` 0.7; `ash` `#6b7280` 0.3 | 1 glow anchor on the acid abdomen (r 30); eyes only if visible, `blink` false |
| `common/dummy_base` (kit layer) | none | no eyes, `blink` false |

Heroes (8), npcs (6, the hub cast included), the other elites and bosses (`bolg_redhand`, `skarn_ironjaw`,
`vex_cutpurse`, `captain_vane`) and the other kit commons get anchors only (PQ5 a).

### 2.3 Cue rows (`games/emberwake/data/catalog/portrait_cues.json`; empty `surfaces` and `rulesets`)

| id | trigger | match | effect | intensity | ms | preset | color |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `hit_flash` | event | `hit` | flash | 0.6 | 140 | - | `#d9cbb4` |
| `hit_jolt` | event | `hit` | jolt | 1.0 | 0 | `hit` | - |
| `crit_flash` | event | `crit` | flash | 0.9 | 180 | - | `#e11d48` |
| `crit_jolt` | event | `crit` | jolt | 1.0 | 0 | `crit` | - |
| `guard_flash` | event | `guard_hit` | flash | 0.4 | 120 | - | `#fbbf24` |
| `guard_jolt` | event | `guard_hit` | jolt | 0.5 | 0 | `hit` | - |
| `heal_glow` | event | `heal` | flash | 0.3 | 420 | - | `#fbbf24` |
| `break_flash` | event | `break` | flash | 0.9 | 120 | - | `#d9cbb4` |
| `break_jolt` | event | `break` | jolt | 1.0 | 0 | `crit` | - |
| `wounded` | condition | `wounded` | desaturate | 0.35 | 0 | - | - |
| `hp_half` | hp | `below` 50 | desaturate | 0.2 | 0 | - | - |
| `hp_low` | hp | `below` 25 | desaturate | 0.5 | 0 | - | - |
| `downed` | down | - | desaturate | 1.0 | 0 | - | - |
| `downed_still` | down | - | still | 1.0 | 0 | - | - |
| `shaken` | derived | `composure`, `drop_pct` 25 (PQ3 a; `below` 20 under b) | tint | 0.5 | 0 | - | `#06b6d4` |
| `stealthed` | status | `stealth` | fade | 0.6 | 0 | - | - |

The `stealthed` row needs RULESET1-S's `stealth` status row; `shaken` needs DERIVE1-D's `composure` (done).

### 2.4 Expression frames (PORTRAITFX1-XA)

#### 2.4.1 Run plan (asked first; one ask may cover the batch or each character, as the user prefers)

| Group | Ids | Reference | Calls |
| --- | --- | --- | --- |
| Founders (heroes) | `kaelen`, `vesper`, `maren`, `hollis`, `dunstan`, `elyse` | `art_src/mara_voss.png` as style reference (PQ1 a); `--base` writes the `neutral` cell as their portrait | 6 |
| Hub cast (npcs) | `tess`, `bram`, `aline` | `art_src/<id>.png` (ART3 picks) | 3 |
| Bosses | `captain_vane`, `proctor_quell`, `guild_master_orlov`, `inquisitor_prelate_nemeth` | `art_src/<id>.png` (ART3 picks) | 4 |

Model `gemini-3-pro-image`; 13 images, one candidate each; up to 2 re-rolls per character on the user's word. The
prompts file `games/emberwake/docs/specs/PORTRAITFX1_FACES.md` (written by XA in the ART3_PROMPTS layout: the style
block of 2.4.2, then one block per id with its `reference:` line): the character lines of the ART3 picks come from
`ART3_PROMPTS.md`; the founders' lines are written from their lore (`games/emberwake/docs/lore/systems/class_*.md`)
and the "What is off" notes of `CAST1_ASSETS.md` 1 (Maren's ink veins, Hollis's left-eye graft, Dunstan's
scrap-wired jaw, Elyse's bandolier of retorts), and shown to the user with the ask.

#### 2.4.2 Prompt (the 2x2 template of `ART_PROMPT_TEMPLATES.md` with the four cells fixed)

```text
Use the attached image as the reference: {for an own portrait: "the same person: face, hair, outfit, colours, framing, lighting and pixel-art style; only the expression changes" | for a founder: "the same pixel-art style, level of detail, framing, lighting, outline weight and flat dark background. This is a different subject, not the person in the reference."}
A 2x2 grid of four pixel art character portrait variations for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. 1:1 square image. No borders, no frame, no text, no dividers, no logo. Plain flat very dark background (#120e12) behind all figures. Each variation shows the same head-and-shoulders, three-quarter view of {name}, {character line}, the head the same size and in the same place in every cell. Each variation utilizes a 16-bit pixel art style with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per color, and light dithering on metal and cloth. Strong cool key light from the upper left and a warm amber tallow-lantern rim light (#d97706) from the right. Same outfit, hair and colours in all four.

The four variations are:
- Top-Left: Neutral: {mood line}, mouth closed, eyes looking right.
- Top-Right: Hurt: wincing in pain, one eye squeezed narrower, teeth gritted, head pulled back slightly.
- Bottom-Left: Angry: brows drawn down hard, jaw set or snarling, eyes glaring.
- Bottom-Right: Afraid: eyes wide, brows raised, mouth slightly open, head drawn in.
```

#### 2.4.3 Face cue rows (added by XA)

| id | trigger | match | effect | expression | ms | priority |
| --- | --- | --- | --- | --- | --- | --- |
| `face_hit` | event | `hit` | face | `hurt` | 900 | 60 |
| `face_crit` | event | `crit` | face | `hurt` | 1200 | 70 |
| `face_low_hp` | hp | `below` 30 | face | `hurt` | 0 | 40 |
| `face_shaken` | derived | `composure`, as `shaken` | face | `afraid` | 0 | 50 |
| `face_empowered` | status | `empower` | face | `angry` | 0 | 45 |
| `face_down` | down | - | face | `hurt` | 0 | 80 |

#### 2.4.4 Line expressions

For each slice dialogue node and `line` card whose speaker is one of the 13 and whose data exists when XA runs: a
threat, an order shouted or a curse gets `angry`; pain or a wound gets `hurt`; dread, a plea or a flinch gets
`afraid`; every other line stays "". Lines added later get theirs in their own data tasks.

## 3. Rows wanted (status `spec`; the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- | --- |
| PORTRAITFX1-E | Engine portrait effects: the face sidecar format and checks (`aegis_core::assets`), `aegis_ui::portrait_fx` (base life, overlays, states, reactions, composition, cues and holds, seeds, size tiers, reduce motion, cache and budget, agent state), `Art::face` and `Art::first_rel`, the Kit widgets Portraits page with the sample face (4; E1-E15) | engine | logic | FX1-E (done) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `engine/crates/aegis_core/src/assets.rs`, `engine/crates/aegis_ui/src/{portrait_fx (new),art,specimens,lib}.rs`, `engine/crates/aegis_ui/tests/portrait_fx.rs` (new) |
| PORTRAITFX1-S | Kit portrait cues: schema `content.portrait_cues`, typed rows, loader and writer beside `CATALOG_KINDS`, checks, `holds` and `reactions`, `step_events`, `EXPRESSIONS`, `face_chain` and `face_sidecars`, asset checks A11 and A12 (5; S1-S8) | kit | logic | PORTRAITFX1-E, DERIVE1-S (done) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `kits/rpg/schema.json`; under `kits/rpg/crates/`: `rpgkit_data/src/lib.rs`, `rpgkit_content/src/{source,store,art}.rs`, `rpgkit_content/src/check/{portrait (new),assets,mod}.rs`, `rpgkit_battle/src/{portrait (new),lib}.rs`, tests `rpgkit_content/tests/portrait_cues.rs`, `rpgkit_battle/tests/portrait.rs` (new) |
| PORTRAITFX1-T | Portrait anchor tool `tools/portraits/face.py` (grid, set, show, check, contact) and its fixtures (1.3; T1-T3) | game (tools) | logic | PORTRAITFX1-E (the format) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `tools/portraits/face.py` (new), `tools/portraits/fixtures/face/` (new) |
| PORTRAITFX1-G | Game portrait binding (`portraits.rs`: looks, `Faces`, draw and react, colours) and the game Gallery Portraits page (6.1, 6.3, 6.4; G1-G7) | game | logic | PORTRAITFX1-E, PORTRAITFX1-S | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `games/emberwake/crates/emberwake_gui/src/{portraits (new),gallery,main}.rs` |
| PORTRAITFX1-D | Surface 1: the dialogue talk box and vignette line cards draw animated portraits (6.2 row 1; D1-D5) | engine + game | logic | PORTRAITFX1-G | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `engine/crates/aegis_ui/src/vignette.rs`, `engine/crates/aegis_ui/tests/vignette.rs`, `games/emberwake/crates/emberwake_gui/src/{region,story}.rs` |
| PORTRAITFX1-A | Portrait sidecars for the 42 portraits (anchors, idle overlays) and the cue rows (2.1-2.3; A1-A3) | game (data) + kit (assets) | asset | PORTRAITFX1-S, PORTRAITFX1-T, RULESET1-S (the `stealth` row) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `games/emberwake/assets/portraits/**/*.face.json`, `kits/rpg/assets/portraits/common/*.face.json`, `games/emberwake/data/catalog/portrait_cues.json` through `tools/fc.sh content`, `games/emberwake/data/seeds/portraitfx1.json` |
| PORTRAITFX1-R | Surface 2: portraits on the Character screen and the hub party list (6.2 row 2; R1-R3) | game | logic | PORTRAITFX1-G, ROSTER1-U and DERIVE1-U (same `party.rs`, `hub.rs`) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `games/emberwake/crates/emberwake_gui/src/{party,hub}.rs` |
| PORTRAITFX1-P | Surface 3: portraits on the battle hero and intel panels, step events at FX1's impacts (6.2 row 3; P1-P5) | game | logic | PORTRAITFX1-G, FX1-L1 (the impact queue), RULESET1-U (same `battle.rs`; the stealth status) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `games/emberwake/crates/emberwake_gui/src/{battle,fx}.rs` |
| PORTRAITFX1-TL | Surface 4: the timeline widget's icon square and portraits on the strip rows (6.5, 6.2 row 4; TL1-TL2) | engine + game | logic | PORTRAITFX1-P, RULESET1-U | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `engine/crates/aegis_ui/src/{timeline,specimens}.rs`, the timeline tests, `games/emberwake/crates/emberwake_gui/src/battle.rs` |
| PORTRAITFX1-X | Expression keys (B): `expression` on dialogue nodes and `line` cards, checks, writer, the card resolution and the talk box using `face_chain` (7; X1-X5) | kit + game | logic | PORTRAITFX1-D, STORYUI1-S (the card schema), STORYUI1-W (card resolution) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `kits/rpg/schema.json`; under `kits/rpg/crates/`: `rpgkit_content/src/check/{dialogue,story}.rs`, `rpgkit_world/src/{dialogue,story}.rs`; `games/emberwake/crates/emberwake_gui/src/{region,story}.rs` |
| PORTRAITFX1-XT | Expression sheet tools (B): `generate.py` per-block `reference:` and the faces record, `sheets.py faces` (1.7; XT1-XT2) | game (tools) | logic | PORTRAITFX1-T | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `tools/portraits/{generate,sheets}.py` |
| PORTRAITFX1-XA | Expression frames for the 13 (B): the prompts file, the image-API runs (each asked first), picks, frames and sidecars, face cue rows, slice line expressions (2.4; XA1-XA3) | game (art + data) | asset | PORTRAITFX1-X, PORTRAITFX1-XT, PORTRAITFX1-A; for 2.4.4 the slice lines (STORYUI1-A, SUNKWARD1) | spec | docs/specs/PORTRAITFX1.md (+ _TASKS.md) | `games/emberwake/docs/specs/PORTRAITFX1_FACES.md` (new), `art_src/`, `games/emberwake/assets/portraits/{heroes,npcs,bosses}/`, `portrait_cues.json` and slice dialogue and vignette rows through `tools/fc.sh content` |
| PUPPET1 | Layered portrait puppets (C): head, eyes, mouth and body layers, head turns, mouth flaps; the sidecar's `mouth` box is reserved for it | engine + game | design | PORTRAITFX1-XA | idea (user 2026-10-25: later) | - | - |

The PORTRAITFX1 row in `docs/TASKS.md` becomes the spec row (status `done (spec written 2026-10-25)`, pointer
`docs/specs/PORTRAITFX1.md (+ _TASKS.md)`).

## 4. Sequencing and hub files

- Order by surface priority: E, then S (and T beside it), then G, then D (surface 1) and A (asset lane, as soon as S
  and T are merged), then R, P, TL; B after D: X and XT, then XA.
- `region.rs`, `story.rs`, `gallery.rs`: STORYUI1-U1 also edits them; D and G should not run beside it. If
  STORYUI1-U1 has not landed when D runs, D draws vignette line cards only through the engine (`vignette.rs`) and the
  `story.rs` part moves to X.
- `party.rs`, `hub.rs`: DERIVE1-U and ROSTER1-U first, then R.
- `battle.rs`: FX1-L1, FX1-L2, RULESET1-U, DERIVE1-U, CAMPAIGN1-U, STORYUI1-U1 and ENCOUNTER2-U also edit it; P and
  TL run after FX1-L1 and RULESET1-U and never beside another `battle.rs` task. P keeps its code in `portraits.rs`;
  `battle.rs` gets the two panel draws and the impact hook (in `fx.rs`).
- `vignette.rs`: FX1-V also edits it; not beside D.
- `kits/rpg/schema.json`: S and X each add rows; not beside another kit schema task (HOOKS1, RULESET1, CAMPAIGN1,
  ROSTER1, STORYUI1-S).
- `specimens.rs`: E, then TL.

## 5. Docs delta (the planner applies it with the builders' deltas)

- `docs/STRUCTURE.md`: `aegis_core/src/assets.rs` gains the face sidecar; `aegis_ui/src/portrait_fx.rs` (new);
  `timeline.rs` `Track.icon`; `specimens.rs` the Portraits page; `rpgkit_battle/src/portrait.rs`,
  `rpgkit_content/src/check/portrait.rs` (new), `art.rs` `face_chain`; `emberwake_gui/src/portraits.rs` (new);
  `data/catalog/portrait_cues.json`; the assets row: `*.face.json` sidecars and `faces/<id>/` frames; the tools row:
  `tools/portraits/face.py`.
- `engine/README.md`: a "Portrait effects (PORTRAITFX1)" bullet (sidecar, effects, tiers, reduce motion, budget, Kit
  widgets page).
- `engine/docs/PORT_NOTES.md`: `portrait_fx`: engine-only (Godot had no portrait effects).
- `engine/docs/authoring/art_bible.md` row "Portraits": add "the anchor sidecar (eyes, glow, veins) and expression
  frames; the effects follow the engine's portrait layer" (the promote-generic-methods rule).
- `games/emberwake/docs/lore/specs/art_and_visual_style.md`: a "Portraits" line in section 4 or 5: the idle life, the
  overlay hues of 6.4, the cue table, reduce motion.
- `games/emberwake/docs/GAME_BRIEF.md` where to look: portrait reactions in `portrait_cues.json`, anchors in
  `*.face.json`, the binding in `portraits.rs`.
- `art_src/README.md`: the faces sheets and their records; `ART_PROMPT_TEMPLATES.md`: a "2x2 expression sheet"
  variant pointing to 2.4.2.

## 6. HISTORY, decision and roadmap rows wanted

- ROADMAP: PORTRAITFX1, proposed (decided by the user 2026-10-25): animated, reactive portraits: engine portrait
  effects, kit cue table, sidecar anchors, four surfaces, expression frames for the main cast.
- Decision row after the user answers PQ1-PQ5 (`games/emberwake/docs/decisions/classes-units.md`, "Animated
  portraits (2)"): the founders' face sets, the expression set, the Composure threshold, the neutral cell, the
  overlay cast.
- HISTORY, if the recommendations are taken: "Portrait anchors as content fields on characters (`aegis content`)":
  not adopted (PORTRAITFX1, 2026-10-25): the anchors describe one image's pixels and a common portrait serves many
  units; sidecars beside the PNG. "Figure masks for portrait effects": deferred (no mask data; `pixelize.py` could
  write one). Under PQ1 b or c, PQ3 b or PQ4 b, the alternative not taken gets its row instead.
