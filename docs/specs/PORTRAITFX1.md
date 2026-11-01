# PORTRAITFX1: animated, reactive portraits

Status: proposed (spec 2026-10-25, from the user decision "Animated, reactive portraits (user 2026-10-25)" in
`games/emberwake/docs/decisions/classes-units.md`: (A) code effects now, (B) expression sheets for the main cast
next, (C) layered puppets later as an idea)
Depends on: FX1-E (done: `aegis_ui::fx` clock, `hash01`, `Shake`/`Camera`, `Settings::reduce_motion`), STORYUI1-E
(done: the vignette player), KIT1-N (kit paths), DERIVE1-S (derived rows), ART3 (the portraits); per task in
[PORTRAITFX1_TASKS.md](PORTRAITFX1_TASKS.md) section 3
Lane: logic (E engine, S kit, T and XT tools, G, D, R, P, TL and X game or engine + game), asset (A, XA)
Touches: per task in [PORTRAITFX1_TASKS.md](PORTRAITFX1_TASKS.md) section 3; overall
`engine/crates/aegis_core/src/assets.rs`, `engine/crates/aegis_ui/src/{portrait_fx (new),art,timeline,vignette,
specimens,lib}.rs`, `kits/rpg/schema.json`, `kits/rpg/crates/rpgkit_{data,content,battle}/src/`,
`games/emberwake/crates/emberwake_gui/src/{portraits (new),region,story,party,hub,battle,gallery}.rs`,
`tools/portraits/{face (new),generate,sheets}.py`, `games/emberwake/data/catalog/portrait_cues.json` (new, through
`aegis content`), portrait sidecars `*.face.json` and expression frames under the game and kit `portraits/` folders
Companion: [PORTRAITFX1_TASKS.md](PORTRAITFX1_TASKS.md) (acceptance checks, asset tables, task rows, sequencing,
docs delta, HISTORY and roadmap rows)

