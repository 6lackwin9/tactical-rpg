# WINDUP1: wind-up, intent stance and stagger animations

Status update (2026-10-28): approved; the open questions answered as recommended (user 2026-10-28, decision "WINDUP1 and STATUSFX1 spec answers" in combat.md).

Status: spec (2026-10-28). Approved by the user 2026-10-28 (decisions "Attack timing on the board" B and "WINDUP1
answers", `decisions/combat.md`); the seven answers are binding and are cited as A1-A7 below
Depends on: FX1-L1 (the fx clock, the impact queue of rule I, the `impact` frame in `clips.json`), DIR4-R1 (the
warrior's views, done), DIR4-G (`Facing`, done), RULESET1 (timeline blocks, poise, breaks, round intents)
Lane: logic (WINDUP1-E engine, WINDUP1-L game), asset (WINDUP1-R1..R3 rig)
Touches: per task in section 9; overall `engine/crates/aegis_core/src/assets.rs`,
`games/emberwake/crates/emberwake_gui/src/{battle,fx,tuning}.rs`, `games/emberwake/tuning.json`,
`tools/sprites/{rig,classes,variants,views,export_units}.py`, `kits/rpg/assets/units/**`,
`games/emberwake/assets/units/**`

## Goal

The timeline's telegraph reads on the board. A unit whose action takes more than one block raises its weapon when the
action starts and holds the pose, breathing, until the impact block; then the strike follows from that pose. While the
party plans, every unit stands in the pose of what it means to do this round (guard, attack, heavy attack, buff,
debuff). A wind-up broken by a poise break ends in a stagger instead of the strike, and a unit with the Staggered status
stands dazed while it lasts. Presentation only: no rule, plan, AI, save or trace changes.

## 1. Survey (what the build does today)

- `Tokens::catch_up` (`emberwake_gui/src/battle.rs`) plays the attack clip from frame 0 when a step pops the action at
  its impact block (`last_popped`, `action_clips`); the actor idles through its wind-up blocks. FX1-L1 moves the hit to
  the clip's `impact` frame.
- `plan_facing` turns units toward their target while the party plans (enemies only with Show enemy tells on).
- The rig (`tools/sprites/classes.py`) draws each attack with its own wind-up keys (frames 0 to 1 or 0 to 3) before
  the strike; creatures are in `rig.py` (CREATURE1). `ClipPlayer` (`aegis_core::assets`) plays a base loop and one shot.
- The kit already knows wind-ups: `poise::action_in_windup` (RULESET1 P5); `ScheduledAction` has `start`, `impact`,
  `cancelled`, `landed`, `broken`; `PoiseState.broken`; the Staggered status `stagger` (lost blocks 1 and 4).
- Art bible section 5 (canon): "Windup: one anticipation frame held across the windup blocks, weapon cocked, a faint
  amber outline". A6 adds a breathing hold; the outline is WQ5.

## 2. Schema contract (engine, WINDUP1-E; schema-first: yes)

Two optional fields on a `clips.json` row, beside FX1-L1's `impact`; both omitted when unset; a body without them
parses and draws exactly as today.

| Field | Type | Allowed | Default | Meaning |
| --- | --- | --- | --- | --- |
| `loop_from` | int | 1 to `frames - 1`, only with `loop` true | absent (0) | frames before it play once (the intro), then frames `loop_from..frames` loop |
| `strike` | int | 0 to `frames - 1`, at most `impact` when `impact` is set | absent (0) | the first frame after the clip's own wind-up frames; where the clip starts after a held wind-up |

Validation (`Clips::parse`, every problem listed):

- L1 `clip '<id>': loop_from needs loop true.`
- L2 `clip '<id>': loop_from must be 1 to <frames - 1>.`
- S1 `clip '<id>': strike must be 0 to <frames - 1>.`
- S2 `clip '<id>': strike must not come after impact.`

Playback (game-agnostic):

- `Clip::frame_at(ms)` with `loop_from` L: intro = sum of `ms[0..L]`; before it the intro frames; after it frame
  `L + (the loop position in (ms - intro) mod sum of ms[L..])`.
- `Clips::ms_before(id, frame) -> u64`: sum of the row's `ms` before `frame`.
- `ClipPlayer::play_from(clip, now, frame)`: a one-shot that starts at `frame` (its clock offset by `ms_before`); its
  length is the clip's total less that offset.
- `ClipPlayer::set_base_from(clip, now, from, still)`: the base loop starts at frame `from`; `still` stops the loop
  part: the intro plays, then frame `loop_from` (0 without one) holds. `set_base(clip, now)` stays
  `set_base_from(clip, now, 0, false)`; an unchanged base keeps its clock (as today).

Acceptance (engine tests): E1 L1, L2, S1, S2 each give their message; E2 a 7-frame clip, `loop_from` 3, ms
[90, 110, 120, 180, 180, 180, 180]: frame 2 at 250 ms, 3 at 320, 6 at 900, 3 again at 1040; E3 `play_from(c, 0, 2)`
shows frame 2 at 0 and ends at total less `ms_before(2)`; E4 `still` holds frame 3 from 320 ms on; `from` 3 with
`still` shows frame 3 at 0; E5 every shipped body parses and draws pixel-identical (regression on the tree).

## 3. Which actions wind up (rule W1)

The pose kind of a command, first match (the screen reads the scheduled action, its skill row at its rank and its
item):

| # | Command | Kind |
| --- | --- | --- |
| 1 | Defend (any blocks) | none: the guard stance as today (`defend`, `idle_guarded`) (A1) |
| 2 | Walk, a bundled move, Disengage | none: the walk and the slide (A1) |
| 3 | Rest, Target, lost-block entries (`staggered`, `stunned`) | none |
| 4 | Attack or Area whose skill has `power_mult` 0 (a debuff-only attack, schema help) | `debuff` |
| 5 | Attack or Area whose skill has `blocks` 3 or more (Heavy tier, RULESET1 4.1; FX1 4.5) | `heavy` |
| 6 | Attack or Area otherwise | `attack` |
| 7 | Support or Focus | `buff` |
| 8 | Item: aimed at a foe (`aims_at_foes`) `debuff`, else `buff` (WQ3) | as said |

- W1a Resolution: an action shows its wind-up only when it spans more than one block (`impact > start`, the scheduled
  span, so ranks and block modifiers count) and its kind is not none. A one-block action plays only its strike (the
  attack clip from frame 0), as today.
- W1b Planning: the kind of the opener (section 5) ignores the span: a Snap opener still shows the attack pose.
- Area skills, dash attacks, the `strike` basic (2 blocks) and enemy skills follow the same table. In the slice data,
  Heavy Thrust (3), Judgment (4), Heavy Cleave (4) and Overhead Smash (3) are `heavy`; Strike, Caustic Vomit, Grease
  Pot, Aimed Bolt are `attack`; Mend and Field Dressing are `buff`.

## 4. The clip set (per body; one set per grip, A2)

Each body sheet is one grip (the warrior's looks are bodies of their own, SPRITE1 WAR2H1), so the clips are per body,
keyed by the grip in the rig and inherited by every variant and named body of that grip (A7).

| Clip | Frames | ms | loop | `loop_from` | Shown |
| --- | --- | --- | --- | --- | --- |
| `windup` | 7 | 90, 110, 120, 180, 180, 180, 180 | true | 3 | attack wind-up and attack pose |
| `windup_heavy` | 7 | 100, 120, 140, 160, 160, 160, 160 | true | 3 | heavy wind-up and pose |
| `windup_buff` | 7 | 90, 110, 120, 180, 180, 180, 180 | true | 3 | buff wind-up and pose |
| `windup_debuff` | 7 | 90, 110, 120, 180, 180, 180, 180 | true | 3 | debuff wind-up and pose |
| `stagger` | 5 | 60, 90, 110, 120, 160 | false | - | the reaction: a break, the Staggered status landing |
| `dazed` | 8 | 150 x 8 | true | - | the loop while Staggered |

- The guard pose is the body's existing `idle_guarded` (A1, A3).
- Fallbacks (the screen plays the first the body has; none: the base as today): `heavy` -> `windup_heavy`, `windup`;
  `debuff` -> `windup_debuff`, `windup`; `buff` -> `windup_buff`; `attack` -> `windup`; guard -> `idle_guarded`;
  dazed -> `dazed`; the reaction -> `stagger`, else `hurt`.
- Creatures (A7) get `windup`, `stagger`, `dazed` only (their skills are attacks).
- Every attack-like clip (`attack`, `jab`, `haymaker`, `kick`, `sweep_kick`, `shield_bash`, `spit`) gains `strike`
  (table 7.2), view rows included.

## 5. Planning: the intent stance pose (A3)

While the party plans, each unit's base is the pose of its round, by priority:

1. `dazed` while it has the Staggered status (the status is the more urgent read; the tell chip keeps the intent).
2. Guard: its round intent is `guard` (timeline), or its first command is a Defend (classic, no intents).
3. The opener's kind (W1, W1b): the first skill or item command of its round. Party: its plan (`plan.steps_of`).
   Enemy: its tell line when that line is a skill or item action, else the first such line of a revealed `plan`.
4. No opener: by round intent: assault `attack`; support `buff`; maneuver: Engage `attack`, Utility `debuff`,
   Disengage none; no intent none.
5. None: `idle`.

- Enemies show a pose only while Show enemy tells is on (WQ1); with tells hidden they idle and keep their facing (as
  FACE1 does).
- The pose changes as the player changes intent, target or opener (its intro plays, section 7.1); the confirm keeps it:
  an action that starts at block 1 with the same clip carries on without a restart (an unchanged base keeps its clock).
  Any other change cuts to the new base.

## 6. Resolution: wind-up, strike, interrupts

At every catch-up the screen finds, per active unit, its action in wind-up: this round's scheduled action of the unit
with `poise::action_in_windup(a, current_block)` true, W1a met and a pose kind. `UnitAnim` keeps `windup: Option<(index,
kind)>` (the action shown).

- W2 Start: when a unit's action enters wind-up (the first catch-up with `current_block >= start`; block-1 actions at
  the confirm) its base becomes the kind's clip (fallbacks, section 4) from frame 0; the unit faces the action's target
  (unit tile, else the aim cell) then and at each later catch-up while it winds up (DIR4 G3 moved to the start).
- W3 Base priority while resolving: the wind-up clip; `idle_guarded` while it guards; `dazed` while Staggered; `idle`.
  Shots play over the base and return to it (a `hurt` during a wind-up flinches back into the held pose).
- W4 Strike: when the step pops the action shown in wind-up, the actor's attack clip (`action_clips`) plays with
  `play_from(clip, t0, strike)` (`strike` 0 when the row has none); FX1-L1's frame-derived times (melee impact, and
  any release time it derives from frames) are measured from `strike`: impact at `t0 + ms between strike and
  impact`. Then the base drops to W3's next entry. An action not shown in wind-up plays from frame 0 as FX1-L1.
- W5 Buff and debuff wind-ups (support, focus, item, debuff-only attacks without an attack clip) end at the pop: the
  base returns at `t0`; FX1-L2's motes and flasks carry the release (WQ4). A debuff-only attack with an attack clip
  strikes as W4.
- W6 Interrupt by a break (A4): the shown action is `cancelled` with `broken`. At the breaking hit's impact (FX1 rule I)
  the unit plays `stagger` instead of `hurt` or `hurt_guarded`; its wind-up base ends; no strike plays at its impact
  block. Any break of a unit (`PoiseState.broken` turning true) plays the same reaction, in a wind-up or not (WQ2).
- W7 The Staggered status (A4): when `stagger` newly appears on a unit (active or pending, `StatusState::has_any`) in a
  step, the unit plays `stagger` at that step's impact (once, even with a break in the same step); its base is
  `dazed` (W3) while `has_any("stagger")`.
- W8 Other ends: the actor falls (the `death` clip, no stagger); it withdraws (gone); any other cancel returns the base
  at once.
- W9 Amber outline (art bible 5, WQ5): while a wind-up of W2 shows, the token outline (`token_outline`, STAGE1 pass 8)
  is Tallow Amber `#d97706` at alpha `windup_outline_alpha`; the selected and hover outlines win.
- W10 A catch-up that runs several steps at once (`resolve`, FX1 4.2 rule 7) shows the end state: wind-ups in progress
  at the end, no reactions for steps it skipped.

### 6.1 Settings (A6)

| Setting | Wind-up and pose intro | Hold and dazed loops | `stagger` reaction | Strike from `strike` |
| --- | --- | --- | --- | --- |
| default | plays | breathe | plays | yes |
| `anim_speed` s | on the fx clock (/ s) | on the fx clock | on the fx clock | yes |
| `skip_anim` | none: `set_base_from(.., loop_from, ..)` | breathe | none (base switches at once) | no clip (as today) |
| `reduce_motion` | plays | still (`still` true: frame `loop_from`, `dazed` frame 0) | plays | yes |

The amber outline shows under every setting. `idle` and the other loops are not changed by reduce motion here.

### 6.2 Agent link

The battle screen's `state` gains `tokens`: one entry per unit, `{"unit": i, "clip": <the drawn clip id>, "base":
<the base clip id>, "pose": "attack" | "heavy" | "buff" | "debuff" | "guard" | "dazed" | "", "windup": [start, impact]
or null}`. STATUSFX1 adds a key to the same entries (whichever lands first adds the array).

### 6.3 Acceptance (game tests in `battle.rs`, scripted session, fixed fx clock, synthetic `Clips`)

1. W1 table: `strike` (ATTACK 2) `attack`; `tl_heavy_cleave` (4) and `tl_overhead_smash` (3) `heavy`; `mend` (SUPPORT
   2) and `focus` (FOCUS 2) `buff`; an ATTACK with `power_mult` 0 `debuff`; `active_guard` none; `move` none.
2. A 2-block strike from block 1: after the confirm the actor's base is `windup` (frame 0, then 3 at 320 ms); at its
   pop the shot is the attack at frame `strike` at `t0`; the impact lands at `t0 + ms_before(impact) - ms_before(strike)`.
3. A 1-block `tl_snap_jab`: no wind-up base; the attack from frame 0.
4. A break mid wind-up: `stagger` at the breaking impact, no `hurt`, no strike at the cancelled impact block; a break
   of a guarding unit also plays `stagger`.
5. Burrow Ambush inflicts Staggered: `stagger` at its impact, then base `dazed` until the status is gone, then `idle`.
6. The actor dies mid wind-up: `death`, no `stagger`.
7. Planning: Guard intent `idle_guarded`; Assault with a Heavy Cleave opener `windup_heavy`; Support `windup_buff`; an
   enemy on Assault with tells shown `windup`, with tells hidden `idle`; Staggered `dazed`.
8. Fallbacks: a body without `windup_heavy` plays `windup`; a creature on Support idles; a body without any wind-up
   idles and strikes from frame 0.
9. `skip_anim`: the wind-up shows frame `loop_from` at once; no `stagger` shot; `dazed` base set. `reduce_motion`: frame
   `loop_from` held through the hold; the intro and the strike play.
10. The amber outline only during a W2 wind-up; the selected outline wins.
11. Classic ruleset: wind-ups the same (blocks); no breaks; planning poses from the first command.
12. `state.tokens` matches the drawn clips.

### 6.4 As built (WINDUP1-L, 2026-10-29)

`emberwake_gui/src/battle.rs` (`Pose`, `PoseSet`, `command_pose`, `plan_pose`, `windup_of`, `base_of`, `put_base`,
`Tokens::follow`, `react_unit`, `windup_outline`, `Tokens::state`), `fx.rs` (`Shown.broken`, `staggered`, `winding`;
`Impact.strike`), `tuning.rs` and `tuning.json` (`windup_outline_alpha` 110). The tests are `battle.rs`
`windup_tests` (12; items 8 and 11 folded into others), on scratch bodies and synthetic clips.

- The bodies' pose rows are read once a battle's units are known (`BattleScreen::learn_poses` on the next tick, as the
  catch-ups run where the art cannot be read); until then, and for a body without them, nothing changes.
- The planning guard pose (5.2) also needs the rows: a body without any pose row stands as before WINDUP1.
- A break or Staggered without the `stagger` row reacts as before (`hurt` only on HP lost; not `hurt` on every break).
- An enemy that is Staggered shows `dazed` in planning with the tells hidden too (a status, not a tell; WQ1 hides the
  intent poses only).
- A wind-up that a step ends (a break, a cancel) ends at that step's impact (rule I); the actor's own ends at its pop.
- W5: a buff or debuff wind-up's base returns at the pop (also when its actor slides first).
- Heavy is read from the skill row found by the command's name (so at its rank); W1a from the scheduled span.
- Skip turned on while a wind-up base already plays keeps that base's clock (the WINDUP1-E note on `set_base_from`);
  a base set under skip starts at `loop_from`.
- `state.tokens[].clip` is the side clip id of the row last drawn (a missing clip's fallback included); the outlines seen
  through cover (STAGE1-O) are unchanged.

## 7. Assets: the rig (WINDUP1-R1..R3)

### 7.1 Rig rules

- K1 Intro: three frames lerping the pose keywords (numbers and tuples) from the clip's ready pose (`idle` frame 0) to
  the hold pose at k = 0.4, 0.75, 1.0. Hold: four frames, bob offsets 0, 0.5, 1.0, 0.5 px; `windup_heavy` adds a
  tremble dx 0, +0.5, 0, -0.5; `windup_buff` two gold motes (GOLDEN, GOLDEN_HI) rising 1 px a frame from the hand;
  `windup_debuff` a 1 px WHITE glint at the weapon tip on hold frames 0 and 2.
- K2 Continuity (A3: the strike follows the held pose): the `windup` hold pose is the grip's main attack at frame
  `strike - 1`; heavy pushes it (weapon 25-35 degrees further back, hands 1-1.5 px higher, bob +1, lean -1).
- K3 `stagger`: k0 head snaps back (head_dy -1), lean -2.5, dx -1.5, weapon arm flung wide; k1 dx -2.5, bob 1.5, back
  foot out; k2 knees buckle (bob 2.5, lean +1.5), weapon tip low; k3 sway dx -2; k4 = `dazed` frame 0. No flash (FX1
  flashes the hit); the weapon is never dropped.
- K4 `dazed`: slumped (bob +2, lean +1.5, head_dy +1.5), weapon tip toward the ground before the feet, the off hand or
  shield arm hanging; a sway of dx 0.8 sin and head_dy 0.5 cos over the 8 frames. No stars (stun is STATUSFX1's).
- K5 Colours: no new palette slot; the `palette` object and palette files unchanged (DIR4 R4); effect colours are the
  rig's constants already in use (WHITE, GOLDEN, GOLDEN_HI, the spark creams).
- K6 Views (warrior/base and warrior/2h_sword only, A7): every new clip gets `_down` and `_up` rows with the side row's
  frames, ms and `loop_from` (DIR4 R3), keys turned as DIR4 6.3 (a knock-back moves 1 px away from the facing).
- K7 Exporter order: today's side rows, the six new side rows, today's view rows, the new view rows.

### 7.2 Poses by grip

| Grip (bodies) | `windup` (= main attack frame `strike - 1`) | `windup_heavy` | `windup_buff` | `windup_debuff` | `strike` of each attack-like clip |
| --- | --- | --- | --- | --- | --- |
| Warrior 1H + kite (`warrior/base`) | sword drawn back by the hip behind the raised shield (attack 1) | sword raised over the shoulder, crouched | sword upright before the face, shield low, motes at the hilt | point levelled at the target at shoulder height | attack 2, shield_bash 2 |
| Warrior 1H + buckler (`1h_buckler`, Brask, Hest, Orsa) | weapon cocked behind the head, buckler forward (attack 1) | weapon higher, deep crouch | salute as base | levelled | attack 2, shield_bash 2 |
| Warrior 2H (`2h_sword`, `2h_axe`, `2h_hammer`, Grakka) | greatsword cocked over the shoulder (chest cut 1); axe and hammer raised behind the head (chop 1) | weapon high overhead in both hands, crouched | planted point down, both hands on the pommel, motes | levelled at the target | attack 2 |
| Ranger bow (base, goblin, Wenn) | full draw (attack 3) | full draw tilted up 10 degrees, lean back | bow lowered, string hand raised palm up, motes | half draw aimed at the target's feet | attack 4 |
| Brawler fists (base, goblin, goblin_berserker, Ash Warchief) | power fist cocked at the chest (jab 1) | power fist wound far back (haymaker 1) | fists together before the chest, head bowed, motes | near fist pointed at the target | jab 2, haymaker 2, kick 2, sweep_kick 2 |
| Priest mace (base, goblin) | mace raised over the head (attack 1) | mace high in both hands, crouched | mace upright before the face, off hand raised, motes | mace pointed at the target, off hand open | attack 2 |
| Rogue daggers (base, goblin, goblin_scout) | crouched, daggers drawn back low (attack 0, bob +1) | deeper crouch, daggers reversed | daggers lowered, off hand at the chest, motes | one dagger pointed at the target | attack 1 |
| Mage staff (base) | staff charging, gem glow 1.0 (attack 2) | staff high in both hands, glow 0.6-1.0 over the hold | staff upright, motes circling the gem | staff pointed at the target, glow 0.5 | attack 3 |
| Crawler (`crawler/base`, Ash-Crawler) | rearing, jaws open (attack 1) | - | - | - | attack 2, spit 2 |
| Quadruped (`beast/base`, Render-Beast) | coiled on the haunches (attack 0) | - | - | - | attack 1 |

Creature reactions: crawler `stagger` the head knocked back and down (head path -3, +4), jaws wide, legs scramble;
`dazed` the head 3 px above the ground, sway 0.6 px, jaw 0.5, bubbles at half speed. Quadruped `stagger` the head
jerked up (neck -55 degrees), dx -2, hind paws splayed; `dazed` head low (neck +15, head +20), tail down, slow sway.

### 7.3 Exporter checks and review (`export_units.py`; it exits on a failure)

- X1 Every humanoid rig body (not the dummy) has the six rows; creatures have `windup`, `stagger`, `dazed`; frames, ms,
  `loop` and `loop_from` as section 4.
- X2 Continuity: `windup` frame `loop_from` and the main attack's frame `strike - 1` differ in at most 15 % of their
  opaque pixels at 48 px.
- X3 Told apart: at the 28 px downsample (`views.difference`) the hold frames of the four wind-ups, `dazed` frame 0
  and `idle` frame 0 differ pairwise in at least 6 % of opaque pixels (the creatures: `windup`, `dazed`, `idle`).
- X4 `strike` on every attack-like row as table 7.2; a view row's frames, ms, `loop_from`, `strike` and `impact` equal
  its side row's.
- X5 Today's rows pixel-identical to an export before the task (DIR4 6.5 check 2); palette unchanged; no blank frame;
  body and content checks clean. Sheet growth: humanoids +6 rows, warrior/base and 2h_sword +18 (33 and 32 rows, at most
  1584 px), creatures +3; widths unchanged.
- X6 `--review OUT`: per body a board (rows `idle`, the four hold frames, the `stagger` strip, `dazed`; side, and down
  and up for the two warrior bodies) and GIFs of `windup` intro, two holds and the attack from `strike` (as the
  battle plays it), and `stagger` into `dazed`.

### 7.4 As built (WINDUP1-R1, 2026-10-29)

- Rig core (`tools/sprites/rig.py`): `strike(im)` / `strike_frame` and `loop_start(im)` / `loop_frame` mark frames as
  `impact` does (metadata only); `windup(kind, ready, hold, draw)`, `dazed(keys, draw)` and `windup_rows(draw, ready,
  holds, stagger_keys, dazed_keys)` draw K1, K3 and K4 through any draw function (the side `pose`, the views'
  `pose_view`); `lerp_kw` lerps numbers and tuples, weapon angles the short way round; `pose` and `pose_view` put the
  main hand and the weapon tip (`WEAPON_TIP`) in the frame's info for the motes and the glint.
- `strike` (table 7.2) on every attack-like clip of every body, the creatures included (`clips.json` only).
- The six rows on all nine warrior bodies (`classes.py`): 1H + kite (`warrior/base`), 1H + buckler (`1h_buckler`,
  Brask, Hest, Orsa), 2H (`2h_sword`, `2h_axe`, `2h_hammer`, Grakka); `windup` holds the grip's attack frame
  `strike - 1` pixel for pixel (X2 0 %). Views (`views.py`): the twelve view rows of `warrior/base` and `2h_sword`.
- Exporter (`export_units.py`): writes `loop_from` and `strike`; X1 binds the classes in `classes.WINDUP_CLASSES`
  (the warrior; R2 and R3 extend it, the other bodies print that they have none yet); X2 and X3 also on the view rows;
  X5 colours and palette slots; X6 `windup_review`. Today's rows are pixel-identical and the palettes unchanged on
  every body (an export before the task compared row by row); sheets 33 and 32 rows for the two view bodies.
- Readings of the rules: the heavy holds follow table 7.2's picture with K2's direction (further back or higher,
  crouched by bob +1.2 to +2.1, lean back); in the views the heavy tremble shakes the weapon hand and the dazed sway
  moves in whole pixels (a half-pixel shift of the symmetric figure blurs its legs); a mote is one pixel; the debuff
  glint sits 1 px past the tip, over the outline, so it shows beside the blade's white core.
- Review fixes (user 2026-10-29, the recommendations taken): the debuff hold's front view angles the blade to the
  unit's right side so the point shows (`warrior/base` and `2h_sword`); the 2H heavy hold is lower and further back
  (weapon behind the head, hands just above the shoulder) so every weapon fits the 48 px frame in every view on every
  2H body (Grakka's two intro frames, passing the weapon upright from his idle, still touch the top by 2 to 4 px, as
  his idle reach and attack do; Grakka's light wind-up and stagger intro frames touch the top too, as his idle, walk and attack already did; in the back view the greatsword's stagger hides the blade on frames 2-3 and shows it over the shoulder on frame 4, handing into dazed); from behind, the greatsword's buff hold is held point up before the chest and its
  dazed blade rests on the right shoulder, the tip showing above it (the hilt stays hidden); the 1H + buckler grip has
  view keys (`views.GRIPS['warrior/buckler']`): idle and walk with the sword raised as the kite look's, drawn on the
  review boards with stand-ins until the buckler has a view drawing (its sheets keep side rows only).
- Review fix (user 2026-10-29, done in WINDUP1-R2): the 2H heavy hold's front and back views no longer hold the
  weapon upright behind the head (the hilt by the face, only the tip over the helmet); as in the side view the blade
  rests angled back over the right shoulder, hands and hilt at shoulder height beside the head, inside the 48 px frame
  (`views.W2_WINDUPS`, `warrior/2h_sword`, the only 2H body with views; the axe and hammer have none). The intro
  frames lerp to it as before; only `windup_heavy_down` and `windup_heavy_up` changed.

### 7.5 As built (WINDUP1-R2, 2026-10-29)

- The six rows on the other five classes and every body of each (`classes.CLASS_WINDUPS`, `WINDUP_CLASSES` all six):
  ranger (base, goblin, Wenn), brawler (base, goblin, goblin_berserker, Ash Warchief), priest (base, goblin), rogue
  (base, goblin, goblin_scout), mage (base); side rows only (none of these bodies has view rows). Variants need no
  code: `classes.clips` draws them through the class's keys and the rig scales the keys by the body's size.
- `windup` holds the main attack's frame `strike - 1` pixel for pixel on every body (X2 0 %), as table 7.2: ranger
  attack 3, brawler jab 1, priest attack 1, rogue attack 0, mage attack 2; the brawler's `windup_heavy` is the
  haymaker's frame 1 pixel for pixel (the clip a heavy blow plays from its strike).
- Readings of table 7.2: the rogue's "attack 0, bob +1" is attack 0's own crouch (0.8 px below the idle's); a further
  1 px fails X2 (41 % apart). The ranger's buff motes rise from the raised string hand and the mage's circle the gem
  (3.5 px out, two opposite, a quarter turn a frame); the debuff glint sits past the nocked arrow's head (ranger) and the
  near fist's knuckles (brawler). The mage's heavy glow pulses 0.6, 0.8, 1.0, 0.8 over the hold.
- Rig (`rig.py`, generic): `windup` / `windup_rows` take `motes` (`'hand'`, `'off'` the off hand, `'tip'` circling the
  tip) and `steps` (a key's value per hold frame); `pose` puts the off hand (`off`) in the frame's info and a `tip` for
  a nocked arrow and for fists (metadata only).
- Checks: today's rows pixel-identical on every body (compared row by row with the sheets before the task), clip
  entries and palettes unchanged, widths unchanged, +6 rows a body; the new frames touch the frame's edge only where
  the body's own rows already do (the priest's `windup` is its attack frame 1, whose mace head already touches the top;
  the mage's hat tip touches the top in every row today).

### 7.6 As built (WINDUP1-R3, 2026-10-29)

- The creatures' three rows (`rig.CREATURE_ROWS`: `windup`, `stagger`, `dazed`; section 4's frames, ms, `loop` and
  `loop_from`) after today's rows on the four creature bodies: the kit's `crawler/base` and `beast/base`, the game's
  Ash-Crawler and Render-Beast (`rig.crawler_clips`, `rig.quad_windups`; side rows only, the creatures have no view
  drawings). Drawn through `rig.windup_rows` with flat pose keys (the crawler's head offset, ground point, neck bow,
  legs, fangs, bubbles, antennae; the quadruped's body, neck, head, jaw, tail and one key per paw, so the intro lerps
  every part from idle frame 0).
- `windup` holds the attack's frame `strike - 1` pixel for pixel (X2 0 %): the crawler rearing, jaws open (attack 1),
  the acid bubbling and the antennae twitching over the hold; the quadruped coiled on the haunches (attack 0), the tail
  lashing (phase and +-6 degrees) over the hold. The holds bob as K1 (the crawler's head, the quadruped's body).
- `stagger`: the crawler's head knocked back and down (-3, +4), jaws wide, the legs scrambling, then falling forward
  into `dazed`; the quadruped's head jerked up (neck -55 degrees), dx -2, the hind paws splayed, the legs giving.
  No flash, no effect colour.
- `dazed`: the crawler's head 3 px above the ground (its underside), swaying dx 0.6 and 0.4 px up and down, the jaws
  slack at 0.5; the quadruped's head low (neck +15, head +20 degrees, as numbers in the rig's angles, not offsets from
  the stance), the tail down 30 degrees, bob +2, a sway of dx 0.6 px and the neck 3 degrees.
- Readings: "bubbles at half speed" is a quarter rise up and back over the 8-frame loop (half the idle's travel per
  loop; a half rise straight up would jump at the loop's seam); a crawler without a coil (`crawler/base`) lying flat
  runs its whole length back along the ground, so its slump falls forward by `fit` px (2.2 for the base) to keep the
  tail and its trailing legs inside the 48 px frame; the Render-Beast's stance already holds its neck at +14, so its
  `dazed` reads low by bob +2, the head angle, the tail and the pitch rather than the neck.
- Rig (generic): `dazed` and `windup_rows` take `sway` (the sin and cos keys and amounts; the humanoids' default
  `rig.SWAY` is K4's dx 0.8, head_dy 0.5) and `steps['dazed']` (a key per frame); `windup` reads the hand only for the
  buff and debuff effects (a creature's draw puts none in the frame's info).
- Exporter: X1 binds the creatures (`variants.CREATURES`: the three rows, no other wind-up; else exit); the note "has
  no wind-up rows yet" is gone (every rig body but the dummy prop has wind-up rows); the review board's holds row names
  the wind-ups the body has. Checks: today's rows and clip entries pixel-identical to the sheets before the task,
  palettes and widths unchanged, +3 rows a body; the new frames touch the frame's edge only where today's rows do.

## 8. Costs

| Task | Size | Sessions |
| --- | --- | --- |
| WINDUP1-E engine fields and player | S | 0.5 |
| WINDUP1-L screen logic, tests | M | 1-1.5 |
| WINDUP1-R1 rig core (intro and hold helpers, `strike` marks on every body, checks, review) and the warrior's three grips, side and the two bodies' views | L | 2.5 |
| WINDUP1-R2 ranger, brawler, priest, rogue, mage; variants and named bodies re-exported | M | 2.5 |
| WINDUP1-R3 crawler and quadruped | S | 1 |

## 9. Task split

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| WINDUP1-E | Clip fields `loop_from` and `strike`, `ClipPlayer::play_from` and `set_base_from` (section 2) | engine | logic | FX1-L1 (same file: its `impact` field) | `engine/crates/aegis_core/src/assets.rs`, its tests |
| WINDUP1-L | Wind-ups, intent stance poses, stagger and dazed, strike from the pose, amber outline, settings, agent link (sections 3, 5, 6) | game | logic | WINDUP1-E, FX1-L1 | `emberwake_gui/src/{battle,fx,tuning}.rs`, `games/emberwake/tuning.json` |
| WINDUP1-R1 | Rig core and the warrior (section 7; review boards) | kit + game | asset | WINDUP1-E, DIR4-R1; after DIR4-R1F when that runs (both edit `views.py`) | `tools/sprites/{rig,classes,views,export_units}.py`, `kits/rpg/assets/units/**` (`strike` marks), `games/emberwake/assets/units/warrior/**` |
| WINDUP1-R2 | The other five classes, goblin and named variants | kit + game | asset | WINDUP1-R1 | `tools/sprites/{classes,variants}.py`, `kits/rpg/assets/units/{ranger,brawler,priest,rogue,mage}/**`, `games/emberwake/assets/units/{ranger,brawler,priest,rogue}/**` |
| WINDUP1-R3 | The creatures | kit + game | asset | WINDUP1-R1 | `tools/sprites/rig.py`, `kits/rpg/assets/units/{crawler,beast}/base/`, `games/emberwake/assets/units/{crawler,beast}/**` |

L does not wait for the art: with no wind-up rows every body behaves as today. Tuning key (L): `windup_outline_alpha`
110 (0-255, 0 = off). Parity: none retires (the GUI and the sheets are outside every parity suite).

## 10. Out of scope

Recovery-block poses (art bible 5 "Recovery"; enemies idle through recovery), a `cast` release clip (WQ4), status
effects other than Staggered (STATUSFX1), views beyond the warrior's two bodies (DIR4-R2, R3), `idle_focus` playback,
the Dojo and region map, wind-ups in the timeline strip, sounds, the reduce-motion idle.

## 11. Open questions (recommended answer first; the planner takes it while the user is away)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| WQ1 | Enemies with Show enemy tells off: show their stance pose in planning? | (a) no: idle, as FACE1 keeps their facing; (b) yes: the pose as a tell on the body | (a): the setting hides the intent; a pose would show it anyway. Their wind-ups still play in resolution |
| WQ2 | A break outside a wind-up (a guarding or idle unit, P7) | (a) the `stagger` reaction on every break; (b) only mid wind-up | (a): a break is a break; the clip says the unit lost its footing |
| WQ3 | Items over one block (P5 winds them up; A1 says skills) | (a) wind up by aim, buff or debuff; (b) no wind-up | (a): the strip shows the item's wind-up; the board should too |
| WQ4 | Buff and debuff wind-ups have no release clip | (a) the base returns at the pop, FX1-L2's motes carry it; (b) a `cast` clip per grip (about 2 more rig sessions) | (a) now; (b) as a later row if it looks weak in play |
| WQ5 | The art bible's faint amber outline during a wind-up | (a) yes, `windup_outline_alpha` 110, resolution only; (b) poses only | (a): canon, no rig cost, and it marks the units a hit would break |
