# INPUT1: one-flow battle input (target, skill, ghost fill) and VISCLAR1 slice A (a calmer board)

Status: proposed (spec 2026-10-31, task INPUT1-S, from the decision "Combat input feedback (user 2026-10-31, all as
recommended)" in `games/emberwake/docs/decisions/combat.md` and the user's addition the same day on target arrows,
section 10.3)
Depends on: RULESET1-U, RULESET1-PL (done); INPUT1-U also waits on CAMPAIGN1-U and ROSTER1-L (shared files)
Lane: logic (INPUT1-U, INPUT1-T, INPUT1-L, VISCLAR1-A); no asset part (no shipped data needs a follow-up list)
Touches (by task, see [INPUT1_TASKS.md](INPUT1_TASKS.md) 4): `engine/crates/aegis_ui/src/timeline.rs`;
under `kits/rpg/crates/`: `rpgkit_battle/src/{round_intent_plan,round_intent,plan}.rs`, `rpgkit_host/src/lib.rs`,
`rpgkit_data/src/lib.rs` (`Deck`), `rpgkit_content/src/check/unit.rs` (deck), `rpgkit_rules/src/unit.rs` (deck terms),
`rpgkit_world/src/state.rs` (member deck, save); `kits/rpg/schema.json` (unit `deck` help); under `games/emberwake/crates/`:
`emberwake_gui/src/{battle,menus,screens,party,dojo,gallery,intent_text}.rs` and a new `timeline_view.rs`
Schema contract, kit and host API, tests, parity, slices and task rows: [INPUT1_TASKS.md](INPUT1_TASKS.md).
Rule ids: IF (flow), HK (hotkeys), GH (ghost fill), ET (empty tiles), SL (selection), IS (inspect), AP (readout),
TV (timeline view), FL (follow-up lists), VC (clarity); RULESET1_INTENT's PL and MM rules are mapped in section 11.

History (`docs/HISTORY.md`): row 2026-10-25 "RULESET1 first draft: authored `follow_ups` and chain picks" replaced
skill-authored chains with per-intent order lists (user: chains for any skill type, fewer clicks, one intent and one
target a round). FL1 does not bring authored chains back: the lists are the player's, kept in the member's `deck`
beside `order`, hold only the opener's intent and act on the one round target, and an unset list falls back to the
order. The reason of that row still holds for skill rows; the new reason for per-opener lists is the user's
decision of 2026-10-31. No other HISTORY row touches the input flow, the inspect click, the AP readout or the
timeline view. Section 11 lists the RULESET1 rules this spec retires (one HISTORY row each, INPUT1_TASKS 6).

## 1. Goal

Under the rule `round_intent` the battle screen takes one input flow per hero: click a target, pick a skill from a
menu of the skills usable on it; the skill sets the round intent and Maneuver kind; the planner fills the rest of the
round as ghost blocks that each further pick replaces; Confirm plays picks and ghosts alike. The intent buttons, the
Manual toggle and Fill go; keys 1-9 become skill hotkeys. Right click inspects any unit, Esc or a click on the
selected hero selects nothing, an AP readout ends the strip, key T opens a full timeline of every unit (as much as
the party knows), and per-opener follow-up lists in the loadout steer the fill. VISCLAR1 slice A calms the board:
the enemy first-move chip goes behind a debug setting, bars show for the units in focus, each unit has at most one
quiet target arrow, and a debug-build toggle hides party foresight. Classic and a timeline battle without
`round_intent` keep today's input (decision "RULESET1-U slice 1": the classic screen stays as it is).

## 2. The flow (INPUT1-U)

- IF1. Targets: an enemy, an ally (the selected hero itself through the Self button, SL4), a tile. A left click on a
  unit or tile while a hero is selected and planning sets the round target at once (the gold ring, TG1; the ghost
  rebuilds, GH2) unless a skill or item pick already holds the target (IF6), then opens the skill menu at the click.
  The menu is the existing battle popup (`popup`, rows with label, detail, valid), titled "`<hero>` -> `<target>`".
