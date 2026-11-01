# PORTRAITFX2: motion per face and four more faces

Status: proposed (spec 2026-11-01, from decision "Portrait motion per expression and four more faces" (user
2026-11-01) in `games/emberwake/docs/decisions/classes-units.md`, with "Expression effects at runtime" in the same file
and "Expression sheets: scale only and aligned cells" in `docs/decisions/workflow.md`)
Depends on: PORTRAITFX1-E, -EX, -X (built: `aegis_ui::portrait_fx`, `portrait_expressions`, the face cues and
`face_chain`); PORTRAITFX1-XA (the first sheets, the scale-only cut and cell alignment); FOUNDERPORT1-P (the founders'
portraits)
Lane: logic (M, F, T), asset (M rows, FA, A)
Touches: by slice, section 10
Paths: `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`.

History: no row on portrait motion or the face list. "Figure masks" (deferred 2026-10-25) holds: motion moves the
whole inner rect, so no mask is needed.

## 1. Goal

Each face moves in its own way instead of the one breath PORTRAITFX1 gives every face: neutral breathes as today, hurt
winces, angry heaves, afraid shivers, and four new faces (Determined, Happy, Sad, Surprised) lift, bounce, sink and
jolt. A motion profile is part of each face's row in data, read by one engine piece. The face list becomes the game's
data. Battle events and dialogue lines show the new faces, and the playable heroes get a second 2x2 sheet. This is
presentation only: no rule, save, sim or trace reads it.

## 2. Survey (develop at f2953438)

| Area | Where | Today |
| --- | --- | --- |
| Breath | `portrait_fx.rs` `breath_px`, `breath_amp`, `breath_period` | Stretch `amp (1 - cos(2 pi t / P)) / 2` anchored at the inner bottom, `amp = max(1, round(0.012 side))` (3 px at 240), `P = 3600 (0.9 + 0.2 h(1))`; full tier only |
| Per-face rows | `data/catalog/portrait_expressions.json`, `rpgkit_data::PortraitExpression`, `check/expression_fx.rs`, engine `ExpressionFx` | Rows `hurt`, `angry`, `afraid` (tint, mark, shake); `neutral` takes no row |
| Face list | `rpgkit_data::EXPRESSIONS` (closed, 4) | Read by `art.rs`, `check/{assets,dialogue,portrait,expression_fx}.rs`, `gallery.rs`, `region.rs`, schema help |
| Face cues | `portrait_cues.json` (XA 2.4.3), `PORTRAIT_EVENTS` (5) | Step events from HP and log lines |
| Sheets | `sheets.py` `FACE_CELLS` (fixed), `artqueue.py request --faces --prompts` | One layout |
| Hooks | encounter `deployment` `ambush` ("enemies on both sides"); skill `level` 1-4 (14 at 3 or more); member `unspent(d)` | - |

## 3. Layers

Engine: `Motion` (formulas, composition, tiers, seeds, agent state, Kit widgets samples) in `aegis_ui::portrait_fx`;
it names no face (a base `Motion` for "", one per `ExpressionFx`). Kit: the motion fields, the open face list, the new
events and checks. Game: the rows (through `aegis content`), firing the events, the listings, and the sheet tool.

## 4. Engine: `Motion` (`aegis_ui::portrait_fx`, PORTRAITFX2-M)

### 4.1 Model

