# DIR4: Four facing directions on the sprite rig

Status: approved (user 2026-10-23): scope A (front and back views of idle, walk and the main attack); a warrior-only
pilot in both looks, then the user's review; weapon looks baked (W1); the named warriors bake only the 2H family
they carry; a diagonal target keeps the side view; DIR4-R2 and R3 wait for STAGE1's camera pick; the region map
gets four-way facing in a separate DIR4-M after PLAYUI1-R; per-frame weapon anchors (W3) only if FX1's spec asks
for them. Was: proposed (spec DIR4-S, 2026-10-23; awaiting the user's answers in section 12)
Depends on: FACE1 (left/right facing by target, being built), WAR2H1 (the warrior's two weapon looks, side views,
being built), SPRITE1 and SPRITE2 (done), ASSETORG2 section 4.1 (the sheet format). STAGE1-S (battle stage look)
follows this spec and decides the camera; see section 9. Weapon looks and their cost: section 7.1 (user 2026-10-23).
Lane: logic (engine DIR4-E, game DIR4-G, DIR4-M), asset (rig and art DIR4-R1..R3)
Touches: `engine/crates/aegis_core/src/assets.rs`, `engine/crates/aegis_ui/src/art.rs`, `tools/sprites/`
(new `views.py`, `classes.py`, `variants.py`, `export_units.py`), `kits/rpg/assets/units/**`,
`games/emberwake/assets/units/**`, `games/emberwake/crates/emberwake_gui/src/battle.rs`, later `region.rs`

## Goal

Playtest 2026-10-23 (note 20261022-204440): "The sprites look weird when engaging unit from top or bottom, may be need
to have sprite facing all 4 directions, is it possible using the rig". Yes: the rig (`tools/sprites/rig.py`) is one
2D skeleton drawn side-on (facing right) with swappable outfit parts; front and back need a second projection of the
same skeleton plus a front and a back drawing of every outfit part. The unit dicts (proportions, part names, palettes)
are reused, so the goblin and named variants mostly come along once their heads and hats have view drawings. This spec
defines the four directions, the sheet and loader contract, the battle rule, the rig work, its cost and a staged plan
with a pilot class for the user's review first.

## 1. What four-way means here

The battle grid is square tiles seen top-down (12 x 9, GRID12; party on the left, foes on the right). A unit faces one
of four screen directions; three drawings cover them:

| Facing | Screen direction | Drawing (view) | Mirrored |
| --- | --- | --- | --- |
| right | toward +x | `side` (today's rows; the sheets face right) | no |
| left | toward -x | `side` | yes (FACE1, as SPRITE2 does for foes today) |
| down | toward +y (the bottom of the screen) | `down`: the front, the face toward the camera | never |
| up | toward -y (the top of the screen) | `up`: the back, the face away from the camera | never |

- The main weapon is in the unit's right hand in every view (the side view's near hand): screen-left in `down`,
  screen-right in `up`. The off-hand (shield, second dagger) is in the left hand. Down and up rows are never mirrored
  (mirroring would swap the hands between turns).
- The token on cards and lists stays the `idle` side clip's first frame (ASSETORG2 4.1); nothing outside the battle
  and region map views changes.

## 2. Options (the scope question)

Question: how far should units turn toward a target above or below them?

| Option | Gameplay effect | Cost | Risk | HISTORY |
| --- | --- | --- | --- | --- |
| A. Front and back views in the rig for `idle`, `walk` and the main attack; other clips fall back to the side view | Units face up and down while standing, walking and striking; a hit or a fall turns them side-on for that clip | Engine S, game S, rig core M, 6 classes and 12 variants (section 7, about 11 rig sessions) | Readability at 28 px (section 8) | none |
| B. No new art: keep FACE1's left/right, add a code lunge of 3 to 4 px toward a target above or below during the attack | Strikes visibly go up or down; the body stays side-on | Game S only (`battle.rs`) | Does not answer "facing all 4 directions"; may still look odd | none |
| C. As A for `idle` and `walk` only; every attack plays the side view | Units face up or down until they strike, then turn side-on | About 60 % of A's rig work | The attack is when the user noticed the problem | none |
| D. Two 3/4 views (front-3/4, back-3/4, mirrored) replacing the side view | Diagonal facings, the isometric look | Redraws every clip of every body; the side rows are dropped | Only right if STAGE1 picks an isometric grid | none |

Recommendation: A. It is what the note asks, the variants inherit most of it, and B's lunge can still come with FX1.
D only makes sense if STAGE1 picks isometric tiles (section 9); the pilot (DIR4-R1) is small enough to redo then.

No HISTORY row touches this. HISTORY row 2026-10-20 "One sprite strip file per clip" (replaced by one sheet per body,
the user wanted a class to be two files) rules out a sheet per direction: the views go in the same sheet (section 3).

## 3. Schema contract (engine, DIR4-E; schema-first: yes)

### 3.1 clips.json: view clips

A view clip is an ordinary row of the body's one sheet with two extra fields:

```json
{"id": "walk_up", "of": "walk", "view": "up", "frames": 8, "ms": [90, 90, 90, 90, 90, 90, 90, 90], "loop": true}
```

| Field | Type | Allowed | Default | Meaning |
| --- | --- | --- | --- | --- |
| `view` | string | `"down"`, `"up"` | absent = a side clip | which drawing this row is |
| `of` | string | the `id` of a side clip in the same list | absent | the clip this row is a view of |

Validation (added to `Clips::parse`; every problem listed, existing rules unchanged):

- V1: `view` present and not `down` or `up`: `clip '<id>': view must be down or up.`
- V2: `view` without `of`, or `of` without `view`: `clip '<id>': of and view go together.`
- V3: `of` does not name a side clip (no `view`) of the list:
  `clip '<id>': of '<of>' is not a side clip of this body.`
- V4: `id` is not `<of>_<view>`: `clip '<id>': a view clip's id must be '<of>_<view>'.` (Ids stay unique snake_case,
  so the existing duplicate rule also stops two `walk` `up` rows.)
- `idle` must still be a side clip (the token). Rows may come in any order; the exporter writes the side clips first
  in today's order, then the view clips (section 6.3), so the existing rows keep their indices.
- A body without view clips is valid and draws exactly as today.

Why fields, not an id suffix alone: a later skill clip such as `wind_up` would otherwise read as a view of `wind`.

### 3.2 Facing and the lookup

New in `aegis_core::assets` (game-agnostic: four-way facing on a square grid):

```rust
pub enum View { Side, Down, Up }
pub struct Facing { pub view: View, pub left: bool }
```

- `Facing::default()` = side, right. `left` is the last horizontal facing; it is kept while the unit faces up or down
  and used for side rows (mirror) and side fallbacks.
- `Facing::toward(self, dx: i32, dy: i32) -> Facing` (the direction rule; `dx`, `dy` in tiles, +y down the screen):
  - `dx == 0 && dy == 0`: unchanged.
  - `|dy| > |dx|`: `Down` when `dy > 0`, `Up` when `dy < 0`; `left` unchanged.
  - otherwise (horizontal, or a diagonal tie `|dx| == |dy|`): `Side`, `left = dx < 0`.
- `Clips::resolve_view(id, view) -> Option<(usize, bool)>`: the row and whether it is a side row. Order: the view row
  of `id`; the side row of `id`; the view row of `idle`; the side row of `idle`; the first row; None for no clips.
  `View::Side` skips the view steps (today's `resolve`).
- `Clips::frame_at_view(id, view, ms)` as `frame_at` on the resolved row (a row of `id` or `idle` animates, any other
  shows its first frame). `ClipFrame` gains `side: bool` (the drawn row is a side row).
- `ClipPlayer::frame_facing(clips, now, view)`: as `frame` with the view; the player keeps side ids (`playing`,
  `held_for` unchanged), so a turn in mid-clip keeps the clip's clock: with equal frames and ms (rule R3) the frame
  index carries over.
- `Art::played_facing(key, palette, player, now, size, facing) -> Option<(ClipFrame, &Image)>`: the frame mirrored
  only when `frame.side && facing.left`. `Art::body_outline` is called with the same flip. `Art::played` stays for
  callers without a facing.

### 3.3 Acceptance checks (engine tests)

1. V1 to V4 each produce their message; a body with side clips only parses as today.
2. A body with `idle`, `walk`, `walk_up`: `resolve_view("walk", Up)` is the `walk_up` row (not side);
   `resolve_view("walk", Down)` is the `walk` row (side); `resolve_view("cast", Up)` is the `idle` side row;
   with `idle_up` added, `resolve_view("cast", Up)` is `idle_up`.
3. `frame_at_view("walk", Up, t)` and `frame_at("walk", t)` give the same frame index for equal frames and ms.
4. `Facing::toward`: (0, 1) is down keeping `left`; (0, -2) up; (2, 1) side right; (-1, -1) side left (tie);
   (-3, 0) side left; (0, 0) unchanged.
5. `played_facing` mirrors a side row when `left` and never mirrors a down or up row; the outline matches the frame.
6. Every committed body draws pixel-identical through `played_facing(side, left = !ally)` and today's
   `played(.., flip = !ally)` (regression on the shipped tree).

## 4. Rules (game, DIR4-G: the battle picks a facing)

Builds on FACE1 (left/right toward the current target, the move direction while walking, party right and foes left by
default). DIR4-G replaces FACE1's left/right flag in `UnitAnim` with a `Facing`:

- G1 Standing: `facing = facing.toward(target - pos)` for the target FACE1 uses (the planned or intended target);
  no target: unchanged. Start of battle: party side right, foes side left (FACE1).
- G2 Walking: on a slide `from -> to` (`Tokens::catch_up`), `facing = facing.toward(to - from)` for the slide; when
  the slide ends, G1 applies again. FACE1's "a vertical-only move keeps the last facing" becomes: it faces up or down
  and keeps `left` for the side clips.
- G3 Acting: the actor faces its action's target tile (`toward(target - pos)`) when its clip starts.
- G4 Drawing: `token_frame` and `token_outline` use `Art::played_facing` with the unit's `Facing`. A clip the body
  lacks in that view falls back to the side clip with `left` (3.2), so a hit or a fall turns the unit side-on for
  that clip and the view returns with the base loop.
- G5 The skip setting (`skip_anim`) still sets the facing (the still frame shows it). No rules, plans or save data
  change; facing is presentation only.

Acceptance (game tests in `battle.rs`):

1. A unit whose target is two tiles straight below draws `idle_down` when its body has it, its side `idle` otherwise.
2. A slide from (5, 4) to (5, 2) plays `walk_up`; from (5, 4) to (7, 5) plays `walk` side, not mirrored; from (5, 4)
   to (3, 3) plays `walk` side, mirrored.
3. A diagonal target at (+1, +1) keeps the side view (tie rule).
4. Hit while facing up: `hurt` side with the last `left`; after it, `idle_up` again.
5. A body without view clips (the dummy, any body before DIR4-R2/R3) draws exactly as FACE1 does.

## 5. Which clips get views

| Stage | Clips | Why |
| --- | --- | --- |
| first (DIR4-R1..R3) | `idle`, `walk`, the main attack: `attack` (warrior, ranger, priest, rogue, mage), `jab` (brawler: `action_clips` plays it for a one-target fist attack) | what a unit shows nearly all the time and when it engages |
| later (DIR4-R4, proposed only) | `hurt`, `death`, `defend`, `idle_guarded`, `hurt_guarded`, `idle_focus`, `shield_bash`, `haymaker`, `kick`, `sweep_kick` | fall back to side until then |

New rows per body in the first stage: 6 (`idle_down`, `idle_up`, `walk_down`, `walk_up`, `<attack>_down`,
`<attack>_up`). Sheets grow from 9 to 11 rows to 15 to 17 rows (at most 816 px tall); widths do not change.

## 6. Assets: the rig (asset lane, DIR4-R1..R3)

### 6.1 Rig rules

- R1 Same skeleton, same unit dict: a new module `tools/sprites/views.py` holds `pose_view(u, view, ...)` and the
  part drawers for `down` and `up`; it reads the keys the side rig reads (`leg`, `arm`, `tl`, `neck`, `scale`,
  `girth`, `head`, `hat`, `mask`, `beard`, `braid`, `torso`, `legs`, `back`, `weapon`, `offhand`, `pauldron`, `pal`).
  `rig.py`'s side code is not changed.
- R2 Frame: 48 x 48, feet on `GY` = 43 (as the side view), pelvis centred at x = 24 (the side view's is at 20, its
  mirror at 27), every offset scaled by `scale` about the feet (as `pose`). The figure (weapons excluded) stays within
  x 12..36 so it keeps to its tile.
- R3 Timing: a view clip has the same frame count and `ms` as its side clip (one view key per side key), so a turn
  in mid-clip keeps the beat and FX1 can hang one impact time per clip. The exporter exits otherwise.
- R4 Palette (as built in DIR4-R1, 2026-10-27: a view may also reuse colours the body's side rows already use, such as
  the attack sparks; a new colour or palette slot still exits): the views draw only with the unit's `pal` colours plus `OUT` and `WHITE`; the body's `clips.json`
  palette (its slots, at most 16) is identical with and without the view rows. The exporter exits naming any colour a
  view adds. Palette files (`palettes/*.json`) are unchanged.
- R5 A body gets view clips only when every part it uses has a view drawer (`views.supports(u)`); otherwise the
  exporter writes it without views and prints which part is missing. The dummy (`dummy/base`) gets none: it is drawn
  front-on already.
- R6 Outline and flash as the side rig (`F.outline`, the white hit flash).

### 6.2 Depth order (back to front)

| Layer | `down` (front, face to the camera) | `up` (back to the camera) |
| --- | --- | --- |
| 1 | back gear past the body: cape hem at the legs' sides, quiver fletchings over the left shoulder | anything held in front of the chest: the shield (its rim and strap side), a weapon held forward |
| 2 | a weapon or shield raised behind the body (wind-up frames) | the forearms in front of the body |
| 3 | legs (the lifted one shortened: knee bent, foot 1 to 2 px up) | legs |
| 4 | robe skirt | robe skirt |
| 5 | torso front: belt, buckle, trim, chest plates | torso back: belt line, plate back, no buckle |
| 6 | both arms at the sides; right arm screen-left | both arms at the sides; right arm screen-right |
| 7 | head front: face skin, two eyes 2 px apart, mouth or beard; hat or helm front | head back: hair, hood or helm back, plume; hat back |
| 8 | weapon and shield in front of the body: shield on the left arm (screen-right) face out, main weapon screen-left | back gear over the body: quiver whole, cape over the torso and upper legs |
| 9 | effects (sparks, slash, bolt, arrow) | weapons raised above the head; effects |

### 6.3 Clip shapes

- `idle_<view>`: 8 frames, 160 ms each (as `rig.idle`): the side idle's bob and breath; hands from the class's ready
  pose (weapon low and forward in `down`, its tip visible past the hip in `up`; the warrior's 1H sword is held upright
  as in the side view instead: pilot review, user 2026-10-28).
- `walk_<view>`: 8 frames, 90 ms each (as `rig.walk`): the feet alternate (each foot lifts and its knee bends in its
  half of the gait), body bob `0.9 - 1.8 * |sin|`, arms swing as 1 px vertical offsets opposite to the legs.
- `<attack>_<view>`: the side attack's keys, one view key each, with the motion turned toward the camera (`down`) or
  away (`up`): a lunge moves the body 1 to 2 px down (`down`) or up (`up`) instead of across; a motion along the facing
  axis (thrust, bow draw, jab, bolt) shows foreshortened, with the impact spark or projectile past the feet (`down`)
  or above the head (`up`), inside the 48 px frame.

| Class | Attack | `down` / `up` motion |
| --- | --- | --- |
| warrior | `attack` (variant A: shield carried lower) | sword drawn back by the right shoulder, thrust down (or up) past the feet (or head) with a tip spark; the shield stays up on the left arm. As built after the pilot review (user 2026-10-28): 1H + kite a stab at chest height (front toward the camera, foreshortened, by the shield edge; back away, the tip past his right side); 2H greatsword a level cut across the chest; neither drops below the upper body; the spark at chest height on the impact frame |
| ranger | `attack` | bow held upright across the chest, string pulled to the chin, release; a 1 x 5 px arrow leaves down (or up) |
| brawler | `jab` | right fist drives 3 px toward the facing and grows 1 px, spark at the fist; the left fist stays at the chin, bobbing |
| priest | `attack` | mace up over the head, smash down in front (`down`: the flash before the feet; `up`: the flash at both sides of the body) |
| rogue | `attack` | right, then left dagger stab toward the facing past the hips, a spark each |
| mage | `attack` | staff raised, gem glows, staff points toward the facing (gem before the chest in `down`), bolt leaves down (or up) |

### 6.4 Exporter and review (`export_units.py`)

- Writes the view rows after the side rows, each with `of` and `view` (3.1); `loop` as its side clip.
- `--review OUT` adds per body: a turnaround board (rows `idle`, `walk`, the attack; columns side right, side left
  mirrored, `down`, `up`) at 3x and at the battle's 28 px token size (`Art::body_clip`'s downsample), and GIFs of
  `walk` in the four directions and the attack in `down` and `up`.
- Readability check (printed, the exporter exits on failure): at the 28 px downsample, `idle_down` and `idle_up`
  frame 0 differ in at least 10 % of their opaque pixels.

### 6.5 Asset acceptance (each DIR4-R task)

1. Each target body's `clips.json` lists the six view clips with `of`, `view`, the side clip's frames and `ms`.
2. The side rows are pixel-identical to an export from before the task on the same machine (not byte-identical
   files: HISTORY 2026-10-22, library versions). Waived once in DIR4-R1 for `warrior/2h_sword`'s `attack` row (user
   2026-10-28: the greatsword's side attack changed from the chop to the chest cut, `classes.warrior_2h` greatsword
   branch; the axe and hammer keep the chop).