- IF2. The menu lists, in hotkey order (HK1), every skill of the hero that could act on a target of that kind under
  the intent table (RULESET1_INTENT 2: an enemy takes Assault and Maneuver skills, an ally Support skills, self Guard
  and Support self skills, a tile Guard tile moves (ET), area and hazard openers, Utility), then the pouch items of
  that aim ("Use `<item>` (`<n>` left)", intent by aim, IN5), then "Rest" (one block, neutral, RS1). A skill that can
  never act on that kind of target is left out; one that could but cannot now is listed disabled with its reason
  (IF6, the first failing check of FU2 c-d, MV5, PL2): "Only Assault skills this round (Heavy Cleave set it)", "One
  target per round: Wolf", "AP 1 < 2", "Blocks 1 < 2", "Out of reach: 3 > 2", "Out of range: 6 > 4". With no
  usable row the title reads "No skill reaches `<target>` this round" (PL7's words).
- IF3. Row text: `<hotkey> <skill name>` (the digit only for HK1's first nine), detail
  `<blocks> blocks, <ap> AP · <Intent>` (Maneuver rows add the kind: "Maneuver (Engage)"). A Maneuver ATTACK
  skill gets one row per kind the kit offers on that target, in MV11's order (Engage, Disengage); a Maneuver skill or foe-aimed item on a tile is
  Utility. A row whose pick would change the intent and drop intent-bound buffs keeps today's warning: a red corner
  and the tip line "Changing loses Riposte." (kit `round_intent::would_drop`).
- IF4. Picking a row (click, or its digit while the menu is open) appends the pick (GH3). The round intent and the
  Maneuver kind follow from the first skill or item pick (IN4, IN5, the row's kind); setting them is today's
  `Session::set_round_intent` / `round_intent_plan::set_kind` (an intent change drops intent-bound buffs then, IN6;
  undo does not bring them back, as with today's buttons). Neutral picks (Rest, Move here, Step here after) set no
  intent.
- IF5. Hovering a row previews it: the strip shows the plan as if the row were picked (picks solid, the new ghost
  dashed) and the AP readout shows its numbers (AP3). Leaving the row restores the strip. A preview changes nothing.
- IF6. A pick is refused, with its reason in the row and the message bar, when (a) it is a skill or item of another
  intent than the round's current pick-set intent (MM1's "Only `<Intent>` skills this round"; to change the intent:
  Clear or undo back to before the first intent pick); (b) another unit or tile is the round target and a skill or
  item pick holds it ("One target per round: Wolf"); (c) its blocks do not fit before a Step-here-after pick (GH4) or
  the round's end; (d) it cannot be paid without AP debt after the picks before it (PL7: never planned debt); (e) it
  cannot act on the target from the tile the hero stands on when it starts (reach band, support reach, area range,
  MV5 range), after the opener's implied before-move (PL5).
- IF7. Undo (key Z, a right click off every unit and widget, as today's `back`) removes the last pick; Clear
  (`strip.clear`) removes every pick; the round target and the current intent stay; the ghost rebuilds (GH2).
- IF8. Gone from the battle screen: the intent buttons (`hero.intent.<id>`), the Maneuver kind and opener menus
  (`hero.kind`, `hero.opener`), Manual (`hero.manual`) and Fill (`hero.fill`). The hero panel shows instead: "Intent:
  `<Intent>` (`<kind>`) from `<first pick>`" (or "(carried over)" with no intent pick), "Target: `<name>`", the picks
  in order, the Self button (SL4) and the Rest line as today. The Dojo's Planner window keeps its intent, kind and
  opener controls (UI4: they judge the planner) and is out of scope.
- IF9. The party tactic (`strip.party`) drafts every member with no pick as today and skips members with a pick (as
  it skipped Manual members); the hero tactic (`strip.hero`) clears the selected hero's picks and drafts it. A draft
  is ghost (GH1). Autoplay ignores Auto-fill (it drafts whole rounds).

Acceptance (rule-derived; INPUT1_TASKS 3 lists the tests): a click on an enemy sets the target and opens a menu with
no Support row; a Support skill never appears on an enemy's menu; after a Heavy Cleave pick a Guard skill row is
disabled with "Only Assault skills this round"; after a pick on Wolf every row of a Footman's menu is disabled with
"One target per round: Wolf"; the first Maneuver pick's row kind becomes `round_plan`'s `maneuver`; hovering a row
then leaving it leaves `round_plan` equal to before.

## 3. Hotkeys (INPUT1-U)

- HK1. Hotkey order, per hero (kit `round_intent_plan::hotkey_skills`): the deck's actives in slot order, then the
  ruleset's `basics` that are not neutral, in `basics` order; without the deck rule, the loadout's slot order then
  the basics. Keys 1-9 are its first nine; later skills and items have no key. The order never changes during a
  battle (a fixed slot under the thumb), whatever the target.
- HK2. With the menu open, a digit picks the row with that digit (a disabled row says why in the message bar). With
  the menu closed, a digit picks that skill on the current round target (the proposed one counts, TG6) if usable, else
  the reason; with no round target: "Pick a target first (click a unit or a tile)".
- HK3. A self-only skill (DEFEND, `target_mode` SELF) picked by its key as the round's first skill pick targets self
  (no pick holds the target yet, TG1); later it needs a self or tile target already set, else the reason.
- HK4. The keys 1-9 no longer pick intents. Classic: nothing (as today). The Controls list (Settings) says "Skills of
  the selected hero (timeline): 1-9".

## 4. Ghost fill and the Auto-fill setting (INPUT1-U)

- GH1. A member's round plan is its picks in order, then ghost blocks: what the planner placed, not the player.
  Confirm plays both; a ghost block is a real plan step (reason tags, AP, forecasts as any step).
- GH2. With no pick, the ghost is the whole auto plan (PL5) for the current intent on the round target (TG6's
  proposal) with PL3's opener; Auto-fill off: Rest on every block. A hero with no intent yet (battle start without a
  `default_intent`) gets the hero tactic's pick (AI5, `Session::auto_draft` for that member; free: no buff is bound
  yet, IN2) as ghost (open question Q1).
- GH3. A pick goes at the first block after the last pick (it replaces the ghost there); its implied moves belong to it
  (the opener's before-move, PL5; a Maneuver skill's carried move, MV4) and draw solid; then the ghost is rebuilt
  after it. Auto-fill on: FU2/FU3's fill (from the opener's follow-up list, FL2), AS3's reserved kite-after unless a
  Step-here-after pick exists, then Rest. Auto-fill off: Rest only.
- GH4. A Step-here-after pick (ET4) reserves the round's last block like AS3; later picks and the ghost fill go
  before it; a pick that would need that block is refused (IF6 c: "Blocks 0 < 1 (the step after takes B6)").
- GH5. Each pick or unpick rebuilds the member's plan from its picks (atomic: a refused pick leaves the plan as it
  was), then every later member's (PL8). A later member's pick that no longer fits (an earlier member claimed its
  approach tile, PLAN1) is dropped with every pick after it, with the message "`<hero>`: `<skill>` dropped: `<reason>`".
