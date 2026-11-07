# FX1 tasks: checks, rows, sequencing, docs delta

Status: proposed (spec FX1, 2026-10-25). Companion of [FX1.md](FX1.md); section numbers below point there, except
sections 7 (the code survey) and 8 (the effect table, FX1 4.4), which live here.

## 1. Acceptance checks (rule-derived; each task's builder writes these tests)

### 1.1 FX1-E (`engine/crates/aegis_ui/tests/fx.rs` new, unit tests in `fx.rs`, `painter.rs`, `canvas.rs`)

| Id | Check | Rule |
| --- | --- | --- |
| E1 | `Clock`: advance(100, 1.0) gives 100; hold(40) then advance(100, 1.0) gives +60; hold(40), hold(30) holds 40; advance(100, 0.5) gives +50; advance(100, 2.0) gives +200 | 3.1 |
| E2 | Every primitive draws nothing before `start` and nothing at `u >= 1`; drawing the same effect at the same `now` twice gives identical pixels | 3.2 |
| E3 | A projectile with `arc_px` 6 is at `from` at u 0, `to` at u 1, and 6 px above the straight line at u 0.5 | 3.2 |
| E4 | `Sparks` with the same seed give the same pixels; another seed differs; a spark's alpha falls to 0 at u 1 | 3.2 |
| E5 | `hash01` is in [0, 1) for 10,000 keys and repeats for the same key | 3.2 |
| E6 | `Fx`: the 33rd push drops the oldest; `prune` drops the done; `busy_until` is the latest end; `draw` with one layer draws only that layer | 3.3 |
| E7 | `Camera::offset`: (0, 0) at and after `ms`; `abs(dx)` and `abs(dy)` never above `px`; axis `X` never moves y and `Y` never x; the main-axis sign alternates between non-zero steps; `Swell` is 0 at t 0 | 3.4 |
| E8 | Limits: a 9 px 5000 ms shake under limits (3, 1500) runs at 3 px for 1500 ms; limits (10, 9000) are clamped to the ceiling (6, 3000) | 3.4 |
| E9 | `start` under `reduce_motion` starts nothing; a weaker shake does not replace a stronger running one; a stronger one does | 3.4 |
| E10 | `Painter::shift(2, 0)`: a 1 px fill lands 2 logical px right; a clip set before the shift keeps its place; `shift(-2, 0)` restores | 3.4 |
| E11 | `Image::silhouette`: every opaque pixel takes the colour, transparent ones stay; `body_silhouette` with `flip` is the mirror of the unflipped one | 3.5 |
| E12 | `Settings::reduce_motion` is false on the defaults and true when set | 3.5 |
| E13 | Specimens: the Effects page lists the 6 primitive buttons and 5 preset buttons; pressing `kit.fx.shake.toll` shakes the stage only (a panel pixel outside it unchanged) | 3.6 |

### 1.2 FX1-S (kit; content check tests, `encounter.rs` parse test)

| Id | Check | Rule |
| --- | --- | --- |
| S1 | A card with `shake` `toll` passes; `shake` `boom` is refused with the preset list in the message; a card without `shake` writes no key | 5.1 |
| S2 | A phase with `{"type": "shake", "preset": "tremor"}` passes; a missing or unknown preset is refused; two shake effects in one phase are refused | 5.1 |
| S3 | `parse_phases` keeps `preset` on a shake effect; firing that phase changes no unit, no log text and no event text (the log entry equals the same phase without the effect) | 5.1, 6 |
| S4 | The gate's parity suites pass unchanged (`trace_parity`, `phasefold`, `checks_parity`, `data_source`) | 6 |

### 1.3 FX1-L1 (game; unit tests in `fx.rs` and `battle.rs`, on a scripted session with a fixed clock)