3. The `palette` object is unchanged; palette files are unchanged.
4. No blank frame (`aegis_ui::art::blank_frames`); the body checks and content checks are clean.
5. The review page shows the turnaround boards; the user's sign-off is recorded for DIR4-R1.

## 7. Cost per body

Units: one rig session = one asset-lane run with its review export. Rough estimates; the pilot calibrates them.

Parts that need a `down` and an `up` drawing: heads `helm`, `hood` (+ `mask`), `headband`, `cowl`, `hat`, `bare`,
`kettle`, `cap` (+ `beard`, `braid`), `goblin` with hats `band`, `bandana`, `hood`, `mohawk`, `skull`, `horns`,
`crown`; torsos `plate`, `leather`, `bare`, `robe` (+ `girth`, `pauldron`); legs `trousers`, robe skirt; back gear
`quiver`, `cape`; weapons `sword` (`sword2` is its long form), `shield`, `bow`, `mace`, `dagger`, `staff`, `fists`.

| Body | New parts | Clips keyed | Size | Sessions |
| --- | --- | --- | --- | --- |
| rig core (`views.py`: `pose_view`, idle and walk, exporter, review boards) | skeleton, legs, arms | idle, walk (shared) | M | 1.5 |
| warrior/base | helm, plate, trousers, sword, shield | attack (thrust) | L | 1.5 |
| ranger/base | hood, leather, quiver, bow | attack (draw) | L | 1.5 |
| brawler/base | headband, bare torso, fists | jab | M | 1 |
| priest/base | cowl, robe and skirt, mace | attack (smash) | M | 1 |
| rogue/base | mask, cape, daggers (hood, leather shared) | attack (double stab) | M | 1 |
| mage/base | hat, staff (robe shared) | attack (bolt) | M | 1 |
| goblin head (once, used by 8 bodies) | goblin head front and back | none | M | 0.5 |
| brawler/goblin, ranger/goblin | band | inherited | XS | 0.2 |
| brawler/goblin_berserker | mohawk, girth | inherited | S | 0.25 |
| rogue/goblin | bandana | inherited | S | 0.25 |
| rogue/goblin_scout | goblin hood, no cape | inherited | S | 0.25 |
| priest/goblin | skull cap | inherited | S | 0.25 |
| warrior/grakka_ashmaw | crown, cape on a warrior, sword2 | inherited | S | 0.25 |
| brawler/ash_warchief | horns, pauldron | inherited | S | 0.25 |
| warrior/brask_toller | kettle, beard | inherited | S | 0.25 |
| warrior/hest_the_quartermaster | cap (beard shared) | inherited | S | 0.25 |
| warrior/orsa_tallow, ranger/wenn_tallow | bare, braid | inherited | S | 0.25 |
| dummy/base | none (drawn front-on) | none | 0 | 0 |

