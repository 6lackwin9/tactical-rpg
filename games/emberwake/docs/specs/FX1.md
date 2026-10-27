# FX1: attack effects and an engine camera shake

Status: proposed (spec FX1, 2026-10-25). Task approved by the user 2026-10-23 (playtest 20261022-204440; order FACE1,
DIR4-S, STAGE1-S, FX1) and extended 2026-10-25 with the camera shake (Q-82, `decisions/lore.md` "Art types (2)")
Depends on: STAGE1-E and STAGE1-L (the tilted stage: chest and foot points, draw passes; STAGE1 sections 3 and 3.7),
STORYUI1-E (the shared `reduce_motion` key), STORYUI1-S (the card schema), KIT1-N (kit crate paths)
Lane: logic (FX1-E engine, FX1-S kit, FX1-L1 and FX1-L2 game, FX1-V engine + game), asset (FX1-A data)
Touches: per task in [FX1_TASKS.md](FX1_TASKS.md) section 2; overall `engine/crates/aegis_ui/src/{fx (new),painter,
canvas,art,screen,specimens,vignette,lib}.rs`, the kit schema and checks (`kits/rpg/schema.json`,
`kits/rpg/crates/rpgkit_{content,battle}/src/`), `games/emberwake/crates/emberwake_gui/src/{fx (new),battle,stage,
tuning,story,menus}.rs`, `games/emberwake/tuning.json`, the `sunk_ward` vignettes and encounters
Companion: [FX1_TASKS.md](FX1_TASKS.md) (acceptance checks per task, task rows, sequencing, docs delta, HISTORY rows)

## Goal

Attacks read as attacks: an arrow flies from the archer to the target, a sword leaves a swing trail and sparks where
it lands, a spell is a glowing bolt or a burst on the ground, a heal is rising motes; the hit (flash, number, hurt
clip, sound) shows when the effect lands, not before. A short hit-stop and a 1 px shake give the impact weight (the
art bible, `lore/specs/art_and_visual_style.md` section 5). An engine camera shake with a strength, a duration, hard
limits and the shared reduce-motion setting serves bell tolls, heavy impacts and titan tremors, in battles and in
vignette cards. Everything is drawn in code from palette hues; no art, no image API. Effects are presentation only:
no rule, plan, AI, save or trace changes.

## 1. Survey

The code survey with file and line references is in [FX1_TASKS.md](FX1_TASKS.md) section 7. What shapes the design:
one battle `step` pops and executes one action at once, so the hit (HP, clips, sounds, floats) shows at the pop; the
GUI paces steps (`step_ms` 420); clips run on an animation clock scaled by `anim_speed`; the per-token hit shake
never oscillates (a bug); no particle, projectile, trail or shake code exists in the engine or the game.

## 2. Layers

| Feature | Layer | Carrier |
| --- | --- | --- |
| Effect primitives (projectile, beam, trail, sparks, flash, ring), the live list, the effect clock with hit-stop | `[engine]` | `aegis_ui::fx` (new) |
| Camera shake (presets by generic name, limits, envelope, deterministic offsets) and `Painter::shift` | `[engine]` | `aegis_ui::fx`, `painter.rs` |
| Body silhouette for hit flashes | `[engine]` | `canvas.rs` `Image::silhouette`, `art.rs` `Art::body_silhouette` |
| Reduce motion read | `[engine]` | `Settings::reduce_motion` (`screen.rs`), the key from STORYUI1-E |
| Effects page in the UI Gallery's Kit widgets | `[engine]` | `specimens.rs` |
| A card's `shake` played by the vignette player | `[engine]` | `vignette.rs` |
| Phase effect `shake` and the card key `shake` (schema, checks, parse) | `[kit hook]` | kit schema, content checks, `encounter.rs` |
| The effect table by command kind, weapon family and skill; impact sync; heavy shakes | `[kit hook]` placed in `emberwake_gui/src/fx.rs` (GUI code stays in `emberwake_gui`, KIT1) | game GUI crate |
| Shake numbers, effect timings and colours | `[data]` (tuning) | `games/emberwake/tuning.json` |
| Which cards and phases shake in the slice | `[data]` | `sunk_ward` vignettes and encounters |