| Id | Check | Rule |
| --- | --- | --- |
| L1 | Look selection: a sword strike on an adjacent foe gives the sword trail; a bow shot 4 cells away an arrow; a wand strike 3 cells away the arcane bolt; a brawler with no weapon the fist ring; a mace at 3 cells with no skill row the generic reach bolt | 4.3, 4.4.1-4.4.2 |
| L2 | Impact time: melee at `t0 + melee_hit_ms`; an arrow over 160 px at 560 px/s flies 286 ms, clamped to 260, and lands at `t0 + 330`; a 20 px shot lands at `t0 + 70 + 60` | 4.2 |
| L3 | Rule I: between the pop and the impact the target's board HP bar shows the old HP, no float exists, no hurt clip plays and no hit sound has played; at the impact all of them start and the hit-stop holds `hit_stop_ms` | 4.2 step 3 |
| L4 | A unit killed by an arrow is drawn standing until the impact, then plays `death` | 4.2 step 3 |
| L5 | Rule F: a second pop before the first impact fires the first impact's feedback before the second action's effects start; `resolve` shows the end state with no pending impact | 4.2 steps 5, 7 |
| L6 | Rule P: at `anim_speed` 0.5 autoplay sends no step before the pending impact's wall time plus `impact_gap_ms`; at 1.0 with a melee hit the next step goes at `step_ms` as today | 4.2 step 6 |
| L7 | Dash: a `charge_in` that slides starts its trail at the slide's end | 4.2 step 1, 4.4.3 |
| L8 | Crit doubles the sparks and uses the crit hue; a hit for 0 halves them | 4.3 |
| L9 | Shakes: a 3-block skill that hits starts `heavy`; a 2-block one does not; a `[Break]` line starts `heavy`; a fired phase with a `tremor` effect starts `tremor` at once; a crit starts none | 4.5 |
| L10 | Token shake: the struck token's x offset is +1 and -1 on alternate 30 ms windows during `hit_flash_ms`, 0 after; never under `reduce_motion` | 4.5 |
| L11 | Settings: `skip_anim` gives no effect, no pending impact, no hit-stop and today's at-once feedback, yet a phase shake still starts; `reduce_motion` gives effects and hit-stop but no camera or token shake | 4.6 |
| L12 | The camera offset moves a board pixel and leaves a planning strip pixel and a card pixel unchanged; hit rects of `unit.<i>` and `tile.<x>.<y>` do not move | 4.1 |
| L13 | Both views: Stage anchors at the chest point (STAGE1 3), Flat at the token centre raised 2 px | 4.1 |
| L14 | Determinism: two runs of the same requests with the same frame times give identical frames; the battle's RNG state after a battle is the same with effects on and off | 4.2 step 8 |
| L15 | Tuning: values outside the ranges of 4.7 are clamped; a missing `shakes` entry uses the engine default; `hit_shake_px` reads 1 | 4.7 |
| L16 | The existing `battle.rs` tests (`tokens_follow_the_battle_step_by_step`, the facing tests) pass; where they assert a clip at the pop, they assert it at the impact (an intended change, named in the commit; `skip_anim` keeps the old behaviour) | 4.2 |
| L17 | The kit check's preset names equal `aegis_ui::fx::SHAKE_PRESETS` | 5.1 |

### 1.4 FX1-L2 (game)

| Id | Check | Rule |
| --- | --- | --- |
| M1 | Each skill row of 4.4.3 gives its delivery and impact (one test per row on a scripted action) | 4.4.3 |
| M2 | `firebomb` on a BURST of 3 struck units: one orb to the aim cell, one ring on the aim, sparks on the 3 units, all at the orb's impact | 4.3, 4.4.3 |
| M3 | `volley` launches one arrow per area cell, 30 ms apart, at most 5 | 4.4.3 |
| M4 | `mend` on an ally: the gold motes at its foot point; the heal float appears at the impact | 4.4.4, 4.2 |
| M5 | An item on another unit throws a flask first; on self it does not | 4.4.4 |
| M6 | A fall draws the dust ring on the ground layer (before the feet pass in the pass list) | 4.4.4, 4.1 |
| M7 | A miss: no sparks, the arrow ends 8 px past the target, the target's sidestep moves it 3 px away and back; no sidestep under `reduce_motion` | 4.3 |
| M8 | The agent link `state.fx` counts live and pending effects and goes to 0 after the last impact plus the longest effect | 4.8 |

### 1.5 FX1-V (engine `aegis_ui/tests/vignette.rs`, game `story.rs` test)

| Id | Check | Rule |
| --- | --- | --- |
| V1 | A card with `shake` `toll` starts the shake as it enters fade in (with its cue); the offset is 0 after 1100 card ms | 5.2 |
| V2 | A skipped reel starts no shake; `reduce_motion` starts none; a card with an unknown preset draws unshaken | 5.2 |
| V3 | While shaking, a caption pixel moves and the Skip button and the fade cover do not | 5.2 |
| V4 | The game's vignette screens use the tuning table (a tuned `toll` of 1 px gives offsets of at most 1) | 5.2 |

### 1.6 FX1-A (asset lane; `tools/fc.sh content check` clean)