History check (`docs/HISTORY.md`): no row on animated portraits, expressions, blinks or puppets. The 2026-10-20 row
"a portrait cropped from the token's idle frame" (replaced by ASSETORG2's separate detailed portraits) is not
re-proposed: effects draw over the detailed portraits, and expression frames are new detailed art.

## 1. Goal

Every portrait in the game feels alive and reacts: it breathes, its lantern rim light flickers, it blinks; corrupted
characters carry a sinister overlay (a pulsing marrow glow, creeping violet veins, burning eyes, drifting ash or
smoke); and a portrait answers what happens to its unit: a flash and a jolt on a hit, a stronger shake on a crit, the
colour draining as the unit is wounded, a cold cast when its Composure is shaken, a fade while it is stealthed. All of
it is drawn in code over the existing 42 portraits (A), as a reusable engine piece with a UI Gallery page; then the
main cast gains true faces from 2x2 expression sheets (B) that plug into the same layer with the single portrait as
the fallback. Presentation only: no rule, plan, AI, save, trace or sim changes.

## 2. Survey (develop at 397a567)

| Area | Where | Today |
| --- | --- | --- |
| Portrait files | `games/emberwake/assets/portraits/{heroes,npcs,elites,bosses,common}/` (29), `kits/rpg/assets/portraits/common/` (13) | 42 PNGs, all 240 x 240: a 120 px grid figure (PIXEL2, 2026-10-26; was 96 at 2.5x) scaled 2x by `pixelize.py`, on an opaque dithered background glow, framed by a 3 grid px border (about 8 px at 240); no mask, no sidecar |
| Lookup | `kits/rpg/crates/rpgkit_content/src/art.rs` `portrait_tier`, `portrait_chain` | Unique tier `portraits/<folder>/<id>.png`, then the body's common portrait, then the class base |
| Where they draw | `emberwake_gui/src/region.rs` `draw_talk` (64 px, `screens::character_art`); `aegis_ui/src/vignette.rs` line cards (`PORTRAIT` 64) | Nowhere else: the Character screen (`party.rs`), the hub list (`hub.rs`), the battle hero and intel panels (`battle.rs`) and the timeline strip show no portrait |
| Effects kit | `aegis_ui/src/fx.rs` (FX1-E) | `Clock` with hit-stop, `hash01`, `Shake`, `Camera::offset`, `SHAKE_PRESETS`; `Settings::reduce_motion` in `screen.rs` |
| Frame loop | `aegis_ui/src/shell.rs` | Redraws every frame (Poll, or the `fps_cap`), so a portrait effect must cache its composed frame |
| Unit state | `rpgkit_rules/src/status.rs` (`u.st.active`), RULESET1 GD5 (`stealth`, RULESET1-S), ROSTER1 W1 (`conditions`, `wounded`), DERIVE1 (`composure`, `status_modifiers` [] in the slice) | No per-unit corruption meter before Act 2 (CAMPAIGN1 M9 reserves unit-scope meters for CAMPAIGN1-DOC) |
| Image API records | `art_src/<image>.json` (ART3) | `generate.py` writes the prompt record beside every image; `sheets.py` cuts sheets; `pixelize.py` makes the 240 px portrait |

## 3. Layers

| Piece | Layer | Where | Why |
| --- | --- | --- | --- |
| The face sidecar format (anchors in image px) and its checks | `[engine]` | `aegis_core::assets` (beside clips and palettes) | Any game with portraits needs the same anchors; the kit check and the UI read one parser |
| The portrait effect layer: effects, composition, timing, seeds, size tiers, reduce motion, budget, agent state | `[engine]` | `aegis_ui::portrait_fx` (new) | Technology any game needs; story-free |
| Kit widgets page "Portraits" with a code-drawn sample face | `[engine]` | `specimens.rs` | The standalone-panes rule: judged on its own in the UI Gallery |
| A reserved icon square in timeline rows | `[engine]` | `timeline.rs` | The widget stays portrait-free; the caller draws into the square |
| The cue table: game event, status, condition, HP or derived stat to effect, intensity, duration | `[kit hook]` | schema `content.portrait_cues`, `rpgkit_data`, `rpgkit_content/src/check/portrait.rs`, `rpgkit_battle/src/portrait.rs` | Any story in the genre reacts to hits, wounds and statuses; the rows are data |
| Battle step events per unit (hit, crit, guard hit, heal, break) | `[kit hook]` | `rpgkit_battle/src/portrait.rs` | Read-only over a step's HP and log |
| Expression names and the face chain | `[kit hook]` | `rpgkit_content/src/art.rs` | The kit owns the portrait file layout (ASSETORG2) |
| Per-portrait anchors and idle overlays | `[data]` (asset sidecar) | `<portrait>.face.json` beside each PNG, game and kit layers | Tied to the picture's pixels; changes when the image changes, like `clips.json` beside a sheet |
| The cue rows | `[data]` | `games/emberwake/data/catalog/portrait_cues.json` through `aegis content` | Named by the game (stealth, wounded, Composure) |
| The binding: building a unit's look, per-screen portrait sets, the surfaces, the game Gallery page, colours | `[game code]` | `emberwake_gui/src/portraits.rs` (new) and the screens | GUI code stays in `emberwake_gui` (KIT1 KQ4) |

Sidecar versus `aegis content`: the anchors describe pixels of one image file, not a game entity; one common
portrait serves many units, and a re-pixelized image needs new anchors. So they live beside the image (as
`clips.json` and `palettes/*.json` do beside a body sheet), written by a tool (`tools/portraits/face.py`) and checked
by the asset checks. The cue rows are game content and go through `aegis content`.

## 4. Engine

### 4.1 The face sidecar (`aegis_core::assets`, PORTRAITFX1-E)

File: the portrait's path with `.png` replaced by `.face.json` (`assets::face_file(rel)`), in the same layer as the
PNG. All coordinates are image px of the PNG (240 for today's portraits); unknown keys are refused.

```json
{"size": 240, "border": 8, "eyes": [[98, 92, 12, 6], [128, 90, 12, 6]], "blink": true, "lid": "",
 "glow": [[120, 200, 30]], "veins": [[140, 120, 200, 40]], "mouth": [], "rim": "",
 "idle": [{"effect": "eye_glow", "color": "#c084fc", "intensity": 0.8}]}
```

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `size` | int | required | 16-1024; equals the PNG's width (else a warning: anchors are scaled by width / `size`) |
| `border` | int | `round(size / 30)` | 0-`size / 8`; effects never draw on or move the outer `border` px |
| `eyes` | `[x, y, w, h]` list | [] | 0-2 boxes inside the inner rect; `w` 2-`size / 6`, `h` 1-`size / 10` |
| `blink` | bool | true | false for masks, lidless creatures and empty sockets |
| `lid` | `#rrggbb` or "" | "" | "" samples each box's lid colour 1 px above its top edge |
| `glow` | `[x, y, r]` list | [] | 0-3; centre inside the inner rect; `r` 4-`size / 2` |
| `veins` | `[x, y, deg, len]` list | [] | 0-3 vein roots; root inside; `deg` -180-360 (0 = right, 90 = down); `len` 6-`size / 2` |
| `mouth` | `[x, y, w, h]` or [] | [] | reserved for (C); parsed and checked, read by nothing |
| `face` | `[x, y, w, h]` or [] | [] | PORTRAITFX1-EX (section 16): the face box of the expression effects; inside the inner rect, at least `size / 8` a side; [] = taken from the eye boxes |
| `rim` | `#rrggbb` or "" | "" | the rim light's key colour; "" = the caller's default |
| `idle` | list of `{effect, color, intensity}` | [] | 0-4; `effect` in `IDLE_OVERLAYS` (`glow`, `veins`, `eye_glow`, `ash`, `smoke`); `glow` needs a `glow` anchor, `veins` a root, `eye_glow` an eye box; `color` `#rrggbb` or "" (the effect's default); `intensity` 0.05-1.0, default 1.0 |

`assets::parse_face(v) -> Result<Face, Vec<String>>` and `assets::face_problems(face, png_w) -> Vec<String>` (every
rule above; the kit check A11 calls it). `Art::face(rel) -> Option<&Face>` reads and caches the sidecar of a PNG
path; `Art::first_rel(chain) -> Option<String>` returns the first path of a chain a layer holds (so the caller knows
which file, and which sidecar, it drew).

### 4.2 Effects (`aegis_ui::portrait_fx`, PORTRAITFX1-E)

`EFFECTS` (the cue vocabulary): `flash`, `jolt`, `desaturate`, `tint`, `fade`, `still`, `glow`, `veins`, `eye_glow`,
`ash`, `smoke`, `face`. The base life (`breath`, `rim`, `blink`) is not a cue: it runs on every portrait whose size
tier allows it, unless `still` holds. `PORTRAIT_JOLTS`: `hit`, `crit`. Positions below are sidecar px scaled to the
drawn image; `side` is the composed image's side in device px; `t` is the portrait clock in ms; `i` is the
intensity; `h(k)` is `fx::hash01(seed, k)`.

| Effect | Kind | Look | Defaults |
| --- | --- | --- | --- |
| `breath` | base | The inner rect is stretched vertically, anchored at its bottom edge, by `a = amp (1 - cos(2 pi t / period)) / 2`, rounded to whole device px (nearest sampling); the border never moves | `amp` = max(1, round(0.012 side)) device px; `period` = 3600 x (0.9 + 0.2 h(1)) ms |
| `rim` | base | Pixels whose hue is within 18 deg of the rim key, saturation at least 0.45 and value at least 0.45 get value x `f(t)`, `f = 1 + 0.12 (0.6 sin(2 pi t / 1700) + 0.4 (2 h(100 + floor(t / 90)) - 1))` | key = sidecar `rim`, else the caller's default, else none |
| `blink` | base | Needs eye boxes and `blink`. Blink n starts `2600 + 3400 h(200 + n)` ms after blink n - 1 ends (the first after `t` 0); 12 % of blinks (`h(300 + n) < 0.12`) add a second one 220 ms later. 130 ms: 0-40 the top half of each box filled with the lid colour, 40-90 the whole box plus a 1 device px line in lid x 0.6 at two thirds of its height, 90-130 the top half again | lid = sidecar `lid`, else the sampled colour |
| `glow` | overlay | At each `glow` anchor three filled ellipses (radii r, 0.66 r, 0.33 r; alpha 40, 60, 90 x `i` x pulse), pulse = 0.6 + 0.4 sin(2 pi t / 1800); with no anchor, one at (0.5, 0.7) of the inner rect, r = 0.15 size | colour `#9333ea` |
| `veins` | overlay | From each root a trunk of 3 px segments, each turned `25 (2 h(400 + k) - 1)` deg from the root's `deg`, up to `len`; two branches at 40 % and 70 % of the trunk, plus and minus 35 deg, 0.4 `len`, same rule; 1 device px wide (2 at full tier above 192 px); revealed over 1800 ms from when it starts (trunk first), then alpha 255 x `i` x (0.6 + 0.4 sin(2 pi t / 2400)); clipped to a circle of `len` around the root and the inner rect | colour `#9333ea` |
| `eye_glow` | overlay | At each eye box a core ellipse (w / 2, h / 2) at alpha 200 `i` and a halo (0.9 w, 1.1 h) at alpha 70 `i`; a flare (+40 % alpha for 140 ms) every `3000 + 4000 h(500 + n)` ms; drawn after the blink lids, so eyes burn through a blink | colour `#e11d48` |
| `ash` | overlay | `n` = min(32, round(18 `i`)) specks of max(1, side / 96) device px; speck k at x = `h(600 + k)` of the inner width plus a 2 % sway (sin(t / 900 + k)), falling `(0.03 + 0.03 h(700 + k))` side per second, wrapping inside the inner rect; alpha 160 `i` | colour `#6b7280` |
| `smoke` | overlay | 4 blobs rising from the inner bottom edge at 0.04 side per second, radius (0.05 + 0.04 h(800 + k)) side, alpha 50 `i` x (1 - the height reached / the inner height), wrapping | colour `#6b7280` |
| `desaturate` | state | `out = lerp(c, lum, i)`, lum = 0.299 r + 0.587 g + 0.114 b | - |
| `tint` | state | `target = min(255, lum / 255 x col x 1.25)` per channel; `out = lerp(c, target, 0.6 i)` | colour `#06b6d4` |
| `fade` | state | The composed image's alpha x (1 - 0.7 `i`) (never below 0.3) | - |
| `still` | state | Stops `breath`, `rim` and `blink` while it holds (a downed unit) | - |
| `flash` | reaction | The inner rect filled with the colour at alpha 200 `i` (1 - u) over `ms` | colour `#ffffff`, `ms` 160 |
| `jolt` | reaction | The whole portrait (border included) offset by `fx::Camera::offset` of the preset; never the caller's panel or hit rect | `hit` 2 px, 180 ms, axis x, ease_out, step 30; `crit` 3 px, 320 ms, both, ease_out, step 30; ceiling 4 px, 600 ms |
| `face` | state or timed | Names the expression the caller draws (section 7); the engine only reports it (`expression(now)`) | "" = neutral |

Engine default colours are plain numbers; the game passes its art-bible hues through the cue rows, the sidecars and
its defaults (section 6.4).

### 4.3 Composition order

1. The image: the PNG the caller's chain resolves to (`Art::first_rel`), at the composed size.
2. `breath` (unless `still`).
3. Colour, per pixel inside the border: `rim` (unless `still`), `desaturate`, `tint`.
4. `blink` lids (unless `still`).
5. Overlays: `glow`, `veins`, `eye_glow`, in that order.
6. `ash`, then `smoke`.
7. `flash`.
8. `fade` multiplies the alpha of the whole composed image.
9. Drawn into the caller's rect, moved by the `jolt` offset (whole logical px, so the pixel art stays on the grid).

Overlays and colour operations never touch the border; the border moves only with `jolt`.

### 4.4 Cues, holds and reactions

- A `Cue` is `{effect, intensity, ms, color, preset, expression, priority, key}`; `Cue::parse(name, ...)` returns None
  for an unknown effect or jolt preset (the game drops it).
- `hold(cues, now)` sets the state cues that hold now (from the sidecar's `idle` and the kit's state rows); it
  replaces the previous set. A cue key that appears eases in from 0 over `EASE_MS` (250); one that disappears eases
  out over `EASE_MS`.
- `fire(cue, now)`: `flash` and `jolt` run once; any other effect with `ms` above 0 runs for `ms` at
  `i x (1 - u)` (a `face` cue holds its expression for `ms`, no fade). A newer `jolt` replaces the running one only
  when its amplitude is at least the running one's current amplitude (as `fx::Camera::start`).
- Combination: per effect the strongest intensity wins (holds, eases and timed cues together; never summed); its
  colour comes with it (ties: the first key in sort order). `face`: the highest `priority`, then the latest started.
- `expression(now) -> &str` is the winning `face` cue's expression, or "".

### 4.5 Timing and seeds

- `PortraitFx::new(key)` seeds from FNV-1a 64 of the key (the character id, else the unit id, else the path). Every
  random choice is `fx::hash01(seed, k)` with the `k` ranges of 4.2; nothing reads or advances any game RNG.
- The caller passes `now` in ms from a clock it advances: the battle passes FX1's clock (hit-stop pauses portraits,
  `anim_speed` scales them); other screens own an `fx::Clock` advanced by the frame's wall time (at most 100 ms a
  frame, as the vignette screen). Two draws at the same `now` give identical pixels.
- Frame timing (as built: see section 11, `tq(now) = floor(floor(3 now / 100) x 100 / 3)`): time-varying effects read `tq = floor(now / 33) x 33`, so a portrait recomposes at most 30 times a
  second whatever the frame rate.

### 4.6 Size tiers (by the drawn rect's side in device px)

| Tier | Side | Effects drawn |
| --- | --- | --- |
| full | 96 and up | all |
| reduced | 48-95 | no `breath`, `veins` or `smoke`; `ash` count halved; `blink` only when a box is at least 2 device px tall after scaling; `jolt` px halved (at least 1) |
| minimal | under 48 | `flash`, `desaturate`, `tint`, `fade`, `face` and the `eye_glow` core only |

A 32 logical px panel portrait is reduced at 2x and full at 3x; a 64 px dialogue portrait is full from 2x up.

### 4.7 Reduce motion (`Settings::reduce_motion`, FX1 FQ4: motion stops, information stays)

| Effect | Default | Reduce motion |
| --- | --- | --- |
| `breath`, `jolt` | drawn | none |
| `rim` | flickers | steady (f = 1) |
| `glow`, `veins` | pulse; veins grow | steady at pulse 0.8; veins at full length |
| `eye_glow` | flares | no flare |
| `ash`, `smoke` | drift | not drawn (as vignette V11) |
| `blink`, `flash`, `desaturate`, `tint`, `fade`, `still`, `face` | drawn | drawn (eases kept) |

### 4.8 Performance limits

- Composition runs on the CPU at the drawn side in device px, capped at 256 per side (a bigger rect composes at 256
  and scales up, nearest).
- Each `PortraitFx` caches one composed image with its key (path, side, `tq` when a time-varying effect is live,
  every live intensity quantized to 1/32, the jolt-free flash level); it recomposes only when the key changes. A
  portrait with nothing time-varying (reduce motion, minimal tier, no reaction running) composes once.
- `Budget` (one per screen, `begin()` at the start of its draw): at most `MAX_ANIMATED` 8 time-varying portraits per
  frame; the 9th and later in draw order draw their last composed image (frozen time, current states) until a slot
  is free.
- Memory: one composed image per `PortraitFx` (at most 256 x 256 x 4 bytes) plus the source in `Art`'s cache.
- Never in the sim, the host or headless tests of rules: only screens create `PortraitFx`.

### 4.9 API (game-agnostic)

`PortraitFx::new(key)`, `hold(&[Cue], now)`, `fire(Cue, now)`, `expression(now)`, `draw(p, art, rel, face, r, now,
opts, budget) -> bool` (false when `rel` loads nothing, so the caller draws its fallback; `face` the sidecar or None;
`opts` = `{reduce_motion, rim_default: Option<Rgba>, show_anchors: bool}`), `state(now) -> Value`, `sample() ->
(Image, Face)` (the code-drawn sample face of 4.10).

### 4.10 Kit widgets page "Portraits" (`specimens.rs`)

`specimens::PAGES` gains "Portraits": the engine's sample face (`sample()`: a 96 x 96 code-drawn bust on a dark
glow with a 3 px border, two 5 x 3 eye boxes, an amber rim column on the head's right edge, one glow anchor on the
chest, one vein root on the cheek) drawn at a chosen size; toggles for each overlay (`kit.portrait.glow`, `.veins`,
`.eye_glow`, `.ash`, `.smoke`) and state (`kit.portrait.desaturate`, `.tint`, `.fade`, `.still`), buttons for the
reactions (`kit.portrait.flash`, `kit.portrait.jolt.hit`, `kit.portrait.jolt.crit`), a size choice 96 / 64 / 48 /
32 / 24 / 16 (`kit.portrait.size`), a local reduce-motion check (`kit.portrait.reduce`) and an anchors check
(`kit.portrait.anchors`: 1 px outlines of the eye boxes, glow circles and vein roots). The foot line names the tier
and what is live.