- GH6. Picks reset at the round start with the target, kind and opener (`round_intent::round_start`); the intent
  carries over (IN2).
- GH7. Setting "Auto-fill the rest" (Settings, Gameplay tab, checkbox `settings.auto_fill`, key `auto_fill`, default
  true; hover: "After your picks the planner fills the round's free blocks with follow-ups (ghost blocks you can
  replace). Off: the free blocks Rest."). One setting for the whole party; changing it during planning rebuilds every
  member's ghost at once. The battle passes it to the kit (host `autofill {on}`, recorded, so a replay keeps it).
- GH8. Look: a ghost span on the strip and in the timeline view is drawn with the engine's ghost style (fill at 45%
  alpha, dashed outline, label in DIM; INPUT1_TASKS 1.4); its tip starts "Auto-filled" and then the reason tag (PL6);
  a ghost Rest block stays dim as today.

Worked examples (move 2 tiles a block; `ap_regen` 4; the readout is AP1's):

| Hero, AP, max, situation | Input | Plan (picks solid, ghost in brackets) | Readout |
| --- | --- | --- | --- |
| tl_vanguard, 10, 10, footman 3 tiles away | click footman, Heavy Cleave | b1 approach (2), b2-b5 Heavy Cleave (4), [b6 Shoulder Check (1)] | `AP 10 - 7 = 3 · +4 next -> 7` |
| same, Auto-fill off | the same | b1 approach, b2-b5 Heavy Cleave, [b6 Rest] | `AP 10 - 6 = 4 · +4 next -> 8` |
| tl_archer, 10, 10, footman adjacent | click footman, Aimed Shot, Snap Shot | b1 kite (2), b2-b3 Aimed Shot (2), b4 Snap Shot (2), [b5-b6 Aimed Shot (2)] | `AP 10 - 8 = 2 · +4 next -> 6` |
| tl_vanguard, 3, 10, footman adjacent | click footman, Heavy Cleave | refused: "AP 3 < 4" (row disabled, readout red on hover); the ghost stays [b1-b3 Shoulder Check (1 each), b4-b6 Rest] | `AP 3 - 3 = 0 · +5 next -> 5` (Rest 3 blocks: +1) |

## 5. Empty tiles and moves (INPUT1-U)

- ET1. A left click on an empty tile (no unit) opens the tile menu: "Move here"; "Move here, then `<skill>`" for each
  Guard skill and each Support skill on self usable from that tile (a move then the skill); the skills that take the
  tile itself as target from where the hero stands (area and hazard openers, Utility, with PL5's implied approach);
  "Step here after" (ET4); "Rest".
- ET2. "Move here" is a plain move (IN5, neutral, keeps the target): offered only as the round's first pick and while
  no move is planned; the kit's plain-move rule decides it (first, once a round, at most 2 move blocks, 1 AP a tile,
  PLAN1 claims). With "Move here, then a Guard skill" the round target is the tile (GD1: move, then hold); with a
  Support skill it is self.
- ET3. Budget refusals (row disabled, reason in the row; the first that fails): "No path", "Too far: 5 tiles > 4 (two
  move blocks)", "Claimed by `<ally>`'s move", "AP 2 < 3" (the move's AP plus the skill's), "Blocks 2 < 3" (the move's
  blocks plus the skill's), "Already moving this round" (a pick's approach or an earlier Move here), and the skill's
  own IF6 reasons as measured from the tile.