| Id | Check | Rule |
| --- | --- | --- |
| A1 | `intro` cards 1 and 2 hold `toll` and `tremor`; no other card changed | 5.3 |
| A2 | `enc_pro_0a` phase `tremor` holds its line and the shake effect, in that order; nothing else in the encounter changed | 5.3 |

## 2. Rows wanted (status `spec`; the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| FX1-E | Engine effects kit: `aegis_ui::fx` (clock with hit-stop, projectile, beam, trail, sparks, flash, ring, the live list, camera shake with presets, limits and the ceiling), `Painter::shift`, `Image::silhouette` and `Art::body_silhouette`, `Settings::reduce_motion`, the Effects page of the Kit widgets (FX1 section 3; checks E1-E13) | engine | logic | STAGE1-E (same `painter.rs`, `canvas.rs`, `art.rs`; `ellipse_ring`), STORYUI1-E (same `screen.rs`, `lib.rs`; the key) | `engine/crates/aegis_ui/src/{fx (new),lib,painter,canvas,art,screen,specimens}.rs`, `engine/crates/aegis_ui/tests/fx.rs` (new) |
| FX1-S | Shake schema: the card key `shake`, the phase effect `shake`, checks, parse, writer, help text (5.1; S1-S4) | kit | logic | STORYUI1-S (the card schema and story check), KIT1-N (paths) | `kits/rpg/schema.json`; under `kits/rpg/crates/`: `rpgkit_content/src/check/{encounter,story}.rs`, `rpgkit_battle/src/encounter.rs`, the content check tests (new ones on scratch data in `rpgkit_content/tests/`, shipped-data ones stay in `emberwake_content/tests/`; KIT1_TASKS 5) |
| FX1-L1 | Battle effects core: `fx.rs` (look selection, melee and ranged rows, `arcane_bolt`, `shield_bash`, dashes), the FX clock in place of `anim_ms`, the impact timeline (rules I, F, P), hit-stop, the silhouette flash, the token shake fix, battle shakes, the camera shift, both views, tuning keys, the Reduce motion checkbox if STORYUI1-U1 has not added it (4.1-4.3, 4.4.1-4.4.2, 4.5-4.7; L1-L16) | game | logic | FX1-E, FX1-S (phase shakes), STAGE1-L, DIR4-G (same `battle.rs`) | `games/emberwake/crates/emberwake_gui/src/{fx (new),battle,stage,tuning,menus}.rs`, `games/emberwake/tuning.json` |
| FX1-L2 | Battle effects, the rest: the skill rows of 4.4.3, support, items, falls, misses with the sidestep, the agent link `fx` state (4.3, 4.4.3-4.4.4, 4.8; M1-M8) | game | logic | FX1-L1 | `games/emberwake/crates/emberwake_gui/src/{fx,battle}.rs` |
| FX1-V | Vignette shakes: `Card.shake`, `VignetteScreen::with_shakes`, the shifted parts; the game passes its tuning table (5.2; V1-V4) | engine + game | logic | FX1-E, STORYUI1-U1 (`story.rs`) | `engine/crates/aegis_ui/src/vignette.rs`, `engine/crates/aegis_ui/tests/vignette.rs`, `games/emberwake/crates/emberwake_gui/src/story.rs` |
| FX1-A | Slice shakes: `intro` cards 1-2, the `enc_pro_0a` tremor phase (5.3; A1-A2) | game (data) | asset | FX1-S, STORYUI1-A, SUNKWARD1 | `games/emberwake/data/worlds/sunk_ward/{vignettes,encounters}.json` through `tools/fc.sh content`, `games/emberwake/data/seeds/fx1.json` |

The FX1 row in `docs/TASKS.md` becomes the spec row (status `done (spec written 2026-10-25)`, pointer
`games/emberwake/docs/specs/FX1.md (+ _TASKS.md)`).

## 3. Sequencing and hub files

- `battle.rs`: STAGE1-L, then DIR4-G, then FX1-L1, then FX1-L2. CAMPAIGN1-U, STORYUI1-U1 and ENCOUNTER2-RU also edit
  it; none of them should run beside FX1-L1 or FX1-L2. FX1 keeps most code in the new `fx.rs`; `battle.rs` gets
  the clock swap, the pending-impact hooks in `after_change`, `sync_anims` and `tick`, and the draw hooks.