### 4.11 Agent link

`state(now)`: `{"key", "expression", "tier", "effects": [names live], "jolt": [dx, dy]}`. Screens that draw portraits
add `"portraits": [...]` to their `state`, so a test or an agent can wait for a reaction to end. No widget id changes.

## 5. Kit

### 5.1 Schema: `content.portrait_cues` (PORTRAITFX1-S; schema-first: yes)

File `data/catalog/portrait_cues.json`, read beside `CATALOG_KINDS` as `derived_stats.json` and `rulesets.json`
are (DERIVE1-S precedent), written with `aegis content upsert portrait_cues @file`.

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | id (key field) | "" | `[a-z][a-z0-9_]{1,31}`; unique |
| `trigger` | enum | `event` | `event`, `status`, `condition`, `hp`, `derived`, `down`; `meter` is refused ("unit meters: CAMPAIGN1-DOC") |
| `event` | enum | "" | required for `event`: `hit`, `crit`, `guard_hit`, `heal`, `break`; else "" |
| `status` | string | "" | required for `status`: an id of the status table (`status.rs`) |
| `condition` | string | "" | required for `condition`: `[a-z_]{1,24}`; a warning when it is not a kit condition id (ROSTER1: `wounded`) |
| `stat` | string | "" | required for `derived`: a `derived_stats` row id |
| `below` | int | 0 | `hp`: 1-100 (HP percent of the current max, strictly below); `derived`: 1-100 absolute; else 0 |
| `drop_pct` | int | 0 | `derived` only: 1-100; exactly one of `below` and `drop_pct` on a `derived` row |
| `effect` | enum | `flash` | a name in the engine's `EFFECTS` (the kit keeps a copy; a test checks it equals `aegis_ui::portrait_fx::EFFECTS`) |
| `intensity` | float | 1.0 | 0.05-1.0 |
| `ms` | int | 0 | event rows: `flash` 60-1000 (required), `jolt` 0 (the preset's) or 60-600, others 100-3000 (required); state rows: 0 |
| `preset` | enum | "" | `jolt` only, required: `hit`, `crit` |
| `color` | string | "" | "" or `#rrggbb`; only on `flash`, `tint`, `glow`, `veins`, `eye_glow`, `ash`, `smoke` |
| `expression` | string | "" | `face` only, required: a name in the kit's `EXPRESSIONS` (5.4) |
| `priority` | int | 50 | 0-100; read only for `face` |
| `surfaces` | strings | [] | subset of `dialogue`, `roster`, `panel`, `timeline`; [] = all |
| `rulesets` | strings | [] | [] = every ruleset; else values of the ruleset option (`classic`, `timeline`) |
| `origin`, `tags` | as every catalog kind | system, [] | standard |

Cross-row check: two rows with the same trigger, the same match (`event`, `status`, `condition`, `stat`, `below`,
`drop_pct`) and the same `effect` are an error on both. The writer stores keys only when they differ from the
default.

### 5.2 Evaluation (`rpgkit_battle/src/portrait.rs`, pure)

- `Look { hp, max_hp, down, statuses, conditions, derived: [(id, with_statuses, without_statuses)], ruleset }`,
  built by the game from a battle unit (with its member's conditions) or a party member (no statuses; HP as the save
  carries it) or a world character (no HP: `hp` and `down` rows never match).
- `holds(rows, look, surface) -> Vec<CueRow>`: the state rows (`status`, `condition`, `hp`, `derived`, `down`) whose
  trigger matches, whose `surfaces` include `surface` and whose `rulesets` include `look.ruleset`.
- Matches: `status` when the status is in `statuses`; `condition` when in `conditions`; `hp` when `max_hp` > 0 and
  `hp x 100 < below x max_hp`; `down` when `down`; `derived` with `below` when `with_statuses < below`, with
  `drop_pct` when `with_statuses x 100 <= without_statuses x (100 - drop_pct)` and `without_statuses` > 0.
- `reactions(rows, events, surface, ruleset) -> Vec<CueRow>`: the event rows for each event in order.
- Same inputs give the same output; no RNG, no state.

### 5.3 Step events (`rpgkit_battle/src/portrait.rs`)

`step_events(hp_before, hp_after, lines, guarded) -> Vec<(unit, event)>` over one battle step: a unit whose HP fell
gets `crit` when the step's attack line on it reads " CRIT", else `guard_hit` when it was guarding (the step's
`hurt_guarded` case), else `hit` (one of the three per unit per step); a unit whose HP rose gets `heal`; a unit named
by a `[Break]` line gets `break`. Read-only over the step; the game fires them at FX1's impact (rule I), so a portrait
flinches when the arrow lands.