Totals before weapon looks: 6 classes with the core about 8.5 sessions; 12 variants with the goblin head about 3
sessions; engine (DIR4-E) and game (DIR4-G) about 0.5 to 1 session each. Variants cost little because `variants.py`
bodies call `classes.clips(cls, unit=u)`: the class's view keys play on the variant's dict. Section 7.1 adds the
warrior's 2H look: about 1.5 more sessions, so about 13 rig sessions in all.

### 7.1 Weapon looks (WAR2H1, user 2026-10-23)

WAR2H1 (built now, side views only) gives the warrior two looks chosen by equipped gear: one-hand plus shield
(today's sheet) and two-handed without a shield, the 2H weapon drawn by its family (greatsword, battle axe,
warhammer); a 2H weapon or an empty off-hand gives the 2H look. Grakka Ashmaw, Hest, Brask and Orsa drop the shield.

What multiplies, and what it costs. A look is a grip (a pose set: how the hands hold the weapon, keyed per clip)
plus a weapon drawing:

| Thing | Rig work per view | Count for the warrior | Front and back cost (`down`, `up`) |
| --- | --- | --- | --- |
| grip (pose set): `1h_shield`, `2h` | idle and walk parameters plus the attack keys | 2 | `1h_shield` is section 7's warrior row; `2h` adds about 1 session (an overhead chop keys more easily front-on than the thrust) |
| weapon drawing: `sword`, `shield`, `greatsword`, `battle_axe`, `warhammer` | one part drawer | 5 | the three 2H heads add about 0.5 session (shared haft and grip, three heads) |
| baked sheet (body x look) | none: the exporter loops | 4 per warrior-class body (1H plus shield and three 2H), or fewer (Q8) | none (machine time and about 60 KB a sheet) |