## 3. Engine effects kit (FX1-E, `aegis_ui::fx`, game-agnostic)

### 3.1 Clock

`Clock { ms: f64, hold: f64 }` (Default 0, 0). `advance(wall_ms: f64, speed: f64)`: `h = min(hold, wall_ms)`;
`hold -= h`; `ms += (wall_ms - h) x speed`. `hold(ms)`: `hold = max(hold, ms)` (a hit-stop; holds do not add).
`now() -> u64` = `floor(ms)`. Callers feed the frame's wall time (at most 100 ms a frame, as the vignette screen).

### 3.2 Primitives

All points and sizes are logical px (f32), times are clock ms (u64), colours `Rgba`. Each effect has `start`, a
`layer` (`Ground` or `Air`) and a `seed: u64`; `u = (now - start) / ms` in 0..1; before `start` nothing is drawn;
at `u >= 1` it is done. Every draw is a pure function of the effect and `now` (no state, no RNG).

| Primitive | Fields | Draw at `u` |
| --- | --- | --- |
| `Projectile` | `from`, `to`, `travel_ms`, `arc_px`, `look` | Position `lerp(from, to, u)` raised `arc_px x 4u(1 - u)`; oriented along the path's tangent. Looks: `Arrow { len, shaft, head }` (a `len`-px line and a 2 px head), `Bolt { len, shaft, head }` (same, head 1 px), `Stone { size, col, hi }` (a `size` square with a 1 px highlight top-left), `Orb { r, core, glow, tail }` (a glow ellipse r + 1.5 at alpha 90, a core circle r, `tail` dots at the positions 12, 24, 36 ms earlier at alpha 160, 100, 50) |
| `Beam` | `from`, `to`, `ms`, `width` (1-3), `core`, `glow` | A `width`-px line in `glow`, a 1 px `core` line on it; alpha 255 x (1 - u) |
| `Trail` | `at`, `radius`, `from_deg`, `to_deg`, `ms`, `width`, `col`, `mirror`, `squash` | A swept arc on an ellipse of radii (`radius`, `radius x squash`): the head angle moves from `from_deg` to `to_deg` over the first 60 % (0 deg = the facing, negative = up; `mirror` flips x), the tail 70 deg behind the head (never before `from_deg`); 8 segments, alpha rising from 40 at the tail to 230 at the head; the whole arc fades over the last 40 % |
| `Sparks` | `at`, `ms`, `count` (1-24), `dir_deg`, `spread_deg`, `speed` (px/s), `gravity` (px/s2, + down), `size` (1 or 2), `cols` (two) | Spark i: angle `dir + spread x (h(seed, i) - 0.5)`, speed `speed x (0.5 + 0.5 h(seed, i + 50))`, position `at + v t + (0, gravity t^2 / 2)` (t in s), colour `cols[i % 2]`, alpha 255 x (1 - u); `size` squares |
| `Flash` | `at`, `rx`, `ry`, `ms`, `col` | A filled ellipse, alpha `col.a x (1 - u)` |
| `Ring` | `at`, `r0`, `r1`, `ms`, `col`, `squash` | `Painter::ellipse_ring` (STAGE1-E) at radius `lerp(r0, r1, u)` by that x `squash`; alpha `col.a x (1 - u)` |

`h(seed, k) -> f32` in [0, 1): splitmix64 of `seed ^ k x 0x9E3779B97F4A7C15`, top 24 bits / 2^24 (`fx::hash01`).

### 3.3 The live list

`Fx { live: Vec<Effect> }` with `MAX_LIVE` 32 (Godot `FxPresets.MAX_LIVE` was 24 tweens): `push(e)` drops the
oldest (smallest `start`, then push order) when full; `prune(now)` drops the done; `draw(p, now, layer)` draws that
layer's effects in push order; `busy_until() -> Option<u64>` the latest end; `clear()`; `kinds(now) -> Vec<&str>`
the live primitives' names (the agent link). Nothing is drawn under the caller's `skip`; the list is the caller's.

### 3.4 Camera shake

- `Shake { px: f32, ms: u32, axis: Axis, envelope: Envelope, step_ms: u32 }`; `Axis` `X`, `Y`, `Both`; `Envelope`
  `Linear` (1 - u), `EaseOut` ((1 - u)^2), `Swell` (sin(pi u)).