### 5.4 Expressions and the face chain (`rpgkit_content/src/art.rs`)

- `EXPRESSIONS`: `neutral`, `hurt`, `angry`, `afraid` (PQ2).
- Frames: `portraits/<folder>/faces/<id>/<expression>.png` (unique tiers only), each with an optional
  `<expression>.face.json`.
- `face_chain(tier, id, body, expression)`: for a unique tier and an expression E other than "" and `neutral`:
  `faces/<id>/E.png`, then `faces/<id>/neutral.png`, then `portrait_chain`; for "" or `neutral`:
  `faces/<id>/neutral.png`, then `portrait_chain`; a common tier: `portrait_chain` alone.
- `face_sidecars(found_rel) -> Vec<String>`: the found file's sidecar, then for a frame the base portrait's
  (`portraits/<folder>/<id>.face.json`), so a frame without its own anchors uses the base's.

### 5.5 Asset checks (`rpgkit_content/src/check/assets.rs`, warnings like A3-A5)

- A11: every `*.face.json` under a layer's `portraits/` parses and passes `face_problems` against its PNG; a sidecar
  without its PNG warns.
- A12: a file under `portraits/<folder>/faces/<id>/` must be named by `EXPRESSIONS` (`.png` or `.face.json`); `<id>`
  must have its unique portrait or a `neutral.png`; a frame without a sidecar whose base sidecar has eyes warns
  ("uses the base anchors").