Rule of thumb: work grows with grips and weapon drawings, each times 3 views (side, `down`, `up`); bodies and looks
only multiply the sheets the exporter writes. A new weapon family on an existing grip costs about 0.25 session (its
three drawings); a new grip costs about 1 session for the side view and 1 for `down` and `up`.

Other per-weapon looks the families allow (none asked for; listed so the look matrix has room): ranger bow or
crossbow (a new grip), mage staff or wand with orb or tome (a new grip), the 1H look drawn by family (axe or mace
with the shield: same grip, two more drawings), priest mace or staff (the staff grip exists on the mage).

How to keep the combinations manageable (options):

| Option | How | Cost | Risk |
| --- | --- | --- | --- |
| W1. Bake every look (recommended) | The rig already draws each frame's parts in depth order (the shield between the far arm and the torso, the fist over the grip); the exporter writes one sheet per body and look from a look matrix in `variants.py` (which bodies get which looks); all looks of a body share its palette slots, so one palette file recolours every look | none in the engine beyond WAR2H1's look pick; work per section 7 and this table | more files (a body folder holds several sheets); bounded by the look matrix |
| W2. Weapon as a runtime layer | Bodies exported weaponless in 2 or 3 depth layers per frame (behind, body, front hand), weapon sprites per family pre-rotated at 16 angles per view, a per-frame anchor (hand x and y, angle, layer) in `clips.json`; the engine composites and outlines | engine M to L (a layered body format, compositing, outline after compositing, palettes per layer); every body re-exported | pixel-art weapons at in-between angles and fist-over-grip overlaps look worse than baked frames; pays off only with many unique weapon sprites (per item) |
| W3. W1 plus per-frame anchors | As W1; the exporter also writes each frame's weapon tip and hand point (a small optional `anchors` list per clip) | small (the rig knows both points) | none for DIR4; FX1 could start projectiles and swing trails at the tip, and W2 stays possible later |