- Preset names (engine constant `SHAKE_PRESETS`, generic): `jolt`, `heavy`, `toll`, `tremor`, `quake`.
  `ShakeTable` maps a name to a `Shake`; `ShakeTable::default()` holds the engine's neutral values (the game's in
  5.6 are the same unless tuned); an unknown name gives None.
- `ShakeLimits { max_px, max_ms }`; engine ceiling `SHAKE_CEIL_PX` 6 and `SHAKE_CEIL_MS` 3000: a game's limits are
  clamped to the ceiling, a shake's `px` and `ms` to the limits.
- `Camera { shake: Option<(Shake, u64, u64)> }` (the shake, its start, its seed). `start(shake, now, seed, limits,
  reduce_motion)`: under `reduce_motion` nothing starts; else the shake (clamped) replaces the running one when its
  `px` is at least the running one's current amplitude, otherwise it is ignored (shakes never add).
- `offset(now) -> (i32, i32)`: `t = now - start`; at `t >= ms` (0, 0) and the shake ends; `k = t / step_ms`,
  `a = px x envelope(t / ms)`, `m = (-1)^k x (0.6 + 0.4 h(seed, k))`; `X`: (round(a m), 0); `Y`: (0, round(a m));
  `Both`: (round(a m), round(a x 0.5 x (2 h(seed, k + 101) - 1))). Whole logical px, so pixel art stays on the grid.
- `Painter::shift(dx: i32, dy: i32)`: moves everything drawn after it by whole logical px (adds `d x scale` to the
  view offset); `shift(-dx, -dy)` undoes it. A clip set before the shift stays where it was, so a shaken board
  stays inside its panel. Widget rects and hit tests never shift.

### 3.5 Silhouette flash and settings

- `Image::silhouette(col) -> Image`: every pixel at or above `OUTLINE_ALPHA` becomes `col`, others transparent.
  `Art::body_silhouette(key, palette, clip, frame, size, col, flip)` caches it like `body_outline`.
- `Settings::reduce_motion(&self) -> bool` reads `reduce_motion` (false when missing). The key and its default
  come from STORYUI1-E (STORYUI1 4.8: "whichever task lands first adds it"); `vignette::reduce_motion` may call it.

### 3.6 Effects page (UI Gallery, Kit widgets)

`specimens::PAGES` gains "Effects": a 240 x 120 dark stage with two dummy points 160 px apart; one button per
primitive (`kit.fx.projectile`, `kit.fx.beam`, `kit.fx.trail`, `kit.fx.sparks`, `kit.fx.flash`, `kit.fx.ring`) and
per preset (`kit.fx.shake.<name>`), a speed choice 0.5 / 1 / 2 (`kit.fx.speed`), a hit-stop toggle and a local
reduce-motion check (`kit.fx.reduce`); the stage shakes, the panels around it do not. The foot line says what ran.

## 4. Battle effects (FX1-L1, FX1-L2; `emberwake_gui/src/fx.rs` new, hooks in `battle.rs` and `stage.rs`)

### 4.1 Anchors and passes

- Stage view: from = the actor's chest point (STAGE1 3: the foot point raised `token_px` / 3) moved 6 px toward its
  facing; to = the target's chest point; an area's aim = the aim cell's foot point; rings and trails use squash 0.75.
- Flat view (until STAGE1-C): the token rect's centre raised 2 px; squash 1.0.
- Ground-layer effects draw after STAGE1 pass 4 (ground arrows); air-layer effects after pass 8 (outlines through)
  and before the floats of pass 9. The hit flash silhouette draws right after its unit's sprite in the feet pass.
- The camera offset shifts passes 1-9 (the board, clipped to its interior (128, 30, 384, 288)); the planning strip,
  the cards, the status bar, the banner and the log never shift.

### 4.2 The impact timeline of an action