## 6. Game

### 6.1 The binding (`emberwake_gui/src/portraits.rs`, PORTRAITFX1-G)

- `Faces`: one per screen: a map key to `PortraitFx`, the screen's `Budget`, its clock (or the battle's FX clock).
- `look_of_unit(session, i, conditions)`, `look_of_member(d, gs, member)`, `look_of_character(d, gs, character)`
  build the kit `Look`; the chain is `face_chain` over `portrait_tier` and the character id, else the unit id, with
  the engine's `expression(now)`.
- `Faces::draw(ui, cx, key, look, surface, rect)`: holds = the sidecar's `idle` plus the kit's `holds`; draws; on a
  false return draws today's initial letter.
- `Faces::react(key, events, surface)`: fires the kit's `reactions`; `skip_anim` (battle) fires none.

### 6.2 Surfaces (in the user's priority order)

| Order | Surface | Where | Portrait | Cues |
| --- | --- | --- | --- | --- |
| 1 | `dialogue` | The talk box (`region.rs` `draw_talk`, 64 px, PLAYUI1 3.1); vignette `line` cards (`vignette.rs`, 64 px) | as today's rects | talk box: base life, sidecar idle, state holds of the speaker's look; vignette: base life and sidecar idle only (staged scenes; the card's `expression` with B) |
| 2 | `roster` | The Character screen header (`party.rs`): 48 px, inside the frame, left of the member picker, overlapping no widget; the hub party list (`hub.rs`): 20 px left of each name, the name's fit width 22 px shorter | new | state holds (wounded, HP carried) |
| 3 | `panel` | The battle hero panel and the intel panel (`battle.rs`): 32 px at the panel's top-left (inset 4); the text lines beside it start 40 px in | new | holds and step events at the impact |
| 4 | `timeline` | The strip rows of the timeline ruleset: the row-height square the timeline widget reserves left of a track's label (`Track.icon`, 6.5) | new | holds and step events (minimal tier: flash, tints, fade) |

Every placement passes the TEXTFIT1 audit (no cut, edge or overlap) at every text size.

### 6.3 Game Gallery page "Portraits" (`gallery.rs`)

Every portrait PNG of the game and kit layers (expression frames grouped under their character) in a list; the
picked one at 96 logical px with the controls of 4.10 (`gallery.portraits.*`), its sidecar summary (eyes, overlays,
or "no sidecar"), the anchors check, one button per cue row of `portrait_cues.json` (event rows fire; state rows
toggle), and the expression picker when frames exist. It is the review page for the asset tasks.

### 6.4 Colours (game defaults, `portraits.rs` constants)

Rim key Tallow Amber `#d97706` (the house rim light of every ART2/ART3 portrait). Everything else comes from the cue
rows and sidecars, in art-bible hues: Blight Violet `#9333ea` (marrow glow, veins), Resonant Orchid `#c084fc`
(infused eyes), Arterial Red `#e11d48` (crit, beast eyes), Candle Gold `#fbbf24` (guard, heal), Salt Cyan `#06b6d4`
(cold), Cinder Ash `#6b7280` (ash, smoke), Bleached Linen `#d9cbb4` (the hit flash).

### 6.5 Timeline icon square (`aegis_ui::timeline`, PORTRAITFX1-TL)

`Track.icon: bool` (default false): the widget reserves a square of the row height left of the track's label and
pushes a widget `<id>.icon.<track key>` with that rect; the caller draws into it after the widget. Nothing else in
the widget changes.

## 7. Expression frames (B)

- Cast (13): the founders `kaelen`, `vesper`, `maren`, `hollis`, `dunstan`, `elyse` (heroes); the hub cast `tess`,
  `bram`, `aline` (npcs); the bosses `captain_vane`, `proctor_quell`, `guild_master_orlov`,
  `inquisitor_prelate_nemeth`. Grendel (no own portrait yet) and the Amalgam (not in the slice) wait.
- One image-API call per character (Gemini 3 Pro Image, as ART3), each run asked first; the 2x2 template of
  `games/emberwake/docs/specs/ART_PROMPT_TEMPLATES.md` with the four cells fixed (the prompt in
  [PORTRAITFX1_TASKS.md](PORTRAITFX1_TASKS.md) 2.4); the reference image is the character's own source
  (`art_src/<id>.png`), or Mara Voss for a founder without one (PQ1).
- `generate.py` writes the sheet to `art_src/<id>_faces_gen<time>_<n>.png` with its prompt record beside it, as
  every ART3 image; `sheets.py faces` cuts the four quadrants (top-left `neutral`, top-right `hurt`, bottom-left
  `angry`, bottom-right `afraid`), clears the model's mark on the whole sheet first, runs each through `pixelize`
  with one crop for all four, and writes the frames of 5.4; `face.py` writes each frame's sidecar.
- Where expressions come from: (1) the dialogue node key `expression` (the speaker's face for that node) and the
  vignette `line` card key `expression` (both "" or a name in `EXPRESSIONS`; the writer stores them only when not
  ""); (2) `face` cue rows (battle and the other surfaces). A frame that does not exist falls back along
  `face_chain`, so every surface works with the single portrait.
- Switching frames is instant; the effects of 4.2 run over whichever frame is drawn.

## 8. Determinism and parity

- Presentation only: `PortraitFx` lives in screens; no rule, plan, AI, session, save, sim or trace reads it. The kit
  evaluation and step events are read-only functions over the state; seeds come from ids; the battle's RNG state
  after a battle is the same with portraits on and off.
- Godot parity: none retires. Godot had no portrait effects, sidecars or cue rows; the parity suites do not run the
  GUI. `portrait_cues.json` is read beside `CATALOG_KINDS` (no Godot world has it, as `derived_stats.json`), so the
  loader, writer and checks parity see nothing new. The B keys (`expression` on dialogue nodes and line cards) are
  stored only when not "", so no Ashlands, Iron Reaches or Tidewrack file changes; if the node key needs a schema or
  help row, it joins the existing after-Godot lists as earlier new fields did, and no recorded expectation changes.
  The content fingerprint of a data folder changes once when `portrait_cues.json` lands (as with
  `derived_stats.json`).
- Playtest recordings replay their requests; portraits follow from the session and the clock, so a replay draws the
  same portraits for the same frames.

## 9. Out of scope

(C) layered puppets (separate head, eyes, mouth and body layers with lip-sync and head turns: an idea row,
PUPPET1); figure masks for silhouette flashes and figure-clipped veins (no mask data exists; `pixelize.py` could
write one later); a Forge or Gallery editor for anchors (the tool and the anchors check serve); portraits on the
party and hostile battle cards, the Results screen, the shop and the Bunks screen (Act 2, ROSTER1-RU); unit-scope
corruption meters (CAMPAIGN1-DOC: the cue trigger `meter` is reserved); new sounds; talking mouth flaps; expression
frames for common portraits; animated region-map tokens.

## 10. Open questions (recommendation first; the user is present)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| PQ1 | The six founders have no own portrait (CAST1_ASSETS: Ashlands faces stand in). How do they get expression frames? | (a) their 2x2 call makes their own face set, Mara Voss as the style reference; the `neutral` cell also becomes `portraits/heroes/<id>.png`; (b) a single-portrait run for each first, then the sheet with that portrait as the reference (12 calls); (c) skip the founders until their portraits exist | (a): 6 calls instead of 12, and the four frames match each other because they come from one image |
| PQ2 | The expression set | (a) `neutral`, `hurt`, `angry`, `afraid` for all 13; (b) per character (a boss gets `gloating` for `afraid`) | (a): one cue table for everyone; the kit list can grow later |
| PQ3 | When is Composure "low" (the cold tint)? | (a) a drop of 25 % or more from the unit's own value without status modifiers ("shaken"); (b) below 20 absolute | (a): (b) would tint every low-KNOW character all the time. Note: no slice status lowers Composure yet, so the tint shows in the Gallery until the Act 2 data adds modifiers |
| PQ4 | Does a sheet's `neutral` cell replace a character's existing single portrait (the ART3 picks)? | (a) yes, per character at the review of its sheet, so dialogue does not jump between two drawings; (b) never: the ART3 pick stays the neutral face | (a), decided per character when the user sees the contact sheet |
| PQ5 | Sinister overlays on the cast (the table in [PORTRAITFX1_TASKS.md](PORTRAITFX1_TASKS.md) 2.2) | (a) glow, veins and eye glow only on corrupted, Inoculated and eldritch portraits; ash or smoke on fire-and-smoke characters (the Ashlands goblins, Orlov's rendering vats, the Purifier's sulfur); heroes, the hub cast and plain humans clean; (b) also a faint ash on everyone | (a): the art bible's contrast rule (human matte versus eldritch glow) |

## 11. Build notes (PORTRAITFX1-E, 2026-10-25)

- Time steps: `tq(now) = floor(floor(3 now / 100) x 100 / 3)`, exactly 30 steps a second (`floor(now / 33) x 33` would give
  31 compositions in 1000 ms, against E12).
- Fade multiplies the alpha of the whole image, the border included (4.3 step 8); E13's "only the jolt changes a border
  pixel" holds with fade left out.
- Seeds the spec left open: ash start `h(650 + k)`, smoke x `h(820 + k)` and start `h(840 + k)`, vein segments
  `k = 400 + 1000 x root + 200 x branch + segment` (the trunk takes the first 60 % of the 1800 ms growth); the jolt seed
  is `seed ^ now`.
- A sidecar that parses is kept even with range problems; drawing clamps to the inner rect and the asset check (A11)
  reports the problems. A reduced-tier eye box blinks only when at least 2 px tall after scaling.

## 12. Build notes (PORTRAITFX1-XT, 2026-10-31)

- `generate.py run <model> <id>... --faces [--prompts F]`: the prompts file defaults to
  `games/emberwake/docs/specs/PORTRAITFX1_FACES.md` (the ART3 layout); each id's section (from its `##` heading that names
  the id to the next `##` heading) holds one `reference: art_src/<file>.png` line, outside or inside its text block, repo-relative;
  the line is never prompt text. An id without the line is skipped with a note. The record keeps generate.py's keys in
  their order and adds `"kind": "faces"`; `reference` is the image sent. `--dry-run` (any run) prints each id's
  reference, prompt size and output name without a key or an API call.
- `sheets.py faces`: tier `hero`, `npc`, `elite`, `boss` or the folder name; `--crop l,t,r,b` is in pixels inside a
  quadrant (default the whole quadrant); an odd last pixel row or column of the sheet is dropped so the quadrants are
  equal; the mark is cleared on the whole sheet (`clear_mark` then `clear_corner`) and each quadrant goes through
  `pixelize` with `mark=False`; the backdrop is `--faction`, else the sheet record's `faction`, else
  `art_src/<id>.json`'s, else hearth; `--root` (default `games/emberwake/assets/portraits`) is for tests; `--contact`
  shows the portrait as it was before the run (so a `--base` run shows old against new), then the four frames.

## 13. Build notes (PORTRAITFX1-D, 2026-11-01)

- Talk box (`region.rs` `draw_talk`): one `portraits::Faces` per region map (`begin` at the top of its draw, `tick` in its
  frame tick); the portrait is the speaker's `Who::of_character` (keyed by the character id; the initial letter from the
  shown `speaker_name`), its look `look_of_member` when a party member plays the speaker (matched by the character's
  `_qid`), else `look_of_character`. `look_of_member` now carries the member's ROSTER1 `conditions` (so the `wounded` row
  holds). The chain is `face_chain` with the engine's expression (with no frames it is today's `portrait_chain`). A
  speaker with no file draws the initial as before and the missing file still goes to the playtest trail. The map's
  `state` gains `portraits` while the talk box shows (the shop hides it), `[]` otherwise.