- `stage.rs` (STAGE1-L, new): FX1-L1 adds the two layer hooks and the shift around passes 1-9.
- `painter.rs`, `canvas.rs`, `art.rs`: STAGE1-E first, then FX1-E. `screen.rs`, `lib.rs`: STORYUI1-E first.
- `menus.rs`: the Reduce motion checkbox is STORYUI1-U1's (label "Reduce motion", id `settings.reduce_motion`, tip
  "No screen shake and no shaking on hits or in story cards; effects still show."); FX1-L1 adds it only when it is not
  there yet, and the later task keeps the one that exists.
- `tuning.json`: STAGE1-L sets `token_px` 40; FX1-L1 adds its keys after that.
- FX1-E and FX1-S can run beside the stage work (different files); FX1-V waits for STORYUI1-U1; FX1-A runs in the
  asset lane once its data exists.
- Slice order: FX1-E, FX1-S, FX1-L1 (melee, ranged, impacts, heavy and phase shakes), FX1-V and FX1-A (the bells),
  then FX1-L2.

## 4. Docs delta (the planner applies it with the builders' deltas)

- `games/emberwake/docs/lore/specs/art_and_visual_style.md`: a section "Effects" after 5: hit-stop 40 ms (limits
  30-60), the 1 px token shake, the camera shake limits (3 px, 1500 ms; engine ceiling 6 px, 3000 ms), the five
  presets of FX1 4.7, the effect colours of FX1 4.4, reduce motion (FX1 4.6); section 4's last line points to FX1.
- `engine/README.md`: an "Effects (FX1)" bullet: `aegis_ui::fx` (clock and hit-stop, primitives, live list, camera
  shake, limits), `Painter::shift`, `Image::silhouette`, `Settings::reduce_motion`, the Kit widgets Effects page;
  the vignette player's card shake.
- `engine/docs/PORT_NOTES.md`: `aegis_ui::fx`: engine-only (Godot had token tweens in `fx_presets.gd`; the miss
  sidestep is ported smaller, the melee bump is not); the token hit shake fixed (it never oscillated).
- `games/emberwake/docs/GAME_BRIEF.md`: where to look: battle effects in `emberwake_gui/src/fx.rs`, numbers in
  `tuning.json`.
- `docs/specs/STORYUI1.md` section 7: "camera shake on cards (FX1)" becomes "camera shake on cards: FX1-V".
- `games/emberwake/docs/lore/narrative/new_game_onboarding_and_intro.md` 4.1 and its open item "Whether the engine has a
  camera shake": it exists (FX1); ambient tolls per FQ6.
- `engine/docs/authoring/art_bible.md`: no change (its Effects row already asks for hit-stop and shake limits).

## 5. HISTORY rows (if the planner takes the recommendations)

| Idea | Status | Reason |
| --- | --- | --- |
| A data field for attack effects on skills and item families (FQ1 b) | not adopted (FX1, 2026-10-25) | A code table by kind and weapon family with per-skill rows covers the user's rule; data when a second game needs it |
| A whole-frame camera shake (FQ2 b) | not adopted (FX1, 2026-10-25) | The HUD must stay readable; the shake moves the world layer |
| Hit feedback at the action's pop with effects as decoration (FQ3 b) | not adopted (FX1, 2026-10-25) | Damage before the projectile lands reads wrong |
| The Godot melee bump (`FxPresets.MELEE_BUMP`) | not ported (FX1, 2026-10-25) | The rig's attack clips move the body |

## 6. Costs and risks

| Task | Size | Builder sessions |
| --- | --- | --- |
| FX1-E | M | about 1.5 |
| FX1-S | S | about 0.5 |
| FX1-L1 | M+ | about 2 |
| FX1-L2 | M | about 1 |
| FX1-V | S | about 0.5 |
| FX1-A | XS | under 0.5 |

Risks: rule I touches what `Tokens::catch_up` starts at the pop (L16 keeps the old tests honest; `skip_anim` keeps
the old path); pacing at slow animation speeds lengthens autoplay rounds (rule P, the intended effect); four rows
share `battle.rs` (section 3); STORYUI1-E and STAGE1-E are in flight on FX1-E's files.

## 7. Survey (develop 64f4419, before KIT1; kit line numbers are pre-KIT1, `rpgkit_*` under `kits/rpg/crates/`)