Recommendation: W1 now; W3's anchors only if FX1 wants them (Q9). Revisit W2 if items ever get their own weapon
sprites (then bodies x items explodes and runtime layers win).

Rules for DIR4 with looks:

- L1 The view fields (3.1) live inside each look's `clips.json`; DIR4's lookup applies after WAR2H1's look pick, which
  does not change. Whatever WAR2H1 uses to store a look (its own sheet, or a body key per look), every look sheet gets
  the same six view clips.
- L2 A look gets view clips only when its grip has view keys and its weapon has view drawers (R5 per look); otherwise
  that look draws side views and the others still turn.
- L3 All looks of one body share the same palette slots (R4 per look), so the character and ash palette files work
  on every look.
- L4 2H depth order: `down`, the weapon is held in both hands before the torso (layer 8), its head above the
  screen-left shoulder in the ready pose; `up`, the haft is hidden by the body except the head over the right
  shoulder and the pommel below the hands (layers 1 and 9). As built (pilot review, user 2026-10-28): in `up` the
  hands, grip and pommel stay hidden behind the body; only the blade over the right shoulder shows.
- L5 The pilot (DIR4-R1) does both warrior grips with one 2H family (greatsword); the battle axe and the warhammer
  come with DIR4-R2.

Cost per warrior-class body with looks:

| Body | Looks (after WAR2H1) | Extra DIR4 work beyond section 7 |
| --- | --- | --- |
| warrior/base | `1h_shield`, `2h` greatsword, battle axe, warhammer | the 2H grip views (about 1) and the three 2H heads (about 0.5) |
| warrior/grakka_ashmaw | `2h` (greatsword, his `sword2`) | none (inherits the grip and the drawing) |
| warrior/hest_the_quartermaster, warrior/brask_toller, warrior/orsa_tallow | `2h` (family per Q8) | none |

## 8. Risks

- Readability at 28 px: the battle fits the 48 px frame into a 28 px token by downsampling (`body_clip`, k = 0), about
  0.58x: a figure about 19 px tall, a face about 4 px wide; 1 px eyes can drop out. Front and back must differ in
  colour mass (face skin and buckle against hair, hood or cape), not single pixels; the review shows the 28 px render
  and the exporter's 10 % check (6.4). STAGE1 may change the token size (section 9).
- Region map: PLAYUI1-R (approved, not built) draws the leader and NPCs at 24 px tall on 16 px tiles (0.5x) and
  flips by the last horizontal step, keeping the facing on up or down steps. Walking up and down is the main use of
  four-way facing there; the same `Facing::toward` on each step gives it (row DIR4-M). Smaller still, so the same
  readability check applies.