- ET4. "Step here after" (skill then move, the after-move): one standard move block at the round's last block
  (`PlanStep::after_move`, AS3's shape), 1 AP a tile, at most one block's tiles; offered after a skill or item pick,
  for Assault, Guard and Support (open question Q4); refused "Too far for one move block: 3 > 2", "AP", "Blocks 0 <
  1 (the picks fill the round)", "Disengage retreats with its skill" (MV8), "One step after a round". It replaces a
  ghost AS3 kite. Its tile is the clicked one (PL4 does not re-rank it).

Acceptance: "Move here" on a tile 5 tiles away is disabled "Too far"; after "Move here" an Assault pick is measured
from the new tile (no approach entry follows the move); "Move here, then Active Guard" plans a move entry then the
Guard skill with the tile as `round_plan` target; "Step here after" puts an after-move entry at the last block and a
later 2-block pick that would cover it is refused.

## 6. Selection and inspect (INPUT1-U)

- SL1. With a hero selected (planning), a left click on an enemy sets the target (IF1).
- SL2. A right click on any unit (token or card; enemy, ally or the selected hero) inspects it (IS1); it never
  targets, selects or undoes. A right click elsewhere keeps today's `back` (close the popup, the log, else undo the
  last pick).
- SL3. Esc: closes the menu or popup, then the log, then clears the inspected unit, then selects nothing; with
  nothing selected and nothing inspected it does what it does today (the Arena leaves; a world battle nothing).
- SL4. A left click on the selected hero (token or its party card) selects nothing. The hero panel's Self button
  (`hero.self`) and key S set self as the target and open the self menu.