| Area | Where | Today |
| --- | --- | --- |
| Action pacing | `rpgkit_host/src/lib.rs:184-190`; `rpgkit_battle/src/session.rs:903-940`; `battle.rs:1080-1087` | One `step` pops and executes one action at once (`last_popped`, `session.rs:228`, set at `:929`); autoplay sends a step every `step_ms` (420 wall ms) |
| Action animation | `battle.rs:181-266` (`Tokens::catch_up`), `:309-320` (`action_clips`) | On a pop the actor turns (FACE1) and plays its attack clip by skill, kind and weapon; at the same moment targets play `hurt`, `death` or `defend` |
| Animation clock | `battle.rs:376-378`, `:1046-1051` | `anim_ms` grows by wall ms x `anim_speed` (0.5-2.0); clips run on it |
| Skip | `battle.rs:656`, `:1617` | `skip_anim` makes `catch_up` take the end state and the draw skip death clips |
| Hit flash | `battle.rs:341`, `:629`, `:1064`, `:1684-1686` | `flashes: (unit, Instant)` pushed on any HP drop (`after_change`, `:622-652`), kept `hit_flash_ms` (180) wall ms; a white rgba(255, 255, 255, 110) fill over the whole token square |
| Hit shake | `battle.rs:1649-1650`, `:1662`; `tuning.rs:15`, `:39`; `tuning.json:8` | `hit_shake_px` 2: the sign uses `now.elapsed()` where `now` is the frame's own `Instant::now()` (`:1379`), so it is always +1: the token sits 2 px right for 180 ms, it never shakes (a bug FX1-L1 fixes) |
| Floats | `battle.rs:113-118`, `:628`, `:633` | Damage and heal numbers born at the step (wall clock, `float_ms` 900) |
| Hit sounds | `battle.rs:640-645`, `:1208-1228`; defeat `:630-632` | Each new resolver log line plays its sound at the step |
| Settings | `screens.rs:24`; `menus.rs:629-634`; `aegis_ui/src/screen.rs:109-112` | Game keys `anim_speed`, `skip_anim`; STORYUI1-E (in progress, worktree `tactical-rpg-storyui1e`) adds engine keys `text_speed` and `reduce_motion` (`screen.rs:111-115`, `vignette.rs:20`, `:127` there) |
| Painter | `aegis_ui/src/painter.rs:90-94`, `:137`, `:310-350` | The painter holds a copy of the `View` (scale, letterbox offset); `clip` converts at the call; lines, ellipses, circles; no translation helper |
| Effects | engine and game crates | None: no particle, projectile, trail or shake code |
| Godot | `legacy/scripts/ui/combat/fx_presets.gd` (S14-L2), `combat_presenter.gd:64-167` | Token tweens only: melee bump 30 % toward the target, ranged 0.25 s, hit shake 4, -4, 2, 0 px, white flash, miss sidestep 6 px, heal and buff pulses, at most 24 live tweens; the engine ported the flash, the shake (with the bug) and the floats |
| Phases | `rpgkit_battle/src/encounter.rs:219-279`, `:285`; `session.rs:651-711`; `rpgkit_content/src/check/encounter.rs:151` | Phase effects `line`, `wave`, `status`, `ai`, `enrage`; a fired phase is in `Phases.fired`; the check refuses other types |
| Schema parity | `emberwake_world/tests/data_source.rs:38-47` | The `phases` help rows are already in `HELP_CHANGED` (PHASEFOLD) |

## 8. The effect table (FX1 section 4.4; code in `fx.rs`, palette hues from the art bible)

Colours: Steel `#9ca3af`, Linen `#d9cbb4`, Ash `#6b7280`, Amber `#d97706`, Gold `#fbbf24`, Orchid `#c084fc`, Cyan
`#06b6d4`, Crit `#e11d48`. Speeds in px/s. Spark `ms` is `spark_ms` (260) unless given.

### 4.4.1 Melee by weapon family (L1)

| Family | Trail (radius, from to deg, width, colour) | Impact |
| --- | --- | --- |
| `sword` | 13, -60 to 50, 2, Steel | Sparks 6 Steel / Linen, dir = the swing's direction, spread 90, speed 70, gravity 120, size 1 |
| `axe` | 13, -70 to 60, 2, Steel | Sparks 7 Steel / Amber, as sword |
| `dagger` | 9, -30 to 30, 1, Steel | Sparks 4 Linen / Steel, spread 60, speed 60 |
| `mace`, `hammer`, `staff` | 12, -80 to 40, 3, Ash | Ring 3 to 9 Linen a 200, 180 ms; sparks 8 Ash / Linen, spread 140, speed 50, gravity 160, size 2 |
| `fist` | none | Ring 2 to 7 Linen a 200, 150 ms; sparks 5 Linen / Linen, spread 120, speed 50 |

### 4.4.2 Ranged by weapon family (L1)