Distances are in px at the reference side `MOTION_REF` 240, scaled by `k = side / 240` to whole device px:
`px(v) = sign(v) max(1, round(|v| k))` for an amplitude `v` other than 0, and each part's output is
`round(px(v) x shape)`, so a full stretch at 240 equals today's breath. `t` is the portrait clock, `u = t - since`
(the face's start, `expression_since`), `m` the motion seed `seed ^ seed_of(expression)`, `hm(k) = fx::hash01(m, k)`.

| Part | Fields | Output |
| --- | --- | --- |
| Cycle (breath, heave) | `breath_amp` A, `breath_ms` P, `breath_in` r, `breath_jitter` j | Phase `b` 0-1. Period `P' = P (0.9 + 0.2 h(1))` with the portrait seed (as today). With j = 0 the cycle runs on `t` (closed form); with j > 0 cycle n (counted from `since`) has period `P' (1 + j (2 hm(4n) - 1))` and amplitude `A (1 - j hm(4n + 1))`. Within a cycle at fraction f: `b = (1 - cos(pi f / r)) / 2` for f < r, else `(1 + cos(pi (f - r) / (1 - r))) / 2` (r = 0.5 is today's cosine). Stretch `round(px(A_n) b c)` |
| Lean | `lean_px` L | dx `round(px(L) b c)` (positive = toward the picture's right, where the faces look) |
| Bob (bounce) | `bob_px` B, `bob_ms` Q | dy `-round(px(B) abs(sin(pi t / Q)) c)` |
| Entry (wince, hop, jolt) | `enter_dy` D, `enter_stretch` S, `enter_ms` E | Envelope `e(u)`: `u / (0.25 E)` up to 0.25 E, then `(1 - (u - 0.25 E) / (0.75 E))^2`, 0 from E on. dy `round(px(D) e)`, stretch `round(px(S) e)` (negative = squeeze) |
| Rest (lift, sink) | `rest_dy` R, `rest_ms` T | dy `round(px(R) (1 - (1 - min(1, u / T))^2))` (T = 0: at once), held while the face shows |
| Settle | `settle_ms` W | Cycle scale `c = 0` while u < E, then `min(1, (u - E) / W)` (W = 0: 1). Cycle, lean and bob use `c` |
| Shiver | `shiver_px` X, `shiver_ms` M, `shiver_gap_ms` G | Burst j starts at `S_0 = round(G (0.5 + hm(1000000)))`, `S_j+1 = S_j + M + round(G (0.5 + hm(1000001 + j)))`; inside a burst, step q = floor((u - S_j) / 40): dx = `px(X) (floor(3 hm(2000000 + 64 j + 2 q)) - 1)`, dy the same at `+ 1`; 0 between bursts |
| Blink | `blink` | false stops `blink` while the face shows (hurt: the frame's eyes are shut) |
| Flush | `tint_breath` F | The face's tint strength x `(1 - F + F b)` (the flush pulses with the cycle) |

Totals: stretch = cycle + entry, clamped to `[-px(3), px(4)]`; dx and dy = the sum of their parts, each clamped to
`±px(3)`.

### 4.2 Composition, blend and anchors

- Step 2 of PORTRAITFX1 4.3 becomes "motion": the inner rect is stretched by `stretch` anchored at its bottom edge
  (negative compresses), then shifted by (dx, dy), nearest sampling; a pixel sampled outside the source's inner rect
  takes its nearest inner edge pixel. The border never moves (as E13).
- Every anchored layer follows the motion as the marks follow the breath today: blink lids, eye glow, glow, veins, the
  face tint box and the mark. `ash` and `smoke` do not.
- Blend: when the winning face changes, the output (stretch, dx, dy as floats) is `lerp(old, new, w)` with
  `w = (t - change) / MOTION_BLEND_MS` (200), then rounded, so the change jumps at most 1 px a step.
- `still` stops every motion part (a downed unit's `hurt` face does not wince).
- `PortraitFx::set_base_motion(Motion)` sets the motion of "" (default `Motion::default()`, today's breath);
  `ExpressionFx.motion` holds each face's motion. A face with no row uses the base motion.

### 4.3 Tiers, reduce motion, budget, agent link

| Case | Cycle, lean, bob | Entry, rest, shiver | Blink rule | Flush |
| --- | --- | --- | --- | --- |
| Full (96 and up) | drawn | drawn | kept | pulses |
| Reduced (48-95) | none (as today's breath) | halved: `px` of v / 2, at least 1 | kept | pulses |
| Minimal (under 48) | none | none | n/a | steady, b = 1 |
| Reduce motion | none | none | kept | steady, b = 1 |

- A face whose motion is moving (any part not yet constant) is time-varying for the `Budget`. A face that has only a
  held rest offset after `rest_ms`, and no cycle, bob or shiver, is not.
- `state(now)` gains `"motion": {"stretch": s, "offset": [dx, dy], "parts": [live part names]}`; the composed cache
  key gains stretch, dx and dy.
- Kit widgets Portraits page: a choice `kit.portrait.motion` of `MOTION_SAMPLES` (`breath`, `wince`, `heave`, `shiver`,
  `lift`, `bounce`, `sink`, `jolt`: the eight profiles of 6.1 with generic names), drawn on the sample face; the foot
  line names the live parts.

## 5. Kit (PORTRAITFX2-M schema, PORTRAITFX2-F list and events)

### 5.1 Motion fields on `portrait_expressions` (schema-first: yes; PORTRAITFX2-M)

| Key | Type | Default | Validation (error unless "warns") |
| --- | --- | --- | --- |
| `breath_amp` | float | 2.88 | 0-4.0 (0 = no cycle) |
| `breath_ms` | int | 3600 | 800-8000 |
| `breath_in` | float | 0.5 | 0.2-0.8 |
| `breath_jitter` | float | 0.0 | 0-0.5 |
| `lean_px` | int | 0 | -3 to 3; needs `breath_amp` above 0 |
| `bob_px`, `bob_ms` | int | 0, 0 | 0-3; `bob_ms` 300-3000 with a bob, else 0 |
| `enter_dy`, `enter_stretch` | int | 0, 0 | -3 to 3 each |
| `enter_ms` | int | 0 | 60-1200 when either entry field is not 0, else 0 |
| `rest_dy`, `rest_ms` | int | 0, 0 | -3 to 3; `rest_ms` 0-3000, 0 without a rest |
| `settle_ms` | int | 0 | 0-4000 |
| `shiver_px`, `shiver_ms`, `shiver_gap_ms` | int | 0, 0, 0 | 0-2; with a shiver `shiver_ms` 80-1000 and `shiver_gap_ms` 200-8000, else 0 |
| `blink` | bool | true | - |
| `tint_breath` | float | 0.0 | 0-1.0; needs a `tint` and `breath_amp` above 0 |

- The defaults are today's breath, so today's three rows keep their bytes (the writer stores only fields that differ,
  comparing floats as normalize snaps them).
- Warns: `|lean_px| + bob_px + |enter_dy| + |rest_dy| + shiver_px` above 3 ("clamped at 3 px").
- `PortraitExpression` gains the fields; `portraits::expression_fx_of` maps them to `Motion`.

### 5.2 The face list is the game's data (PORTRAITFX2-F)

- FL1. The faces are `neutral`, then the ids of `portrait_expressions` in file order. When the kind has no rows, the
  kit's `DEFAULT_FACES` apply (`neutral`, `hurt`, `angry`, `afraid`). `rpgkit_data::EXPRESSIONS` becomes
  `DEFAULT_FACES`, and `NEUTRAL` ("neutral") keeps `face_chain`'s fallback. `Catalog::faces() -> Vec<String>`.
- FL2. Row `id`: `[a-z][a-z0-9_]{1,23}`, unique. `neutral` may have a row with motion fields only ("neutral is the
  plain face: motion fields only" when it sets a tint, a mark or a shake). Its motion becomes the base motion.
- FL3. The "draws nothing" warning now needs no tint, no mark, no shake and every motion field at its default.
- FL4. Readers take the list from the catalog: `check/dialogue.rs` (a node's and a `line` card's `expression`: "" or a
  face), `check/portrait.rs` (a `face` cue's `expression`), `check/assets.rs` A12 (file names under `faces/<id>/`),
  `gallery.rs` (frames in list order), `region.rs`. Each error lists the game's faces. Schema help texts say "a face of
  `portrait_expressions` (or neutral)" instead of the four names.

### 5.3 Triggers for the new faces (PORTRAITFX2-F; `rpgkit_battle/src/portrait.rs`, read-only)

| Event or trigger | Who gets it | When |
| --- | --- | --- |
| `battle_start` | every living unit | the battle screen's first frame, before the first step |
| `ambush` | the player side's living units | with `battle_start`, when the encounter's `deployment` is `ambush` |
| `round_start` | every living unit | the session's round counter rises (classic; the timeline ruleset has no rounds and fires none) |
| `skill` | the acting unit | the step's wind-up (FX1), when its command is a skill whose `level` is at least the row's `min_level` |
| `ally_down` | every other living unit on the side of a unit that went down in the step | the impact, once per unit per step |
| `victory` | the player side's living units | the step that ends the battle in the player's win |
| trigger `unspent` (state) | a party member with `unspent(d)` above 0 | while it holds; `Look.unspent` (0 in battle) |

`PORTRAIT_EVENTS` gains the six events, and `PORTRAIT_TRIGGERS` gains `unspent`. Cue rows gain `min_level` (int,
default 0, 0-9, only on `skill` rows; 0 = any skill). Dialogue nodes and `line` cards name the new faces through their
existing `expression` key. A speaker with no frame for the face draws its neutral frame along `face_chain`, with the
face's motion and colour.

## 6. Data (asset lane; through `tools/fc.sh content upsert`, then `content check`)

### 6.1 Motion rows (M: the first three; FA: the four new rows). Values at 240 px; blank = default

| Face | Look | `breath_amp` / `_ms` / `_in` / `_jitter` | Other fields |
| --- | --- | --- | --- |
| `neutral` | today's breath (no row) | 2.88 / 3600 / 0.5 / 0 | - |
| `hurt` | a squeeze, then shallow uneven breaths, no blinks | 1.5 / 1700 / 0.4 / 0.35 | `enter_stretch` -2, `enter_ms` 280, `blink` false |
| `angry` | deep fast heaving, a lean, the flush pulsing | 4.0 / 2100 / 0.4 / 0.1 | `lean_px` 1, `tint_breath` 0.5 |
| `afraid` | quick shallow breaths, 1 px jitter bursts, the sweat | 1.2 / 1300 / 0.5 / 0.2 | `shiver_px` 1, `shiver_ms` 280, `shiver_gap_ms` 1400 |
| `determined` | a small lift, steady slow breath | 2.4 / 4400 / 0.5 / 0 | `rest_dy` -1, `rest_ms` 400 |
| `happy` | a hop, then a light bounce | 2.4 / 3000 / 0.5 / 0 | `enter_dy` -2, `enter_ms` 320, `bob_px` 1, `bob_ms` 1100 |
| `sad` | a slow sink, drooping breaths (short in, long out) | 2.0 / 5200 / 0.3 / 0 | `rest_dy` 2, `rest_ms` 1500 |
| `surprised` | a quick jolt up, then still, wide-eyed | 1.0 / 2600 / 0.5 / 0 | `enter_dy` -3, `enter_ms` 360, `settle_ms` 1600, `blink` false |

The new rows have no tint, mark or shake (Q8).

### 6.2 Face cue rows (FA; `portrait_cues.json`, `effect` `face`, beside XA's 2.4.3 rows)

| id | trigger | match | expression | ms | priority | surfaces |
| --- | --- | --- | --- | --- | --- | --- |
| `face_battle_start` | event | `battle_start` | `determined` | 1800 | 30 | [] |
| `face_round_start` | event | `round_start` | `determined` | 1000 | 25 | [] |
| `face_big_skill` | event | `skill`, `min_level` 3 | `determined` | 1200 | 55 | [] |
| `face_ambush` | event | `ambush` | `surprised` | 1500 | 65 | [] |
| `face_ally_down` | event | `ally_down` | `sad` | 2000 | 65 | [] |
| `face_victory` | event | `victory` | `happy` | 3000 | 90 | [] |
| `face_level_up` | unspent | - | `happy` | 0 | 35 | `roster` |

### 6.3 Line expressions (FA, as XA 2.4.4)

For slice dialogue nodes and `line` cards with existing data: a hero's battle-start line or a vow gets `determined`; camp
banter, a joke or a celebration gets `happy`; grief, a loss or a farewell gets `sad`; a reveal, a twist or an ambush
line gets `surprised`. Lines already marked `angry`, `hurt` or `afraid` keep their marks. Any speaker may get a new face
(Q10).

## 7. The second sheet (PORTRAITFX2-T tool, PORTRAITFX2-A art)

- T: `sheets.py faces --cells determined,happy,sad,surprised` (default `FACE_CELLS`; four names in reading order) and
  `--align-to <png>`. Every cell is registered (scale and offset) to that frame instead of the sheet's own neutral
  cell. The default is `faces/<id>/neutral.png`, else the base portrait. Scale-only reduction (Gemini art), as XA's cut.
- A: the six founders (`kaelen`, `vesper`, `maren`, `hollis`, `dunstan`, `elyse`). The prompts file is
  `games/emberwake/docs/specs/PORTRAITFX2_FACES.md`, laid out like PORTRAITFX1_FACES.md: the block below, then per id
  its "Character:" and "Mood:" lines from [FOUNDER_PORTRAITS.md](FOUNDER_PORTRAITS.md) 3.x, its keep line (table) and
  `reference:` (Q9). The planner queues them (`artqueue.py request <ids> --faces --prompts ...PORTRAITFX2_FACES.md`, 6
  images, generation only). Then a lane ingests them, cuts with `--cells` and aligns, writes
  `portraits/heroes/faces/<id>/{determined,happy,sad,surprised}.png` and their sidecars (`face.py`, PORTRAITFX1_TASKS
  2.1), and posts a board in `C:\dev\localgen\review\PORTRAITFX2-A\`. Up to 2 re-rolls a founder, or a single-cell
  edit of the neutral frame (as XA), on the user's word.

```text
Use the attached image as the reference: the same person: face, hair, outfit, colours, framing, lighting and pixel-art style; only the expression changes.
A 2x2 grid of four pixel art character portrait variations for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG portraits. 1:1 square image. No borders, no frame, no text, no dividers, no logo. Plain flat very dark background (#120e12) behind all figures. Each variation shows the same head-and-shoulders, three-quarter view of the character below, the head the same size and in the same place in every cell, the same pose, hands and props in the same place: only the face changes. Each variation utilizes a 16-bit pixel art style with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per color, and light dithering on metal and cloth. A warm amber tallow-lantern rim light (#d97706) from the right, soft warm fill light, deep warm brown shadows. Same outfit, hair and colours in all four.

Keep every accessory, scar, marking and piece of equipment exactly as in the reference in all four cells, including eyepieces, masks and jewellery.

The four variations are:
- Top-Left: Determined: jaw set, chin slightly raised, brows level and lowered a little (firm, not a scowl), eyes focused straight ahead with a steady glint, lips pressed in a firm line; resolved, clearly different from a calm neutral face and from anger (no snarl, no flush).
- Top-Right: Happy: a broad warm smile showing teeth, cheeks lifted, eyes crinkled into bright crescents, brows relaxed and raised a little; openly glad.
- Bottom-Left: Sad: the inner ends of the brows raised and drawn together, eyelids lowered, eyes looking down and glistening, mouth corners turned down, lips closed and trembling.
- Bottom-Right: Surprised: eyes wide and round with the whites showing above the irises, brows arched high and even (not pinched together), mouth dropped open in a round O, the face otherwise relaxed; startled, clearly different from fear (no grimace, no cringe).
```

| Id | Keep line (added after the accessory rule) |
| --- | --- |
| `kaelen` | The notched billhook upright in his left fist and the linen-bound left forearm with its three bone spurs stay the same in every cell. |
| `vesper` | The reversed bodkin in her raised hand, the smoke wisps from those fingertips, the tin whistle and the lowered cowl stay the same in every cell. |
| `maren` | The round wire spectacles with the cracked lens, the ink-black throat veins and the censer crook stay the same in every cell. |
| `hollis` | The grafted black left eye with its red slit pupil and stitched scar ring never changes or closes; only his own right eye and the face around it change. |
| `dunstan` | The slag-burned right jaw with its wire and iron strap, the raised fists in penal cuffs with chain, and the shoulder brand stay the same in every cell. |
| `elyse` | The brass goggles pushed up on her forehead, the bleached curl, the flask bandolier and the sprayer nozzle stay the same in every cell. |

## 8. Tests (rule-derived)

| Test | Checks |
| --- | --- |
| `aegis_ui/tests/portrait_fx.rs` MO1 | No profile and `Motion::default()` give `breath_px` exactly at every `t` and side; the existing E and EX tests pass unchanged |
| MO2 | Cycle: `breath_in` r puts the peak at `r P'`; j = 0 is periodic on `t`; j > 0 repeats for the same key, and each cycle's amplitude is within `[A (1 - j), A]` |
| MO3 | Entry: `enter_stretch` -2 at side 240 reaches -2 at `0.25 E` and 0 at E; the cycle is 0 before E and reaches full scale at `E + settle_ms` |
| MO4 | Lean, bob, rest, shiver per 4.1 (shiver 0 between bursts, within `±px(X)` in one); the 4.1 clamps |
| MO5 | No border pixel changes under any profile (jolt and shake off); content sampled past the inner edge takes the edge pixel |
| MO6 | `blink` false: no lid pixel while the face holds; blinks return after |
| MO7 | `tint_breath` 0.5: the tint strength is half at b = 0 and full at b = 1; steady at full under reduce motion and at the minimal tier |
| MO8 | Reduce motion and the minimal tier: stretch 0, offset [0, 0] for every sample profile; reduced: no cycle, entry and rest halved (at least 1 px) |
| MO9 | `still` stops every part; at 100 ms of a blend the output is the rounded midpoint |
| MO10 | Lids, mark and face tint box move with the motion; ash does not |
| MO11 | Scaling: at side 120 each amplitude is `px(v)` (a 1 px shiver stays 1 px); the Budget counts a moving face and not a settled rest-only face |
| `rpgkit_content/tests/portrait_expressions.rs` | Each error and warning text of 5.1 and FL2-FL3; defaults leave today's rows' bytes; writer round trip |
| `rpgkit_content/tests/faces.rs` (F) | FL1 (no rows: the default four; rows: neutral then file order); FL4 for dialogue, cues and A12 with a game face `smug`, and its refusal before the row exists |
| `rpgkit_battle/tests/portrait.rs` (F) | Each 5.3 row: who and when (`ambush` player side, that deployment only; `skill` vs `min_level`; not the downed unit; no timeline `round_start`) |
| `emberwake_gui` `portraits.rs` (F) | A `sad` face on a speaker without frames draws the neutral frame sunk by `px(2)` after 1500 ms; a `face_victory` cue wins over `face_low_hp` |
| `test_sheets_faces.py` (T) | `--cells` names the outputs; `--align-to` registers a shifted, scaled fixture cell within 1 px |

## 9. Determinism and parity

Seeds come from the portrait key and the face name; no game RNG is read. The events are read-only over the session, so
RNG and traces are unchanged. Godot parity: none retires (both kinds are after-Godot, the `expression` help is in
HELP_CHANGED; no Godot world file changes).

## 10. Slices and task rows (the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| PORTRAITFX2-M | `Motion` in `portrait_fx` with tiers, reduce motion, samples and agent state; the 5.1 fields; then the three rows of 6.1 | engine + kit + game | logic, then asset | PORTRAITFX2-S, PORTRAITFX1-XA | `aegis_ui/src/{portrait_fx,specimens}.rs`, `aegis_ui/tests/portrait_fx.rs`, `kits/rpg/schema.json`, `rpgkit_data/src/lib.rs`, `rpgkit_content/src/check/expression_fx.rs`, its tests, `emberwake_gui/src/portraits.rs`; rows: `portrait_expressions.json` |
| PORTRAITFX2-F | The face list as data (5.2), the events and `unspent` (5.3), `min_level`; the game fires the events and lists faces | kit + game | logic | PORTRAITFX2-M (same kit files) | `kits/rpg/schema.json`, `rpgkit_data/src/lib.rs`, `rpgkit_content/src/{art.rs,check/{assets,dialogue,portrait,expression_fx}.rs}`, `rpgkit_battle/src/portrait.rs`, tests, `emberwake_gui/src/{portraits,battle,gallery,region,hub,party}.rs` |
| PORTRAITFX2-FA | The four new face rows (6.1), the face cue rows (6.2), line expressions (6.3) | game (data) | asset | PORTRAITFX2-F | `portrait_expressions.json`, `portrait_cues.json`, slice dialogue and vignette rows (through `aegis content`) |
| PORTRAITFX2-T | `sheets.py faces --cells` and `--align-to` | game (tools) | logic | PORTRAITFX1-XA (its scale-only cut and alignment) | `tools/portraits/sheets.py`, `test_sheets_faces.py` |
| PORTRAITFX2-A | The second sheet for the six founders (7) | game (art) | asset | PORTRAITFX2-F, -T, FOUNDERPORT1-P, the founders' first sheet (PORTRAITFX1-XA) | `games/emberwake/docs/specs/PORTRAITFX2_FACES.md` (new), `art_src/`, `games/emberwake/assets/portraits/heroes/faces/` |

## 11. Docs delta

- GAME_BRIEF: per-face motion (`portrait_expressions`), the face list as data (`Catalog::faces`), the new events.
- `engine/docs/authoring/`: a game-agnostic note on motion profiles ("promote generic methods").
- PORTRAITFX1.md 4.2 and 5.4: a pointer to this spec. Kit README: `DEFAULT_FACES`.
- ROADMAP: a PORTRAITFX2 row, `approved`, after PORTRAITFX1.
- HISTORY: "One breath for every face (PORTRAITFX1 4.2)" replaced by per-face motion, with today's breath as the default.
  "The kit's closed expression list `EXPRESSIONS`" replaced by the game's face list, with the four as kit defaults.

## 12. Out of scope

Shears, head turns, per-layer motion (PUPPET1); figure masks; colour effects for the new faces (Q8); sheets for the
hub cast and bosses; sounds; region tokens; portraits on the Results screen.

## 13. Open questions (recommended option first)

- Q1. Where the profiles live: (a) on the per-face rows `portrait_expressions`; (b) on each `face` cue row (per
  trigger, but every row repeats the profile). Recommend (a).
- Q2. Formula: (a) parts composed by one function (4.1); (b) a `kind` enum with one fixed formula each. Recommend (a):
  a new face needs no engine change.
- Q3. Heave depth against the 1-3 px rule: (a) stretch up to 4 px at 240 (angry only), offsets at most 3; (b) 3 at most
  (today's breath is already 2.88, so the heave reads only from speed and lean). Recommend (a).
- Q4. The 64 px talk box at 1x is the reduced tier: (a) as PORTRAITFX1 (no cycle, entries halved); (b) cycles at half
  amplitude there too. Recommend (a).
- Q5. Face list home: (a) the ids of `portrait_expressions`, with the kit's four when there are no rows; (b) a `faces`
  list in `game.json`; (c) its own kind. Recommend (a): one place per face.
- Q6. Happy on level-up: (a) held on the roster surfaces while points are unspent; (b) a timed event on the first draw
  after a level gain (a not-saved memory); (c) none. Recommend (a).
- Q7. `round_start` in the timeline ruleset: (a) none; (b) at a unit's own turn. Recommend (a).
- Q8. Colour effects for the new faces: (a) none; (b) Happy a faint Candle Gold tint, Sad a cool tint. Recommend (a);
  the prompt pushes Determined away from Neutral.
- Q9. Sheet 2 reference: (a) the image the founder's first sheet used, cells aligned to its neutral frame; (b) the
  240 px game portrait. Recommend (a): the two sheets match.
- Q10. New faces for speakers without frames: (a) allowed (motion over the single portrait); (b) heroes only.
  Recommend (a).
