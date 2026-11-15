# PORTRAITFX1 Resolved Questions & Historical Build Notes (§§10–18)

Archived 2026-11-15 (CTX-CLEANUP1-DESIGN), resolved `PQ1`–`PQ5` and `2026-10-25..11-01` build notes moved verbatim from [PORTRAITFX1.md](../../specs/PORTRAITFX1.md).

## 10. Open questions (recommendation first; the user is present)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| PQ1 | The six founders have no own portrait (CAST1_ASSETS: Ashlands faces stand in). How do they get expression frames? | (a) their 2x2 call makes their own face set, Mara Voss as the style reference; the `neutral` cell also becomes `portraits/heroes/<id>.png`; (b) a single-portrait run for each first, then the sheet with that portrait as the reference (12 calls); (c) skip the founders until their portraits exist | (a): 6 calls instead of 12, and the four frames match each other because they come from one image |
| PQ2 | The expression set | (a) `neutral`, `hurt`, `angry`, `afraid` for all 13; (b) per character (a boss gets `gloating` for `afraid`) | (a): one cue table for everyone; the kit list can grow later |
| PQ3 | When is Composure "low" (the cold tint)? | (a) a drop of 25 % or more from the unit's own value without status modifiers ("shaken"); (b) below 20 absolute | (a): (b) would tint every low-KNOW character all the time. Note: no slice status lowers Composure yet, so the tint shows in the Gallery until the Act 2 data adds modifiers |
| PQ4 | Does a sheet's `neutral` cell replace a character's existing single portrait (the ART3 picks)? | (a) yes, per character at the review of its sheet, so dialogue does not jump between two drawings; (b) never: the ART3 pick stays the neutral face | (a), decided per character when the user sees the contact sheet |
| PQ5 | Sinister overlays on the cast (the table in [PORTRAITFX1_TASKS.md](../archive/specs/PORTRAITFX1_TASKS.md) 2.2) | (a) glow, veins and eye glow only on corrupted, Inoculated and eldritch portraits; ash or smoke on fire-and-smoke characters (the Ashlands goblins, Orlov's rendering vats, the Purifier's sulfur); heroes, the hub cast and plain humans clean; (b) also a faint ash on everyone | (a): the art bible's contrast rule (human matte versus eldritch glow) |

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
  `docs/archive/specs/emberwake/PORTRAITFX1_FACES.md` (the ART3 layout); each id's section (from its `##` heading that names
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

- Panels (`battle/draw.rs`): the hero panel and the intel panel each draw their unit's portrait at (panel x + 4, panel y +
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
- Tests (`battle/tests/portrait_panel_tests.rs`, fixture cue rows): P1 placement, lines beside 40 px in, no spill or edge
  of either panel's texts at the four text sizes, classic and timeline; P2 no reaction at the pop, each event's flash
  from the impact's ms in its row's colour with a jolt for a blow, crit jolting harder than hit, heal without a jolt;
  P3 stealth alpha 0.58, below 25 % desaturate 0.5, down still and desaturate 1.0; P4 hit-stop and animation speed;
  P5 the session state, RNG state, log and request trace equal with portraits drawn and not drawn, and two drawn runs
  with the same portrait pixels.

## 18. Build notes (PORTRAITFX1-TL, 2026-11-01)

- Which rows: the battle's top plan strip is the screen's own one-hero block row, not the timeline widget, so it has no
  icon square; surface 4 is the full timeline view (key T, INPUT1-T), the only battle use of the widget: every unit's
  track (Party and Enemies) leads with its portrait. The Gallery's Battle timeline (same builder) keeps plain tracks.
- Engine (`aegis_ui::timeline`, 6.5): `Track.icon` reserves `icon_rect(row)`, a `ROW_H` (15 px) square 1 px in from the
  row's left edge, and pushes `<id>.icon.<track>` (kind `icon`, no click); the label starts `ICON_W` (16 px) further in
  and the label column is that much wider when an icon track's label is the longest. The caller draws through
  `Timeline::draw_icons(.., icon)`: `icon(ui, track, square)` is called right after the square's widget inside the
  rows' scroll clip (a half-scrolled square is cut, not squeezed; a widget rect alone would be clipped), `draw` being
  `draw_icons` with none. The Kit widgets Timeline page has an Icons check (`kit.timeline.icons`): every track's square
  as a chip in its colour with its label's initial.
- Tier (4.6): a 15 px square is reduced from a 4x window (60 device px), so `PortraitOpts.tier_cap` (engine,
  `PortraitOpts::tier(side)`: the side's tier, no richer than the cap) and `Faces::capped(Tier::Minimal)` keep the row
  portraits minimal at any window size (flash, desaturate, tint, fade, face, eye-glow core; no jolt, no breath).
- Game (`battle/draw.rs`, `battle/playback.rs`): a third `Faces`, `timeline_faces` (capped minimal), set to FX1's clock at every draw like
  `panel_faces`; `react_portraits` fires each step's events on it too, on surface `timeline` (cue rows with empty
  surfaces apply to both); the look and key are the panels' `unit_portrait` (board HP and life, rule I); the square is
  drawn on the panels' dark backing with the portrait 1 px inside. State: `timeline_portraits`.
- Tests: TL1 (`aegis_ui/tests/kit_widgets.rs`): a track with and without an icon (the square, its widget, the label
  `ICON_W` in, every other widget and the plain row's pixels as before, `draw_icons` on plain tracks equal to `draw`),
  a long icon label widening the column by `ICON_W`, the Kit page's Icons check (a square per track, TEXTFIT clean);
  `aegis_ui/tests/portrait_fx.rs` the tier cap. TL2 (`battle/tests/portrait_timeline_tests.rs`, fixture cue rows, scratch
  portraits): every track's square and minimal tier at the four text sizes with no timeline text spilled or cut
  without its hover text; on a 4x window the row stays minimal (flash, no jolt, no breath) while the panel is full and
  jolts; a struck unit's row flashes from the impact's ms, nothing at the pop; a stealthed row draws at alpha 0.58.
  `the_battle_view_and_the_gallery_build_the_same_timeline` now compares with the battle's icon flags cleared.
- Answers (user 2026-11-01): enemy rows show state cues only when the board shows them: an enemy's row is plain (no
  fade, no wound tint, no other hold; a fall still shows) while the view masks its track (an unrevealed plan, TV4) or
  it is stealthed (the board's stealth veil, GD5) (`timeline_row_plain`, `plain_look`); its event cues fire only for
  a blow (`hit`, `crit`, `guard_hit`) the board shows on it (the step's floats and hit flashes,
  `react_portraits_shown`); party rows unchanged. The rows react only while the view is open. The Gallery's Battle
  timeline keeps plain tracks. Test `tl2_a_stealthed_enemys_row_is_plain_while_a_stealthed_members_fades` (also: no
  flash for an unshown blow or an enemy's heal, a shown blow flashes, nothing on the rows while the view is closed).