- SL5. With no hero selected, a left click on an enemy inspects it; on an ally selects it (as today).
- SL6. With a hero selected, a left click on another ally targets it (its Support menu, IF2) with a last row
  "Select `<ally>`"; with no usable row the click selects the ally at once (today's auto-mode answer). A party card
  click always selects (cards are not targets).
- SL7. Outside planning (playback, battle over) a left click on an enemy inspects it, on an ally selects it.
- IS1. Inspect pins the threat intel panel (`intel`) on the unit, headed "Inspecting `<name>` (Esc clears)", until
  another inspect, Esc or the round's end; with none pinned the panel follows the hovered unit as today.
- IS2. An inspected enemy shows only what the party knows: name, HP, poise (rule `poise`), statuses, reach ("Reach
  1-2"), facing and "Hidden"; the round intent and target (rule `plan_badges`); the tell's first action while tells
  show; the impact badge row (V1); its revealed plan only when revealed (foresight, enrage, `plan_reveal`; VC4 hides
  foresight reveals in debug); the forecasts of the selected hero's planned hits on it. Never: an unrevealed plan's
  skills, blocks or walks, its deck or order, its AI row or profile, a hidden hazard of its side.
- IS3. An inspected party member shows its intent, target, picks and plan, its AP readout (AP1) and statuses; it does
  not change the selected hero.

Acceptance: right click on the selected hero leaves `selected` and `round_plan` unchanged and sets `inspect`; Esc
with a menu open closes only the menu; with no hero selected a left click on an enemy sets `inspect`, not a target;
an inspected unrevealed enemy's panel text contains none of its plan's skill names (other than its tell's first
action while tells show).

## 7. The AP readout (INPUT1-U)

- AP1. At the strip's end, after the last block and before the tactic menus (widget `strip.ap`), for the selected
  hero while planning: `AP <now> - <spent> = <left> · +<gain> next -> <next>` with `now` the hero's AP, `spent` every
  planned command's AP (picks, their moves and the ghost), `left = now - spent`, `next` the AP the round's end would
  give with no hit taken (kit `round_intent_plan::ap_forecast`: `min(max_ap, left + ap_regen + ap_regen_bonus)` then
  RS2's Rest gain of the planned Rest blocks, capped at `max_ap`, as `status::roll_over` and
  `round_intent::round_start` apply them) and `gain = next - left`. Its tip splits `spent` into picked and
  auto-filled and names the Rest gain.
- AP2. It is rebuilt on every pick, unpick, Clear, target change, Auto-fill change and replan; while a menu row is
  hovered it shows that row's preview (IF5).
- AP3. It turns red (FOE) when the numbers shown cannot be paid: the hovered row's `left` would be below 0 (the row is
  disabled, IF6 d), or `now` is already below 0 (AP debt from earlier). Otherwise TEXT.
- AP4. Text fit (TEXTFIT1): when the full text does not fit the space the blocks leave at a text size, it shortens to
  `AP <left> -> <next>`, the full text on hover. The blocks lane gives the readout the width of its full text at
  Normal size at most.
- AP5. Only under `round_intent` and for party-side units (enemies pay no AP, AI2). The `·` is U+00B7; if the game font
  lacks it the builder reports it and uses `;` (open question Q7).

Acceptance: the four worked examples of section 4 read exactly so; hovering a row the hero cannot pay turns `strip.ap`
red and leaves it TEXT after the hover; on a dummy fight (no AP effects) `next` equals the hero's AP at the next
planning.

## 8. The full timeline view (INPUT1-T)

- TV1. Key T (planning and playback, rule `round_intent` or any timeline battle) toggles an overlay over the
  battlefield panel: rect (126, 28, 388, 292) on the 640 x 360 canvas, the grid panel's own rect; T or Esc closes it
  (Esc closes it first, before SL3's steps). It pauses nothing; while open the board under it takes no click.
- TV2. It draws the engine timeline widget (`aegis_ui::timeline::Timeline`): the ruler of the round's blocks (the
  playhead during playback), section "Party" (allies in roster order, guests too) above section "Enemies" (in the
  hostile list's order); one track per unit present or acting this round; a downed unit's track is hatched.
- TV3. A party track holds the member's plan while planning (picks solid, ghost in the ghost style, GH8; parts and
  BREAK marks as the strip), its committed actions during playback; the label leads with the intent glyph's letter
  ("A Vanguard"); the selected hero's track is the selected track; its label tip carries the AP readout (AP1).
- TV4. An enemy track shows only what the party knows: while planning an unrevealed enemy is masked with the note
  "`<Intent>`: `<target>`" (rule `plan_badges`; else "Plans in secret"), followed by its tell ("; Next: Cut B1") while
  tells show, and its impact badges as marks (red at a hit's impact block, gold for other actions, V1); a revealed
  enemy (foresight, enrage, `plan_reveal`) is unmasked with its plan (recovery hatched) and the tip "Plan revealed
  (`<reader>`)". During playback an unrevealed enemy's track holds its actions that have started (block <= the
  playhead) and nothing later; a revealed one holds all. VC4 masks foresight-revealed tracks in debug.
- TV5. A click on a party track selects that hero (the view stays open); hover cards are the segments' own (reason
  tags, forecasts, as the strip's tips). No block snapshot in the battle (the Gallery keeps it, TV7).
- TV6. Layout at 640 x 360: ruler 13 px, a section header 11 px, a track 15 px (the widget's `RULER_H`, `HEADER_H`,
  `ROW_H`); 4 px padding; the label column grows from 74 px to the longest label, at most a quarter (97 px); six
  blocks share the rest (about 47 px a block); 16 tracks fit before the widget's own scroll. At the four text sizes
  no label or note spills (`text_audit`): labels and notes shorten with the full text on hover (TEXTFIT1).
- TV7. UI Gallery (memory "Standalone UI panes"): a Kit group entry `timeline_pane` "Timeline pane" draws the engine
  widget alone on sample data that needs no battle (two sections, a masked track with its note and marks, parts,
  hatching, a link, a ghost segment, the playhead, a selected block and track, enough tracks to scroll), using no
  game type. The Battle panels entry `timeline` ("Battle timeline") draws the same view as TV2-TV4 through the shared
  builder (game `timeline_view.rs`), keeping its own tools (< > round archive, Reveal, the block snapshot strip).

Acceptance: with an unrevealed enemy planning, its track is masked and its note holds the intent and target but no
skill of its plan beyond the tell; during playback at block 3 its track holds no segment starting after block 3;
a foresight-read enemy's track is unmasked; the Gallery's `timeline_pane` renders with no session; the battle view
and the Gallery's Battle timeline build the same tracks for the same session (one builder).

## 9. Per-opener follow-up lists (INPUT1-L)

- FL1. Field: the unit and member `deck` gains `follow_ups` (object; default {}): opener family -> skill families, in
  the player's priority (INPUT1_TASKS 1.1). A key is a family of the deck's `active` or the ruleset's `basics` whose
  intent (IN4) is not neutral; every value is a family of `active` or `basics` with the key's intent; no repeats in
  a list (the key may stand in its own list: Aimed Shot after Aimed Shot); an empty list is no list (dropped on save
  and in the content writer).
- FL2. The planner's fill (FU2) after an opener reads, in order: `follow_ups[<opener family>]` when set (Q2 a: it
  replaces the order for that opener), else `order[<intent>]`, else the auto order (FU4). The list is chosen by the
  opener (the first pick, or PL3's opener in a ghost-only plan), not by later picks; an item opener reads the order.
  A family the unit cannot use in this battle (left the deck, no rank) is skipped. The same reader serves enemies and
  catalog units; no shipped data sets a list, so every plan the AI and the sims make today stays the same.
- FL3. Loadout (party screen, Skills tab deck view, RULESET1-DU): each entry of an intent's order list gets a
  "Follow-ups" toggle (`skills.follow.<family>`) that opens that opener's list under it: "After `<skill>`:", its
  entries (`skills.follow.<family>.<k>`, drag to reorder, drag keys `follow.<family>.<k>`, right click removes), the
  intent's actives and basics to add (`skills.follow.add.<family>.<family2>`, click or drag), and "Use the
  `<Intent>` order" (`skills.follow.clear.<family>`: empties the list). An entry with a set list shows a small "+"
  mark. A follow-up edit pins the automatic actives as an order edit does (`DeckTerms::base`); an actives edit that
  removes a family removes it from every list and drops its key. The Dojo's Deck window uses the same terms.
- FL4. Save: `follow_ups` sits inside the member's saved `deck`, written only when non-empty (`deck` itself only when
  used, save v13); a save without it loads as empty, and a save with no list writes byte-identical (Q9 a).