- Side-on pop: a hit or a fall while facing up or down turns the unit side-on for that clip (G4). Accepted for the
  first stage; DIR4-R4 removes it.
- Asymmetric gear: a left-facing unit (mirrored side) holds its weapon in the far hand; down and up show it in the
  right hand. This is the usual sprite cheat; a turn from left to down moves the weapon across. Accepted.
- Pixel drift: the side rows must not change (6.5 check 2); `views.py` is a new module so `rig.py` stays untouched.
- Look sprawl: every new grip costs about 2 sessions across three views and every new family about 0.25 (7.1); the
  look matrix in `variants.py` is the one place that says which body bakes which look, so additions are reviewed.

## 9. Interaction with STAGE1 (camera)

| STAGE1 camera | Effect on DIR4 |
| --- | --- |
| Top-down flat (today) | fits: front and back views read as facing the camera or away |
| 3/4 view on the same square grid (tiles with a visible south face, upright props drawn in row order with the units) | fits best: the classic upright sprite with four facings; units drawn in row order (as PLAYUI1 2.4) |
| Isometric (diamond tiles) | does not fit: grid axes run diagonally on screen, so facings are diagonal and need front-3/4 and back-3/4 drawings (option D); DIR4-R1's views would be redrawn and the side rows would go unused in battle |