| Family | Look | Speed, arc | Impact |
| --- | --- | --- | --- |
| `bow` | Arrow len 7, shaft Linen, head Steel | 560, 6 | Sparks 4 Linen / Steel, dir = travel, spread 70, speed 60 |
| `crossbow` | Bolt len 5, shaft Steel, head Linen | 720, 2 | as bow |
| `sling` | Stone 2, Ash, hi Linen | 480, 8 | Sparks 4 Ash / Linen, spread 120, speed 40 |
| other ranged | as bow | as bow | as bow |
| generic reach (rule 4) | Orb r 1.5, core Linen, glow Steel, tail 2 | 480, 0 | Flash rx 4 ry 3 Linen a 120, 100 ms |

### 4.4.3 Skills (L2, except `arcane_bolt`, `shield_bash` and the dash rows in L1)

| Skill | Delivery | Impact |
| --- | --- | --- |
| `arcane_bolt` (and rule 2) | Orb r 2.5, core Linen, glow Orchid, tail 3; 420, arc 0 | Flash rx 5 ry 4 Orchid a 150, 120 ms; sparks 6 Orchid / Linen, spread 360, speed 50 |
| `hex` | Orb r 2, core Orchid, glow Orchid, tail 2; 360, arc 4 | Ring 2 to 8 Orchid a 220, 200 ms |
| `arc_lance` (LINE) | Beam to the last line cell, width 2, core Linen, glow Orchid, 160 ms | Sparks 4 Orchid / Linen per struck unit |
| `firebomb`, `inferno` (BURST) | Orb r 3, core Gold, glow Amber, tail 3; 380, arc 16, to the aim cell | Ring on the aim 4 to (area radius x 32 x 0.6) Amber a 230, 260 ms; Flash rx 10 ry 7 Amber a 140, 160 ms; sparks 10 Amber / Gold per struck unit, dir -90, spread 160, speed 70, gravity -20 |
| `glacial_burst` (BURST) | Orb r 3, core Linen, glow Cyan, tail 3; 380, arc 16 | Ring as firebomb in Cyan; sparks 8 Cyan / Linen, gravity 80 |
| `smite`, `judgement` | Orb r 2, core Linen, glow Gold, falling from 48 px above the target's chest; 600, arc 0 | Flash rx 8 ry 6 Gold a 160, 140 ms; sparks 6 Gold / Linen, dir -90, spread 200 |
| `volley` (PLUS) | One bow arrow per area cell (at most 5), launched 30 ms apart | Bow sparks per struck unit |
| `shockwave` (LINE) | Ring 3 to 12 Ash a 200 on each line cell, 40 ms apart from the actor out, ground layer | Ash sparks 5 per struck unit, gravity 160 |
| `sweep_kick`, `cyclone_kick` (SWEEP) | Trail radius 14, -150 to 150, width 2, Linen, at the actor | Fist impact per struck unit |
| `shield_bash` | none | Ring 3 to 10 Steel a 220, 180 ms; sparks 8 Steel / Gold, spread 120, speed 70 |
| `haymaker`, `crushing_blow` | Trail 10, -40 to 60, 2, Linen | Ring 2 to 9 Linen a 220, 180 ms; sparks 7 Linen / Ash |
| `lunge`, `pounce`, `charge_in` | `t0` after the slide (4.2 step 1); the family's melee row | the family's |

### 4.4.4 Support, items and falls (L2)

| Case | Effect |
| --- | --- |
| `mend`, `greater_mend`, `renewal`, `rejuvenate`, and any other support | At the target: sparks 8 Gold / Linen from the foot point, dir -90, spread 50, speed 30, gravity -30, 500 ms; Flash rx 7 ry 5 Gold a 80, 300 ms |
| `purify` | as mend in Cyan / Linen |
| `aegis` | Ring 4 to 12 Cyan a 220 at the chest, 260 ms; Flash rx 7 ry 5 Cyan a 90 |
| `rally`, `taunt_shout` | Ring 4 to 20 Linen a 200 at the actor's chest, 300 ms |
| Item on another unit | First a thrown flask: Stone 2, Linen, hi Steel; 420, arc 10; then the case of its effect: a heal or buff the mend motes, a damage consumable (`damage_type` `fire`) the firebomb impact on its aim |
| Item on self | The case of its effect, no flask |
| A unit falls | At its foot point, ground layer: Ring 3 to 10 Ash a 200, 300 ms; sparks 6 Ash / Linen, spread 160, speed 30, gravity 120, size 2 |