- FL5. Checks: the content check (catalog units) and `deck_errors` (the deck screen's Confirm) refuse "Follow-ups
  after `<opener>`: `<skill>` is `<Intent>`, not `<Intent>`", "... `<skill>` is not in the deck", "... `<skill>`
  twice", "Follow-ups for `<skill>`: not an opener in the deck" (key not in `active` or `basics`, or neutral).

Acceptance: on a test hero whose `order.assault` ranks A before B, a list `<opener> -> [B]` fills with B where the
order would place A, and when B cannot be placed (AP, blocks, reach) the block is Rest, not A; without the list the
fill reads `order.assault`; a Guard family in an Assault opener's list fails both checks; removing Heavy Cleave from the actives
drops its key; a v13 save without `follow_ups` loads and re-saves byte-identical; the timeline goldens and feel runs
are unchanged (no data sets a list).

## 10. VISCLAR1 slice A: a calmer board (VISCLAR1-A)

Wherever the overlay groups apply (not classic; `Layers`, RULESET1-U slice 5).

### 10.1 The enemy first-move chip

- VC1. The tell's kind chip on an enemy's token (INTENT1's `intent_chip`, the first action's kind, Flat and Stage)
  joins a new overlay group `first_move` ("Enemy first move", "The kind chip of each enemy's first action on its
  token (debug).") that is off by default. The round intent glyph (`glyphs`) stays on; the hostile card's line ("Assault:
  Warrior; Cut B1") and the area tell plates stay (open question Q5).

### 10.2 Bars in focus

- VC2. The focus set: the selected member, its round target when that is a unit, the unit under the cursor (token or
  card) and the inspected unit (IS1). By default the HP bar and the poise bar under a token draw only for the focus
  set; with Alt held, for every unit. Cards (party, hostile) keep their HP and poise bars always.
- VC3. A new overlay group `all_bars` ("All bars", "Every unit's HP and poise bars on the board; off: only the
  selected hero, its target and the unit under the mouse (hold Alt: all).") is off by default; on gives today's look.

### 10.3 One quiet target arrow per unit (user 2026-10-31 addition)

- AR1. Each unit draws at most one dashed arrow, while planning: from its tile to its target (round target; an enemy
  without a round intent: its tell's target), or move-then-target, one polyline: the move leg from its tile to the end
  tile of its moves before its first command (corners of the path dropped), then the target leg to the target, one
  arrow head at the end. A tile target ends at the tile; a self target with a visible move ends at the move's end
  with the head there; a self target with no move, or no target, draws nothing.
- AR2. The move leg shows only when that unit's move is visible to the party: a party member's always; an enemy's
  only when its plan is revealed (V2; VC4 hides foresight reveals in debug). An unrevealed enemy's arrow goes
  straight from its tile to its target. An enemy's arrow draws only while the tells show (I).
- AR3. Default view: only the selected member's arrow. With Alt held: every unit's single arrow.
- AR4. Look: dashed (4 px on, 3 off), a 4 px head, low opacity: alpha 90 of 255 (party GOLD, enemy FOE); Flat width
  1.0, Stage width 1.5 on the ground (under the tokens, the Arrows pass). The gold target ring stays as it is.
- AR5. It replaces every other line of moves and tells: the tell arrows (Flat and Stage), the planned-move lines of
  the `moves` group (walk, carried, after-move), the revealed enemy walks of `walks` (also the hovered enemy's walk
  above the tokens, RULESET1-U slices 5-6). Kept: the `moves` group's destination plates (each move's end tile, so an
  after-move still shows where it ends; open question Q6) and the area tell plates. The groups `moves` and `walks`
  now switch the move leg (off: the arrow goes straight).

### 10.4 Party foresight off (debug builds only)

- VC4. A checkbox "Party foresight off" (`settings.debug.party_foresight_off`, key `debug_party_foresight_off`,
  default false) at the foot of the Settings Battle tab, under a "Debug" heading. With it on, every enemy plan
  revealed through a party unit's `foresight` passive (Hawk Eye: the targeting rule and the `range` reveal) is drawn
  as unrevealed: board, hostile cards (V2's fill), hover cards, the intel panel, the timeline view, the arrows' move
  legs. Reveals by enrage and by the world's `plan_reveal` stay. Display only: the kit, the AI, the sims and the
  host replies are unchanged. Both rulesets (foresight exists in classic too, INTENT1).
- VC5. Kept out of release builds: the checkbox is compiled only under `#[cfg(debug_assertions)]`, and the one reader
  (`fn party_foresight_off(settings, debug: bool) -> bool`, called with `cfg!(debug_assertions)`) returns false when
  `debug` is false, so a hand-edited settings file cannot turn it on in a release build. `default_settings` does not
  hold the key.

### 10.5 Defaults

- VC6. `screens::default_settings` holds every overlay group on except `first_move` and `all_bars` (a per-group default
  in `battle::OVERLAY_GROUPS`); a saved `battle_overlays` without a group takes its default, so an older settings
  file shows the calmer view; Restore defaults returns to it. The facing chevron (`facings`, pale yellow rgb 250 226
  160) stays on.

Acceptance: with default settings the board draws no `intent_chip` and no token HP bar for a unit outside the focus
set, and Alt draws all; the agent-link `overlay.arrows` holds at most one entry per unit; by default only the
selected hero's; with Alt one per unit with a target; an unrevealed enemy's entry has two points (no move leg), a
revealed one may have three; every arrow's alpha is 90; `party_foresight_off(settings_on, false)` is false; with the
debug toggle on a Hawk Eye reveal leaves the hostile card's badge row unfilled.

## 11. RULESET1 rules: stay, change, retire

| Rule | Under INPUT1 |
| --- | --- |
| PL1 | changes: inputs are the unit, the target and its picks (the first skill or item pick is the opener) and Auto-fill; the intent and kind come from the first intent pick, else the current intent |
| PL2 | stays: the target must fit the intent; the refusal is on the menu row (IF2, IF6) |
| PL3 | stays for the ghost with no pick (GH2); with picks the opener is the first pick (FU1) |
| PL4 | stays for implied and carried moves; Move here and Step here after use the clicked tile |
| PL5 | changes: before-move, the picks in order (each with its carried move), a Step-here-after pick or AS3's reserve, the ghost fill (FL2), Rest; ghost marked (GH1-GH4) |
| PL6 | stays; new tags `pick k` (a pick after the opener), `move here`, `step after`; ghost entries keep theirs |
| PL7 | changes: no intent buttons to disable; a row is disabled with its reason and the menu title says when nothing reaches; never AP debt stays |
| PL8 | stays, with picks: later members are rebuilt from their picks; a pick that no longer fits is dropped (GH5) |
| MM1 | changes: the toggle retires; its refusals (another intent, another target) are pick refusals (IF6 a, b) |
| MM2 | retires as a mode; its uses: skills in any order and repeat, Rest anywhere, items after the opener are picks; the plain move first is Move here; bundled moves (approach, kite) are implied by the pick; a clicked carried-move tile retires (Q3) |
| MM3 | retires: Fill is the ghost (Auto-fill on); "Manual off rebuilds the auto plan" is Clear |
| MM4 | changes: a delayed opener (Rest picks, then the skill), an out-of-order follow-up, a Snap between two Rests and items as follow-ups are picks; a carried move to a tile PL4 ranks lower retires (Q3) |
| UI1 | retires (the buttons, menus and toggle); its "loses Riposte" warning moves to the menu rows (IF3) |
| UI2 | stays; ghost spans get GH8's look |
| UI4 | stays (the Dojo is out of scope) |

The kit keeps its `manual` flag and the host keeps `round_intent`, `manual` and `fill` (recordings, sims and the Dojo
send them); the battle screen no longer sends them (INPUT1_TASKS 1.3).

## 12. Out of scope

The Dojo's planner controls (UI4); classic input; the sims' drivers (`feel`, `timed`, autoplay); enemy AI and its
order lists; controller input for the battle (UI-REVIEW1 Q6 later); drag and drop on the strip; a block snapshot in
the battle's timeline view; VISCLAR1's later slices (badges, forecasts, glyph placement, the poise bar's place);
balance numbers (no rule number changes); new art (the arrow and ghost looks are drawn).

## 13. Open questions (the planner asks the user; each has a recommendation)

- Q1 (GH2). A hero with no intent at battle start (no `default_intent`): (a) its ghost is the hero tactic's pick
  (AI5), free since nothing is bound (recommended: Confirm with no input still plays a sensible round); (b) Rest
  until the player picks.
- Q2 (FL2). A set follow-up list: (a) replaces the order for that opener, as `order` replaces the auto order
  (recommended: the player gets exactly what was set, Rest when nothing qualifies); (b) the list first, then the
  intent's order.
- Q3 (MM2, MM4). The hand-picked carried-move tile: (a) retire it, PL4's tile stands (recommended: one flow, fewer
  clicks); (b) after a Maneuver pick, a click on a lit MV5 tile moves its carried move there.
- Q4 (ET4). "Step here after" for: (a) Assault, Guard and Support (recommended: Assault's kite-after is the same
  step, the decision names Guard and Support); (b) Guard and Support only.
- Q5 (VC1). Besides the token chip, should the first action leave the hostile card's line ("; Cut B1") too: (a) no,
  the chip only (recommended: the card is the detail view); (b) yes, behind `first_move` as well.
- Q6 (AR5). An after-move (kite after, Step here after, a Disengage retreat) has no place in a single target arrow:
  (a) its destination plate only (recommended); (b) a second, fainter leg after the head.
- Q7 (AP5). If the font lacks `·`: (a) add the glyph to the font (recommended, the decision's text); (b) `;`.
- Q8 (AR3, VC2). Does an inspected enemy count as selected for AR3 (its arrow shows by default)? (a) yes
  (recommended: inspecting is asking about it); (b) no, the selected hero's arrow only, as the addition reads.
- Q9 (FL4). Save: (a) `follow_ups` sits inside the member's `deck`, written only when non-empty, no save version
  bump (recommended: absent reads as empty, saves without it stay byte-identical); (b) bump to save v14.