- Line cards (`aegis_ui::vignette`): `CardPortrait` keeps one `PortraitFx` for the speaker on view (a new key starts a
  new one; a card of another style, a done or skipped reel keeps none), holds only the sidecar's `idle` (no state rows:
  a staged scene), its own `Budget`, and a clock `VignetteScreen::advance` moves by the frame time it feeds the reel.
  `Card.portrait_key` seeds it (the game sets the character id; "" uses the drawn file's path);
  `VignetteScreen::portrait_rim` sets the rim key (the game passes Tallow Amber). The sidecar is the drawn file's own
  (`Art::face`): an expression frame without its own sidecar does not fall back to its base portrait's in the engine
  (the kit's `face_sidecars` rule); PORTRAITFX1-X, which brings the card's expression, resolves that. `vignette::draw`
  keeps its signature and draws the portrait still (a fresh `CardPortrait`); `draw_with` animates it. The screen's
  `state` gains `portraits`.
- At text size Normal on a 1x canvas the 64 px talk box portrait is the reduced tier (no breath: rim flicker and blinks
  only); from 2x up it is full (4.6).

## 14. Build notes (PORTRAITFX1-R, 2026-11-01)

- Character screen (`party.rs`): the shown member's portrait at (frame x + 10, 44), 48 px (`PORTRAIT`), on the talk
  box's dark panel: inside the frame below its title, left of the member picker and above every tab's content (no tab
  draws above y 100), so it overlaps no widget on any tab at any text size. One `portraits::Faces` per screen (`begin`
  at the top of the draw, `tick` in the screen's tick); Q / E draw the next member's portrait and `begin` drops the old
  one's `PortraitFx` at the next frame. At text size Normal on a 1x canvas it is the reduced tier (rim flicker and
  blinks, no breath); from 2x up it is full.
- Game menu (`hub.rs`): each party row draws a 20 px portrait (`ROW_PORTRAIT`) at its left and its three lines (name,
  class and level, HP with the wounded mark) start 22 px in (`ROW_INDENT`); the column grows by the indent, so the
  name fits 22 px shorter than the column and the class and HP lines keep their full width (the portrait is taller than
  one line, so only moving the name would put it over the class line). The minimal tier: a wounded member's
  desaturate shows, no motion.
- Both surfaces use `hub::member_portrait`: `Who::of_member` keyed `member:<id>` (two members playing one character,
  as test copies do, never share holds) and `look_of_member` (carried HP, the ROSTER1 conditions). Widgets
  `character.portrait` and `hub.portrait.<member id>` (kind `portrait`, no action, no hover); both screens' `state`
  gains `portraits`. A member without a file draws the initial letter (the Character screen on the panel, the menu on
  the box).

## 15. Build notes (PORTRAITFX1-X, 2026-11-01)

- PORTRAITFX1-S had already landed X1 (the schema rows, the check `check::dialogue::expression_error` on nodes and
  `line` cards, the writer leaving "" out, and the narrowed parity row of 2026-10-26); X adds what reads the keys.
- Talk box: `DialogueRunner::view` carries `expression` only when the node names one (so a Godot world's views are
  unchanged and the journey parity stays as it was). `talk.rs` passes it to `Faces::say(key, expression)` after `begin`:
  a held `face` cue keyed `line.face` at priority 100 (`vignette::CARD_FACE_PRIORITY`, the top of the rows' 0-100), so
  the line's face wins over the cue rows' faces while the node shows; "" leaves the face to the rows. The frame follows
  `face_chain` as before. The region map and the battle talk share `draw_talk`, so both get it; `battle.rs` is
  untouched. `talk_state` (the agent link) gains `expression` ("" none).
- Line cards: the kit's card resolution is `story::card_portrait(d, home, card)` (the speaker's `face_chain` with the
  card's `expression`); the game's `card_of` uses it for scenes and the Gallery's entry reels. The engine `Card` reads
  `expression` and holds it as a `face` cue keyed `card.face` (priority 100, so the agent state shows it); the chain the
  caller gives already starts at that frame. The sidecar fallback is a resolver: `CardPortrait::sidecars`, an
  optional `fn(&str) -> Vec<String>` set by `VignetteScreen::portrait_sidecars`; the game passes the kit's
  `face_sidecars`, so a frame without its own sidecar takes its base portrait's anchors and idle overlays. Without a
  resolver the drawn file's own sidecar, as in D.
- X4 is a binding test (`portraits.rs`): a `face` `hurt` cue fired for 900 ms draws the `hurt` frame from its start to
  its last millisecond and the neutral frame after; a missing frame falls back along the chain. With no frames shipped
  (XA makes them) every surface draws as before.

## 16. PORTRAITFX1-EX: expression effects at runtime (2026-11-01)

Decision "Expression effects at runtime" (user 2026-11-01): each expression can carry a runtime effect drawn over the
sidecar's face and eye boxes from a tunable data table. Display only: no rule, save, sim or trace reads it; goldens and
parity unchanged (one kind joins the schema compare's after-Godot list, as `portrait_cues` did).

- Sidecar (`aegis_core::assets`, `tools/portraits/face.py` mirrors it): the optional key `face` (`[x, y, w, h]` or [],
  after `mouth` in `FACE_KEYS`; inside the inner rect, at least `size / 8` a side). `Face::face_box()` is that box, else
  one from the eye boxes (`FACE_FROM_EYES`: the eyes' union, span = max(its width, round(size / 5)), x from its centre
  minus 0.9 span, y from its top minus 0.6 span, 1.8 span wide, 1.9 span tall, clamped to the inner rect); with neither,
  None (the tint is skipped, nothing crashes). `face.py set --face x,y,w,h` writes it (`--face ""` clears it); the
  writer leaves the key out when unset, so the 42 sidecars keep their bytes.
- Kit table `content.portrait_expressions` (file `data/catalog/portrait_expressions.json`, beside `CATALOG_KINDS` as
  `portrait_cues`; typed `rpgkit_data::PortraitExpression`, `Catalog::portrait_expressions`; checks
  `rpgkit_content::check::expression_fx`; the writer stores only fields that differ from the defaults, comparing a float
  default as normalize snaps it):

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | id (key field) | "" | an `EXPRESSIONS` name but `neutral` (the plain face takes no row); unique |
| `tint` | string | "" | "" (none) or `#rrggbb`: laid over the face box |
| `tint_strength` | float | 0.3 | 0.05-1.0 |
| `tint_ms` | int | 0 | 0 (holds, easing in over `EASE_MS`) or 60-3000 (full at the start, fading out: a flash); needs a tint |
| `mark` | enum | "" | "", `vein` (the anger mark), `sweat` (a drop): `PORTRAIT_MARKS`, the engine's `MARKS` |
| `mark_color` | string | "" | "" (the mark's default) or `#rrggbb`; needs a mark |
| `mark_box` | enum | `face` | `face` or `eyes` (the eye boxes' union) |
| `mark_x`, `mark_y` | float | 0.85, 0.15 | -1.0-2.0: the mark's centre in units of its box (outside allowed: a temple) |
| `mark_size` | float | 0.12 | 0.04-0.3: the mark's width as a share of the composed side, never under its bitmap's width |
| `mark_pulse_ms` | int | 0 | 0 (steady) or 300-4000; needs a mark |
| `shake_px`, `shake_ms` | int | 0, 0 | 0-4 px; 60-600 ms with a shake, else 0 |
| `origin` | as every catalog kind | system | standard |

A row with no tint, no mark and no shake warns ("draws nothing"). Shipped rows (seed applied, then deleted): `hurt`
Arterial Red at 0.8 over 240 ms with a 2 px, 220 ms shake (the red flash and the short shake); `angry` Arterial Red at
0.35 held, the vein mark in Arterial Red at (0.88, 0.12) of the face box pulsing every 900 ms; `afraid` Salt Cyan at 0.4
held (a cool pallor), the sweat drop in Salt Cyan at (0.12, 0.2), size 0.08.

- Engine (`aegis_ui::portrait_fx`, game-agnostic: the rows are data): `ExpressionFx` (the row), `Mark` (`Vein`, `Sweat`:
  9 x 9 and 5 x 7 bitmaps, `X` the colour, `H` its highlight, `O` its outline, scaled by nearest), `MarkBox`,
  `PortraitFx::set_expressions(&[ExpressionFx])`, `expression_since(now)` (the winning face and when its cue started),
  `expression_fx(now)`, `expression_shake(now)`, `offset(now)` (the jolt plus the shake, each axis within 4 px). The row
  of the winning face's expression draws from that cue's start; a held face whose expression changes starts again
  (`hold`), so a line that turns hurt flashes then.
  - 3b (after the colour step): the tint `tinted(c, colour, strength x w)` over the face box, `w` a soft ellipse
    (`face_weight`: clamp(2 (1 - d^2), 0, 1), full over its inner half, 0 at the box's corners).
  - 5b (after the eye glow): the mark centred on its anchor (`ExpressionFx::anchor`, moved with the breath), its width
    `mark_width`; a pulse grows it by up to 20 % and dims it to 70 % at the low (`mark_pulse`).
  - The shake is a `Shake` (axis x, ease-out, a flip every 30 ms) from the expression's start, added to the jolt.
- Tiers and reduce motion: the tint at every tier; the mark from the reduced tier up (none at minimal); the shake halved
  at reduced (at least 1 px), none at minimal; reduce motion: no shake, no pulse (the mark steady at full size and
  alpha); the tint's ease and fade stay (information stays). A pulsing mark counts as time-varying for the `Budget`.
- Cache and agent link: the composed key gains the expression, its tint in 1/32 and the mark's width and alpha;
  `state(now)` gains `expression_fx` (the parts drawn: `tint`, `mark`) and `shake` ([dx, dy]).
- Game: `portraits::expression_fx_of(rows)`; `Faces` draws `Catalog::portrait_expressions` on every surface
  (`Faces::expressions`); the vignette line cards through `VignetteScreen::portrait_expressions` (`story::vignette_screen`).
  Gallery Portraits page: a box per row (`gallery.portraits.fx.<id>`, hover: the row's effect) holds its expression as
  the face (priority 1, over the frame picker) so each effect shows on any portrait; a second click releases it.
- Tests: `aegis_core` `the_face_box_is_its_own_or_the_eyes`; `tools/portraits/test_face.py`
  `test_ex_set_writes_the_face_box_only_when_given`; `aegis_ui/tests/portrait_fx.rs` EX1 (the tint in the face box,
  timed, derived, absent), EX2 (the mark at its anchor, cell by cell; the eye box; minimal; pulse and reduce motion), EX3
  (the shake, tiers, reduce motion, jolt sum), EX4 (a changed face starts again); `rpgkit_content/tests/portrait_expressions.rs`
  (5: missing file, schema options and defaults, every error text, loader, writer); `portraits.rs`
  `ex_the_expression_rows_draw_through_the_binding` and the mark-name check; the Gallery G6 test gains the boxes.
- Open (recommendation taken): the face box comes from the eye boxes until the asset lane writes `face` boxes
  (`face.py set --face`); the vein and sweat marks are code-drawn bitmaps; the `afraid` tint is Salt Cyan (cool) rather
  than a whitening pallor.

## 17. Build notes (PORTRAITFX1-P, 2026-11-01)

- Panels (`battle.rs`): the hero panel and the intel panel each draw their unit's portrait at (panel x + 4, panel y +
  4), 32 px (`PANEL_PORTRAIT`, `PANEL_INSET`), on the talk box's dark panel; widgets `hero.portrait` and `intel.portrait`
  (kind `portrait`, no action). The hero panel's name, its corner (the stance or the intent, now on the line under the
  name so both fit the 72 px beside the portrait) and the Self button start 40 px in (`PANEL_TEXT_X`); a following line
  whose first row fits beside the portrait starts 40 px in and wraps to the room left, the first that does not starts
  under the portrait at the full width. The intel panel does the same through `intel_parts_at` (the parts that start
  beside the portrait wrap 36 px narrower and draw 40 px in). At text size Normal on a 1x canvas the 32 px portrait is
  the minimal tier (flash, desaturate, tint, fade, face); reduced at 2x, full at 3x (4.6).
- Whose portrait (`BattleScreen::unit_portrait`): a world fight's party unit is its member (keyed `member:<id>` as
  PORTRAITFX1-R, `Who::of_member`, its ROSTER1 conditions in the look); a unit whose hero key names a character of the
  battle's world draws as it; any other as itself (`Who::of_unit`, keyed `<unit id>#<i>`). The look is
  `look_of_unit` with the board's HP and life (`Tokens::shown`, rule I), so a wound's desaturate also waits for its
  impact.
- Clock: a second `Faces` (`panel_faces`; the talk box keeps its own wall-time one) whose clock is set to FX1's clock
  at the start of every draw, so a hit-stop holds the portraits and the animation speed scales them (4.5).
- Step events: `after_change` computes `portrait::session_step_events` over the HP before the step and the step's new
  log lines; they ride in `fx::Impact.portrait` and `BattleScreen::fire` sends them to `react_portraits` at the
  impact's FX ms (rule I); a step with no delivery reacts at once. Under skip animations `Faces::react` fires none.
- State: the screen's `state` gains `portraits` (the panel portraits kept).
- Tests (`battle.rs` `portrait_panel_tests`, fixture cue rows): P1 placement, lines beside 40 px in, no spill or edge
  of either panel's texts at the four text sizes, classic and timeline; P2 no reaction at the pop, each event's flash
  from the impact's ms in its row's colour with a jolt for a blow, crit jolting harder than hit, heal without a jolt;
  P3 stealth alpha 0.58, below 25 % desaturate 0.5, down still and desaturate 1.0; P4 hit-stop and animation speed;
  P5 the session state, RNG state, log and request trace equal with portraits drawn and not drawn, and two drawn runs
  with the same portrait pixels.