So the camera family should be settled before the full rollout (DIR4-R2, DIR4-R3). DIR4-E and DIR4-G hold for every
camera (the views stay named `down` and `up`; an isometric pick would map them to front-3/4 and back-3/4 and add a
mirror rule). DIR4-R1's warrior can appear in STAGE1's mockups so the user judges both together.

## 10. Staged plan

1. DIR4-E (engine): the view fields, the lookup, `Facing`, `Art::played_facing`, tests (3.3). Runs beside FACE1
   (no shared files).
2. DIR4-R1 (pilot, asset): `views.py` core and warrior/base end to end (idle, walk, attack in `down` and `up`) in
   both looks, `1h_shield` and `2h` with the greatsword (7.1 L5), the review page; after WAR2H1 (its 2H side keys)
   and DIR4-E are merged (before that, `--review` only).
3. DIR4-G (game): the battle facing (section 4), after FACE1 and DIR4-E; with R1 merged the warrior turns in battle.
4. User review of the warrior in battle and on the review page (Q3), together with STAGE1's camera pick (Q4).
5. DIR4-R2: ranger, brawler, priest, rogue, mage; the warrior's battle axe and warhammer views.
6. DIR4-R3: the goblin head, the 7 hats and the named heads; the 12 variant bodies re-exported.
7. DIR4-M: the region map uses `Facing` (after PLAYUI1-R).
8. DIR4-R4 (proposed only): views for the secondary clips.

## 11. Out of scope

- Diagonal (8-way) facings; isometric 3/4 views (option D) unless STAGE1 picks isometric.
- The stage look, tile depth, contact shadows (STAGE1); projectiles, swing trails and impact sparks drawn in code (FX1).
- Turning a hit unit toward its attacker; any rules, plan, AI or save change.
- Portraits, the token on cards and lists, palette files.
- Secondary clips' views (DIR4-R4, not approved).
- How WAR2H1 stores and picks a look (its own spec); new looks beyond the warrior's (7.1 lists candidates only); a
  runtime weapon layer (7.1 W2).

## 12. Open questions (recommended answer first)

1. Scope: A (front and back for idle, walk and the main attack), B (code lunge only), C (idle and walk only) or D
   (3/4 views)?
2. Pilot body: the warrior alone (hardest gear: helm, plate, sword and shield in depth order), or the warrior plus the
   goblin brawler (the common foe and the goblin head; adds about 1.5 sessions: brawler class and goblin head)?
3. Review stop: pause after the pilot for your review (recommended: it sets the look for about 13 rig sessions), or
   build R2 and R3 straight on?
4. Order with STAGE1: hold R2 and R3 until STAGE1's camera is picked (recommended), or roll out now on the top-down
   assumption?
5. Diagonal tie (a target one across and one down): side view (recommended: the sides face each other across the
   grid), or the vertical view?
6. Region map: four-way facing there too through DIR4-M after PLAYUI1-R (recommended), or fold it into PLAYUI1-R if
   DIR4-E lands first?
7. Weapon looks: bake every look per body (W1, recommended), or a runtime weapon layer in the engine (W2)?
8. The named warriors (Grakka, Hest, Brask, Orsa) without the shield: bake only the 2H family their character
   carries (recommended: Grakka the greatsword; the others per their gear), or all three 2H families since the look
   follows equipped gear? And if one of them equips a 1H weapon and a shield, show the 2H look anyway (recommended,
   they dropped the shield) or bake the `1h_shield` look for them too?
9. Per-frame weapon anchors (W3) for FX1's projectiles and swing trails: add them with DIR4-R1 (recommended only if
   FX1's spec wants them), or leave them to FX1?