When a step pops an action (`last_popped`, the action's impact block resolving) the screen builds its effects on the
FX clock (`Clock`, which replaces `anim_ms`; clips run on it too):

1. `t0` = now, or the end of the actor's slide when it moved in this step (dash, charge, Engage carry).
2. Melee: the trail starts at `t0`; impact at `t0 + melee_hit_ms`. Projectile: launch at `t0 + launch_ms`; travel
   = clamp(path px / speed, `travel_min_ms`, `travel_max_ms`); impact at launch + travel. Beam and support: impact
   at `t0 + launch_ms`.
3. At the impact (rule I): the impact effects (sparks, rings, flashes of the table), and everything the player reads
   as the hit on the board: the hit flash and token shake of each unit whose HP fell in this step, the floats, the
   `hurt`, `hurt_guarded`, `defend` and `death` clips, the board HP bars (they show the HP before the step until
   then), the fall (a unit whose fall is pending is drawn standing), the step's log sounds, the hit-stop
   (`hit_stop_ms` when any HP fell) and the heavy shake (4.5). The actor's attack clip and facing start at `t0`.
4. The HUD (cards, strip, log, message bar) updates at the pop, as today.
5. Flush (rule F): any later catch-up (the next pop, a new battle or view, `resolve`, the skip setting turning on)
   first fires every pending impact at once, in order. A step that popped no action (block or round wrap-up,
   status ticks, phases) has no effects and shows its changes at once.
6. Pacing (rule P): autoplay sends the next step at the later of `now + step_ms` and the wall time of the last
   pending impact plus `impact_gap_ms` (wall time = clock ms to go / `anim_speed` + the hold left).
7. Only the last pop of a catch-up gets effects (as clips today: a `resolve` that runs many actions in one frame
   shows its end state).
8. Seeds: `hash(round, block, action index, unit)`; effects never read or advance the battle's RNG.

### 4.3 Which look an action gets (first match)

1. A skill row in table 4.4.3 by skill id.
2. Kind `attack` or `area`, target 2 or more cells away (grid distance), and the main weapon's family is `staff`,
   `wand`, `orb` or `tome` or its `damage_type` is `magic`: the arcane bolt (4.4.3 `arcane_bolt`).
3. Kind `attack` or `area` with a ranged main weapon (`Equipment::is_ranged_weapon`): table 4.4.2 by family.
4. Kind `attack` or `area` with the target 2 or more cells away: the generic reach bolt (4.4.2 last row).
5. Kind `attack` or `area`, adjacent: table 4.4.1 by the main weapon's family ("" and `unarmed` read as `fist`).
6. Kind `support`: 4.4.4 by skill id, else the default motes. Kind `item`: 4.4.4 item rows.
7. Anything else (move, defend, focus, rest, status): no effect.

Modifiers on the impact: a crit (the step's `[Attack]` line has " CRIT") doubles the spark count and makes the second
spark colour Arterial Red `#e11d48` (the palette's crit hue); a hit for 0 (mitigated) halves the sparks and uses
Polished Steel `#9ca3af` for both; a miss (`[Attack Miss]`, `[Area Miss]`) has no sparks, a projectile flies 8 px
past the target, and the target steps 3 px away from the attacker over 80 ms and back over 140 ms (Godot
`FxPresets` miss, smaller for the 40 px tokens). An area's impact effects play on each unit whose HP fell, plus the
ring on the aim cell.

### 4.4 The effect table

The table (code in `fx.rs`, its numbers in the spec) is in [FX1_TASKS.md](FX1_TASKS.md) section 8, numbered 4.4.1
(melee by weapon family), 4.4.2 (ranged by weapon family), 4.4.3 (skills) and 4.4.4 (support, items, falls).

### 4.5 Shakes in battle

| Trigger | Preset | When |
| --- | --- | --- |
| A Heavy action (skill `blocks` 3 or more) that lowered any HP | `heavy` | at its impact |
| A break (`[Break]` log line in the step, RULESET1 P9) | `heavy` | at the impact of the step's action, else at once |
| A phase with `shake` effects fires (`Phases.fired` grew) | the effect's `preset` | at once; several: the strongest |

Crits do not shake (FQ5). The per-token hit shake: the struck token moves `hit_shake_px` (1) left and right,
switching every 30 wall ms, for `hit_flash_ms` from its impact (x only; fixes the bug in section 1).

### 4.6 Settings matrix

| Setting | Effects | Impact sync | Hit-stop | Token shake | Impact shakes (4.5 rows 1-2) | Phase and card shakes |
| --- | --- | --- | --- | --- | --- | --- |
| default | drawn | rule I | yes | yes | yes | yes |
| `anim_speed` s | durations / s (the clock) | yes | wall ms, unscaled | wall | clock | clock |
| `skip_anim` | none | none (today's at-once feedback) | none | none | none | yes |
| `reduce_motion` | drawn | yes | yes | none | none | none |

`reduce_motion` stops every shake (camera and token) and the miss sidestep; effects carry information (who hit
whom) and stay (FQ4). Hit-stop is a pause, not a motion, and stays.

### 4.7 Tuning (`games/emberwake/tuning.json`, live reload; clamped where read)

| Key | Default | Allowed | Meaning |
| --- | --- | --- | --- |
| `hit_stop_ms` | 40 | 0, 30-60 | the art bible's hit-stop; 0 = off |
| `hit_shake_px` | 1 (was 2) | 0-2 | the struck token's shake |
| `hit_flash_ms` | 180 (unchanged) | 60-400 | the white silhouette flash, alpha 150 |
| `melee_hit_ms` | 90 | 40-200 | swing start to impact |
| `trail_ms` | 150 | 60-300 | a trail's life |
| `launch_ms` | 70 | 0-200 | to a projectile's release |
| `travel_min_ms`, `travel_max_ms` | 60, 260 | 20-500 | projectile flight clamp |
| `spark_ms` | 260 | 100-600 | a spark's life |
| `impact_gap_ms` | 80 | 0-400 | rule P's pause after the last impact |
| `shake_max_px`, `shake_max_ms` | 3, 1500 | 0-6, 0-3000 | the art bible's limits (clamped to the engine ceiling) |
| `shakes` | the table below | per preset `px` 0-6, `ms` 0-3000, `axis` x/y/both, `envelope` linear/ease_out/swell, `step_ms` 20-120 | the game's `ShakeTable` |

| Preset | px | ms | axis | envelope | step_ms | Serves |
| --- | --- | --- | --- | --- | --- | --- |
| `jolt` | 1 | 90 | both | linear | 30 | spare (a light hit, a door) |
| `heavy` | 2 | 160 | both | ease_out | 30 | Heavy hits, breaks |
| `toll` | 2 | 1100 | x | ease_out | 70 | a cathedral bell: a slow sway |
| `tremor` | 2 | 1400 | y | swell | 45 | a titan tremor: rises, then fades |
| `quake` | 3 | 900 | both | ease_out | 35 | Act 3 quakes, a titan's wake |

The engine's `ShakeTable::default()` holds the same rows.

### 4.8 Agent link

The battle screen's `state` gains `fx`: `{"live": <n>, "pending": <n>, "shake": [dx, dy], "kinds": [...]}`, so a
test or an agent can wait for the effects to end. Widget names do not change.

## 5. Shakes outside battle

### 5.1 Schema (FX1-S, kit; schema-first: yes)

| Where | Key | Type | Default | Rule |
| --- | --- | --- | --- | --- |
| Vignette card (STORYUI1 4.2) | `shake` | enum | "" | "" or a name in `SHAKE_PRESETS`; the shake starts as the card enters its fade in, with its `cue` |
| Phase effect (catalog and world encounters) | `{"type": "shake", "preset"}` | object | - | `preset` required, a name in `SHAKE_PRESETS`; at most 1 per phase; no rule reads it |

The content check refuses an unknown preset (the kit check keeps its own copy of the five names, as the content
crate does not depend on `aegis_ui`; an FX1-L1 test checks that it equals `aegis_ui::fx::SHAKE_PRESETS`);
`parse_phases` keeps `preset` on `shake` effects; the `phases` help text gains the shape (its rows are already in
`HELP_CHANGED`, so no parity check changes). The writer stores `shake` on cards only when not "". (As built in FX1-S, 2026-10-28: cards are one JSON field the
writer stores as given, so a card without `shake` has no key while an explicit `"shake": ""` is kept; the asset lane
omits the key.)

### 5.2 The vignette player (FX1-V)

`Card.shake: String` (from `shake`, "" default); `VignetteScreen::with_shakes(table, limits)` (default: the engine
table and ceiling). When a card enters its fade in (the moment its cue plays, V7) its shake starts on the player's
card clock (`card_ms`); a skipped card starts none; `reduce_motion` starts none. The shift covers the picture
area, the caption box, the title and line texts and the portrait; the backdrop gradient, the fade cover and the Skip
button do not shift. The game builds its vignette screens with its tuning table (`emberwake_gui/src/story.rs`).

### 5.3 Slice data (FX1-A)

| Data | Change |
| --- | --- |
| `sunk_ward` vignette `intro`, card 1 ("... Time is kept only by the cathedral bells.") | `shake` `toll` |
| `sunk_ward` vignette `intro`, card 2 ("... whose nightmares shake the earth ...") | `shake` `tremor` |
| `sunk_ward` encounter `enc_pro_0a`, phase `tremor` (round 2, CAST1_ENCOUNTERS section 2) | effect `{"type": "shake", "preset": "tremor"}` added after its `line` |

## 6. Parity

None retires. Effects are drawn by the GUI, which no parity suite runs; rules, plan, AI, session, saves and
content are untouched. The `shake` phase effect is kept by the parser and read by no rule, so battle traces,
`phasefold`, sims and replays are unchanged; no Ashlands, Iron Reaches or Tidewrack row uses it, so `checks_parity`
and the loader parity see nothing new; the `phases` help rows are already exempt. Playtest recordings replay their
requests; effects follow from the session and the clock, so a replay draws the same effects for the same frames.

## 7. Out of scope

Windup anticipation frames and the amber windup outline (clip work), status visuals, knockback dust (no knockback
rule yet), Inoculated miasma (Act 2), the poise-break flash and crack on the strip (the strip's task; FX1 only adds
the `heavy` shake), animating the timeline strip at impacts, new sounds (the bell toll sound is SQ9's placeholder),
camera pan or zoom, region-map and world-map shakes and ambient tolls (FQ6), effects for 96 x 96 units, image-API
art, the Godot melee bump (the rig's attack clips move the body).

## 8. Open questions (recommended answer first; the planner takes it while the user is away)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| FQ1 | Where does the effect table live? | (a) code in `emberwake_gui/src/fx.rs` with the numbers in tuning; (b) data: an `fx` field on skills and item families (kit schema, checks, Forge) | (a): the user's rule is one look per skill kind and weapon family with per-skill extras, which a code table covers; data when a second game or a per-world look needs it |
| FQ2 | What does the camera shake move? | (a) the world layer: the battle board, the vignette's picture and texts; (b) the whole frame | (a): the strip, cards and log stay readable during a 1.4 s tremor; the engine supports both |
| FQ3 | When does the hit show? | (a) at the effect's impact (rule I), autoplay waits for it (rule P); (b) at the pop as today, effects only decorate | (a): an arrow that lands after the damage number reads wrong; the cost is one pending-impact queue in `fx.rs` |
| FQ4 | What does reduce motion stop? | (a) every shake and the miss sidestep; effects and hit-stop stay; (b) also the effects | (a): effects show who hit whom; the setting targets motion sickness |
| FQ5 | Do crits shake? | (a) no: double sparks with the crit hue; (b) a `jolt` on every crit | (a): the Ashlands roll crits often; shakes stay rare and mean something |
| FQ6 | The onboarding's bells "every few minutes with a heavy screen shake" (onboarding 4.1) | (a) FX1 does the story-placed shakes (cards, phases); ambient tolls on maps later as a kit hook (a timed location cue); (b) ambient tolls now | (a): the slice's bells are in the intro cards and the alley's tremor; a timed toll is a separate scheduler |
| FQ7 | The hit flash | (a) a white silhouette of the frame; (b) keep the white square | (a): at 40 px the square covers the ground and neighbours |
| FQ8 | Arcane spells use Resonant Orchid `#c084fc`, which the art bible lists for strain alerts and tremors | (a) Orchid as the slice placeholder, revisited in the M4 art pass (Q-81); (b) Candle Gold | (a): Gold is the Guard glow and headers; Blight Violet is reserved for Inoculated actions |
