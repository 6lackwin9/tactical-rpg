# INPUT1: one-flow battle input, the Maneuver rework, the AP gauge, playback aids and VISCLAR1 slice A

Status: proposed (spec 2026-10-31, task INPUT1-S; revised 2026-11-01 with the user's answers "INPUT1 answers, part 1",
"part 2", "part 3" (user 2026-10-31) and "part 4" (user 2026-11-01) in `games/emberwake/docs/decisions/combat.md`, on top of "Combat input feedback
(user 2026-10-31, all as recommended)" and the user's arrow addition, section 10.3)
Depends on: RULESET1-U, RULESET1-PL (done); per task: [INPUT1_TASKS.md](INPUT1_TASKS.md) 4
Lane: logic (INPUT1-M, -U, -AP, -T, -L, -PB, VISCLAR1-A); asset (INPUT1-MA: the Maneuver skills' range and position, 11.7; DASH1: the dash clip, MR12);
balance (INPUT1-MB, a feel run after INPUT1-MA, 11.8)
Touches: by task, [INPUT1_TASKS.md](INPUT1_TASKS.md) 4. Schema contract, kit and host API, tests, parity, slices and
task rows: [INPUT1_TASKS.md](INPUT1_TASKS.md).
Rule ids: IF (flow), HK (hotkeys), GH (ghost fill), ET (empty tiles, the one regular move), SL (selection), IS
(inspect), AP (the AP forecast and gauge), TV (timeline view), FL (follow-up lists), VC and AR (clarity), MR (the
Maneuver rework), PB (playback aids); RULESET1_INTENT's PL, MM, MV and UI rules are mapped in section 13. A rule that
did not change keeps its id and text; a changed rule keeps its id and says "(changed 2026-11-01)"; a retired rule
keeps its id marked "retired".

Answers applied 2026-11-01: Q1 a (GH2, read as written: every hero with no pick); Q2 (IF2, IF10, FL2: the list
first, then the order); Q3 replaced by the Maneuver rework (section 11); Q4 (ET4, ET5); Q5 a (VC1); Q6 (AR6: arrows
while planning only); Q7 replaced by the AP pip gauge (section 7, INPUT1-AP); Q8 replaced by the playback aids
(section 12, INPUT1-PB); Q9 a (FL4). Applied 2026-11-01 from part 4: Q10 a (MR2), Q11 a (MR4), Q12 and Q13 (MR1,
MR16: one range rule for every skill, the `position` attribute, the chase for all skills; no `maneuver_range`), Q14 a
(AR5), Q15 a (AR3), Q16 a (AR6), Q17 a (11.7), Q18 a (GH2; TACTIC2 revisits the tactics), Q19 a (MR8, MR12: the dash
on the wind-up's tick with its own animation, DASH1).

History (`docs/HISTORY.md`): row 2026-10-25 "RULESET1 first draft: authored `follow_ups` and chain picks" replaced
skill-authored chains with per-intent order lists (user: chains for any skill type, fewer clicks, one intent and one
target a round). FL1 does not bring authored chains back: the lists are the player's, kept in the member's `deck`
beside `order`, hold only the opener's intent and act on the one round target, and an unset list falls back to the
order. The reason of that row still holds for skill rows; the new reason for per-opener lists is the user's
decision of 2026-10-31. Row 2026-10-25 "facing, backstab and stealth cones" (not adopted: rolls and cones on a
deterministic resolver) does not touch section 11: positions are named tiles around the target, chosen without a
roll or a cone, as RULESET1_INTENT's flanks are. Rows 2026-10-23 on diagonal views (DIR4 option D, STAGE1 option C)
concern art views, not hits; diagonal hits (MR10) need no new view and re-propose neither. The classic decisions
"Gap-close" and "Tuning notes from M1" (diagonals count as distance 2) stay: classic has no ruleset row, so
`melee_diagonal` is `none` there. Section 13 lists the RULESET1 rules this spec changes or retires (HISTORY rows:
INPUT1_TASKS 6).

## 1. Goal

Under the rule `round_intent` the battle screen takes one input flow per hero: click a target, pick a skill from a
menu of the skills usable on it; the skill sets the round intent and Maneuver kind; the planner fills the rest of the
round as ghost blocks that each further pick replaces; Confirm plays picks and ghosts alike. The intent buttons, the
Manual toggle and Fill go; keys 1-9 become skill hotkeys. Right click inspects any unit, Esc or a click on the
selected hero selects nothing, an AP pip gauge ends the strip, key T opens a full timeline of every unit (as much as
the party knows), and per-opener follow-up lists in the loadout steer the fill. A Maneuver skill becomes a dash to a
named position at its target (flank or behind) and a hit, diagonal when the rule option allows, missing when the
flank is protected. Playback gets aids to follow the simultaneous round: impact tracers, a block cursor, slower and
block-by-block speeds, the log grouped by block and the timeline view while paused. VISCLAR1 slice A calms the
board: the enemy first-move chip goes behind a debug setting, bars show for the units in focus, each unit has at most
one quiet target arrow while planning, and a debug-build toggle hides party foresight. Classic and a timeline battle
without `round_intent` keep today's input (decision "RULESET1-U slice 1": the classic screen stays as it is).

## 2. The flow (INPUT1-U)

- IF1. Targets: an enemy, an ally (the selected hero itself through the Self button, SL4), a tile. A left click on a
  unit or tile while a hero is selected and planning sets the round target at once (the gold ring, TG1; the ghost
  rebuilds, GH2) unless a skill or item pick already holds the target (IF6), then opens the skill menu at the click.
  The menu is the existing battle popup (`popup`, rows with label, detail, valid), titled "`<hero>` -> `<target>`".
- IF2 (changed 2026-11-01, Q2). Before the round's first pick the menu lists, in hotkey order (HK1), every skill of
  the hero that could act on a target of that kind under the intent table (RULESET1_INTENT 2: an enemy takes Assault
  and Maneuver skills, an ally Support skills, self Guard and Support self skills, a tile Guard tile moves (ET), area
  and hazard openers, Utility), then the pouch items of that aim ("Use `<item>` (`<n>` left)", intent by aim, IN5),
  then "Rest" (one block, neutral, RS1). A skill that can never act on that kind of target is left out; one that
  could but cannot now is listed disabled with its reason (IF6, the first failing check of FU2 c-d, MR5, PL2): "Only
  Assault skills this round (Heavy Cleave set it)", "AP 1 < 2", "Blocks 1 < 2", "Out of reach: 3 > 2", "Out of range:
  6 > 4", "Flank protected". With no usable row the title reads "No skill reaches `<target>` this round" (PL7). After
  the first pick the menu lists only the rows that can still be picked, with no disabled row: the round intent's
  skills that fit the blocks and AP left and act on the target from where the plan leaves the hero, that intent's
  pouch items, and Rest, in hotkey order. On a unit other than the round target held by a pick the menu has no skill
  row and the title "One target per round: `<target>`" (an ally's menu keeps "Select `<ally>`", SL6).
- IF3 (changed 2026-11-01: the separator). Row text: `<hotkey> <skill name>` (the digit only for HK1's first nine),
  detail `<blocks> blocks, <ap> AP; <Intent>` (Maneuver rows add the kind: "Maneuver (Engage)"; an Engage row's tip
  adds MR4's range line). A Maneuver ATTACK skill gets one row per kind the kit offers on that target, in MV11's order
  (Engage, Disengage); a Maneuver skill or foe-aimed item on a tile is Utility. A row whose pick would change the
  intent and drop intent-bound buffs keeps today's warning: a red corner and the tip line "Changing loses Riposte."
  (kit `round_intent::would_drop`). The `;` replaces the first draft's `·`, so no font glyph is needed.
- IF4 (changed 2026-11-01, Q2). Picking a row (click, or its digit while the menu is open) appends the pick (GH3);
  the menu stays open at its place and re-lists (IF2's after-the-first-pick form); with no row left it closes. The
  round intent and the Maneuver kind follow from the first skill or item pick (IN4, IN5, the row's kind); setting
  them is today's `Session::set_round_intent` / `round_intent_plan::set_kind` (an intent change drops intent-bound
  buffs then, IN6; undo does not bring them back, as with today's buttons). Neutral picks (Rest, Move here, Step here
  after) set no intent.
- IF5. Hovering a row previews it: the strip shows the plan as if the row were picked (picks solid, the new ghost
  dashed) and the AP gauge shows its numbers (AP8). Leaving the row restores the strip. A preview changes nothing.
- IF6. A pick is refused, with its reason in the row and the message bar, when (a) it is a skill or item of another
  intent than the round's current pick-set intent (MM1's "Only `<Intent>` skills this round"; to change the intent:
  Clear or undo back to before the first intent pick); (b) another unit or tile is the round target and a skill or
  item pick holds it ("One target per round: Wolf"); (c) its blocks do not fit before a Step-here-after pick (GH4) or
  the round's end; (d) it cannot be paid without AP debt after the picks before it (PL7: never planned debt); (e) it
  cannot act on the target from the tile the hero stands on when it starts (reach band, support reach, area range,
  MR4 range, MR5 a protected flank), after the opener's implied before-move (PL5).
- IF7. Undo (key Z, a right click off every unit and widget, as today's `back`) removes the last pick; Clear
  (`strip.clear`) removes every pick; the round target and the current intent stay; the ghost rebuilds (GH2).
- IF8. Gone from the battle screen: the intent buttons (`hero.intent.<id>`), the Maneuver kind and opener menus
  (`hero.kind`, `hero.opener`), Manual (`hero.manual`) and Fill (`hero.fill`). The hero panel shows instead: "Intent:
  `<Intent>` (`<kind>`) from `<first pick>`" (or "from `<tactic>`" for a ghost-only plan, GH2; "(carried over)" with
  no intent pick), "Target: `<name>`", the picks in order, the Self button (SL4) and the Rest line as today. The
  Dojo's Planner window keeps its intent, kind and opener controls (UI4: they judge the planner) and is out of scope.
- IF9. The party tactic (`strip.party`) drafts every member with no pick as today and skips members with a pick (as
  it skipped Manual members); the hero tactic (`strip.hero`) clears the selected hero's picks and its player-set
  target (GH2 a) and drafts it. A draft is ghost (GH1). Autoplay ignores Auto-fill (it drafts whole rounds).
- IF10 (new 2026-11-01, Q2). A right click anywhere, or a left click outside the open menu, closes the menu and does
  nothing else (it neither targets, inspects, selects nor undoes); Esc closes it too (SL3). The ghost fill takes the
  blocks the picks leave (GH3).

Acceptance (rule-derived; INPUT1_TASKS 3 lists the tests): a click on an enemy sets the target and opens a menu with
no Support row; a Support skill never appears on an enemy's menu; after a Heavy Cleave pick the re-listed menu has
no Guard row and no disabled row, and the Guard skill's hotkey answers "Only Assault skills this round"; after a pick
on Wolf a click on a Footman opens a menu titled "One target per round: Wolf" with no skill row; the first Maneuver
pick's row kind becomes `round_plan`'s `maneuver`; hovering a row then leaving it leaves `round_plan` equal to
before; with the menu open a right click on an enemy closes the menu and leaves `inspect`, `selected` and
`round_plan` unchanged.

## 3. Hotkeys (INPUT1-U)

- HK1. Hotkey order, per hero (kit `round_intent_plan::hotkey_skills`): the deck's actives in slot order, then the
  ruleset's `basics` that are not neutral, in `basics` order; without the deck rule, the loadout's slot order then
  the basics. Keys 1-9 are its first nine; later skills and items have no key. The order never changes during a
  battle (a fixed slot under the thumb), whatever the target.
- HK2 (changed 2026-11-01). With the menu open, a digit picks the listed row with that digit; a digit whose skill is
  disabled or no longer listed (IF2) says why in the message bar. With the menu closed, a digit picks that skill on
  the current round target (the proposed one counts, TG6) if usable, else the reason; with no round target: "Pick a
  target first (click a unit or a tile)".
- HK3. A self-only skill (DEFEND, `target_mode` SELF) picked by its key as the round's first skill pick targets self
  (no pick holds the target yet, TG1); later it needs a self or tile target already set, else the reason.
- HK4. The keys 1-9 no longer pick intents. Classic: nothing (as today). The Controls list (Settings) says "Skills of
  the selected hero (timeline): 1-9".

## 4. Ghost fill and the Auto-fill setting (INPUT1-U)

- GH1. A member's round plan is its picks in order, then ghost blocks: what the planner placed, not the player.
  Confirm plays both; a ghost block is a real plan step (reason tags, AP, forecasts as any step).
- GH2 (changed 2026-11-01, Q1 a). A hero with no pick shows the hero tactic's plan as its ghost: (a) with no round
  target the player set this round (IF1; the game keeps which members it is), the plan the hero's AUTO1 tactic
  drafts (Balanced: the AI table, AI5; `Session::auto_draft` for that member), its intent, kind and target
  included; the hero panel reads "Intent: `<Intent>` from `<tactic>`", and when the draft's intent would drop
  intent-bound buffs the panel shows "Changing loses Riposte." (IN6 applies at Confirm, as for a pick); (b) with a
  player-set target and no pick, the auto plan (PL5, PL3's opener) on that target for the current intent when it can
  act on that target, else for the first intent by target kind (an enemy: Assault, else Maneuver; an ally: Support;
  self: Guard, else Support; a tile: Guard, else Assault for an area or hazard opener, else Maneuver Utility).
  Auto-fill off: Rest on every block (a and b). A hero with no intent yet (battle start without a `default_intent`)
  takes the draft's intent for free (nothing is bound, IN2). Clear and undo back to no pick return to (a) or (b) by
  whether the target is player-set. Q18 a confirms (a) also for a hero carrying another intent; linking the tactics
  to the round intents is TACTIC2 (design), not this spec.
- GH3. A pick goes at the first block after the last pick (it replaces the ghost there); its implied moves belong to it
  (the opener's before-move, PL5; a Maneuver skill's carried move, MR12) and draw solid; then the ghost is rebuilt
  after it. Auto-fill on: FU2/FU3's fill (the opener's follow-up list, then the order, FL2), AS3's reserved
  kite-after unless a Step-here-after pick exists or the round has its regular move (ET5), then Rest. Auto-fill off:
  Rest only.
- GH4. A Step-here-after pick (ET4) reserves the round's last block like AS3; later picks and the ghost fill go
  before it; a pick that would need that block is refused (IF6 c: "Blocks 0 < 1 (the step after takes B6)").
- GH5. Each pick or unpick rebuilds the member's plan from its picks (atomic: a refused pick leaves the plan as it
  was), then every later member's (PL8). A later member's pick that no longer fits (an earlier member claimed its
  approach or position tile, PLAN1, MR3) is dropped with every pick after it, with the message "`<hero>`: `<skill>`
  dropped: `<reason>`".
- GH6. Picks reset at the round start with the target, kind and opener (`round_intent::round_start`); the intent
  carries over (IN2).
- GH7. Setting "Auto-fill the rest" (Settings, Gameplay tab, checkbox `settings.auto_fill`, key `auto_fill`, default
  true; hover: "After your picks the planner fills the round's free blocks with follow-ups (ghost blocks you can
  replace). Off: the free blocks Rest."). One setting for the whole party; changing it during planning rebuilds every
  member's ghost at once. The battle passes it to the kit (host `autofill {on}`, recorded, so a replay keeps it).
- GH8. Look: a ghost span on the strip and in the timeline view is drawn with the engine's ghost style (fill at 45%
  alpha, dashed outline, label in DIM; INPUT1_TASKS 1.4); its tip starts "Auto-filled" and then the reason tag (PL6);
  a ghost Rest block stays dim as today.

Worked examples (move 2 tiles a block; `ap_regen` 4; AP numbers as AP1, the gauge's pips in section 7):

| Hero, AP, max, situation | Input | Plan (picks solid, ghost in brackets) | AP: now - spent = left, next |
| --- | --- | --- | --- |
| tl_vanguard, 10, 10, footman 3 tiles away | click footman, Heavy Cleave | b1 approach (2), b2-b5 Heavy Cleave (4), [b6 Shoulder Check (1)] | 10 - 7 = 3, next 7 |
| same, Auto-fill off | the same | b1 approach, b2-b5 Heavy Cleave, [b6 Rest] | 10 - 6 = 4, next 8 |
| tl_archer, 10, 10, footman adjacent | click footman, Aimed Shot, Snap Shot | b1 kite (2), b2-b3 Aimed Shot (2), b4 Snap Shot (2), [b5-b6 Aimed Shot (2)] | 10 - 8 = 2, next 6 |
| tl_vanguard, 3, 10, footman adjacent, tactic Balanced | click footman, Heavy Cleave | refused: "AP 3 < 4" (row disabled, gauge red on hover); the ghost stays [b1-b3 Shoulder Check (1 each), b4-b6 Rest] | 3 - 3 = 0, next 5 (Rest 3 blocks: +1) |

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
- ET4 (changed 2026-11-01, Q4). "Step here after" (skill then move, the after-move): one standard move block at the
  round's last block (`PlanStep::after_move`, AS3's shape), 1 AP a tile, at most one block's tiles; offered after a
  skill or item pick, for every intent (Assault, Guard, Support, Maneuver), only while the round has no regular move
  (ET5); refused "Already moved this round (approach)" (the kind of the planned move: approach, kite, move to the
  Guard tile, Move here, out of range), "Too far for one move block: 3 > 2", "AP", "Blocks 0 < 1 (the picks fill the
  round)", "One step after a round". It replaces a ghost AS3 kite. Its tile is the clicked one (PL4 does not re-rank
  it). The first draft's refusal "Disengage retreats with its skill" retires: a carried retreat (MV8) is not the
  regular move.
- ET5 (new 2026-11-01, Q4). One regular move a round: a plan holds at most one regular move, either at the start (a
  before-move: AS1's approach, AS2's kite, GD1's move to the Guard tile, SP1's and MV10's approach, MR7's move when
  the target is beyond range, Move here) or at the end (an after-move: AS3's kite, Step here after), never both. A
  Maneuver skill's carried move (MR12, MV8's retreat) and a DISENGAGE skill's jump are not that move, so a Maneuver
  round can dash in, hit, kill and still step away. The auto planner already plans one (PL5: AS3 only without a
  before-move); picks refuse a second ("Already moving this round", ET3; "Already moved this round", ET4).

Acceptance: "Move here" on a tile 5 tiles away is disabled "Too far"; after "Move here" an Assault pick is measured
from the new tile (no approach entry follows the move); "Move here, then Active Guard" plans a move entry then the
Guard skill with the tile as `round_plan` target; "Step here after" puts an after-move entry at the last block and a
later 2-block pick that would cover it is refused; after a Heavy Cleave pick that implies an approach, "Step here
after" is disabled "Already moved this round (approach)"; after a Flank Cut pick whose dash carries the hero 2 tiles
(no regular move), "Step here after" is offered; no `plan_unit` plan holds both a before-move and an after-move.

## 6. Selection and inspect (INPUT1-U)

- SL1. With a hero selected (planning), a left click on an enemy sets the target (IF1).
- SL2. A right click on any unit (token or card; enemy, ally or the selected hero) inspects it (IS1); it never
  targets, selects or undoes. A right click elsewhere keeps today's `back` (close the popup, the log, else undo the
  last pick). With the skill menu open a right click only closes it (IF10).
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
- IS3 (changed 2026-11-01). An inspected party member shows its intent, target, picks and plan, its AP numbers (AP9's
  lines) and statuses; it does not change the selected hero.

Acceptance: right click on the selected hero leaves `selected` and `round_plan` unchanged and sets `inspect`; Esc
with a menu open closes only the menu; with no hero selected a left click on an enemy sets `inspect`, not a target;
an inspected unrevealed enemy's panel text contains none of its plan's skill names (other than its tell's first
action while tells show).

## 7. The AP forecast (INPUT1-U) and the AP pip gauge (INPUT1-AP)

The decision's text readout (`AP 6 - 4 = 2 · +3 next -> 5`) gives way to a pip gauge (Q7, user 2026-10-31). INPUT1-U
builds the kit forecast (AP1, AP2) and the host's `ap` keys and draws no strip readout (today's AP display stays as
it is until then); INPUT1-AP draws the gauge.

- AP1 (changed 2026-11-01: numbers only). For a party member while planning, kit `round_intent_plan::ap_forecast`:
  `now` the member's AP, `spent` every planned command's AP (picks, their moves and the ghost; split into `picked`
  and `filled`), `left = now - spent`, `next` the AP the round's end would give with no hit taken (`min(max_ap, left +
  ap_regen + ap_regen_bonus)` then RS2's Rest gain of the planned Rest blocks, capped at `max_ap`, as
  `status::roll_over` and `round_intent::round_start` apply them), `gain = next - left`, `rest_gain`, `max`.
- AP2. It is rebuilt on every pick, unpick, Clear, target change, Auto-fill change and replan; while a menu row is
  hovered the gauge shows that row's preview (IF5).
- AP3 (changed 2026-11-01). The gauge turns red (FOE) when the numbers shown cannot be paid: the hovered row's `left`
  would be below 0 (the row is disabled, IF6 d), or `now` is already below 0 (AP debt from earlier); AP8 draws both.
- AP4 retired (2026-11-01): the readout's short text form; the gauge has no text to fit (its numbers are in AP9's tip).
- AP5 (changed 2026-11-01). Only under `round_intent` and for party-side units (enemies pay no AP, AI2). The first
  draft's glyph question retires with the text.
- AP6 (new). Place and size: widget `strip.ap` at the strip's end, after the last block and before the tactic menus,
  for the selected party member. One pip a point of AP up to `max_ap`, at most 12 pips (game constant `AP_PIPS` 12,
  the user's first try); with `max_ap` above 12 a pip stands for `u = ceil(max_ap / 12)` AP and every count below is
  divided by `u` and rounded up. A pip is 4 x 6 px with a 1 px gap (12 pips: 59 px), centred on the strip lane; the
  blocks lane leaves it 59 px plus the debt label (AP8). Drawn by the engine widget `aegis_ui::pips` (INPUT1_TASKS
  1.4b), shown alone in the UI Gallery's Kit group as `pip_gauge` (sample states: the four rows below, red, debt, a
  14 AP hero at 2 AP a pip).
- AP7 (new). Planning look, pip `i` from 1 to `max`: fill full (GOLD) when `i <= left`; ghost (GOLD at 45% alpha,
  GH8's alpha) when `left < i <= next`; none otherwise. Outline solid (TEXT) when `i <= now`; a DIM slot when
  `now < i`. So a full pip is AP kept, a hollow pip is AP the plan spends (ghost-filled when next round returns it),
  and a ghost-filled slot is next round's gain above what the hero has now.
- AP8 (new). Red and debt: hovering a row the hero cannot pay (AP3) draws every outline FOE and a FOE label "`-<n>`"
  after the last pip (`n` = the shortfall, `-left` of the preview); `now < 0` draws every pip as a DIM slot and a FOE
  label "`-<n>`" before the first pip (`n` = the debt). Leaving the row restores TEXT and GOLD.
- AP9 (new). Numbers on hover: the tip reads, one per line, "AP `<now>` of `<max>`", "Plan: `<spent>` (picked
  `<picked>`, auto-filled `<filled>`)", "Left: `<left>`", "Next round: +`<gain>` -> `<next>` (Rest +`<rest_gain>`)",
  and with `u > 1` "1 pip = `<u>` AP". The timeline view's label tip (TV3) and the inspect panel (IS3) use the same
  lines.
- AP10 (new). Playback: the gauge shows the selected party member's live AP (full pips up to its AP, slots to
  `max`); when one of its commands starts and pays (FU6), those pips turn hollow over 150 ms, the rightmost first; at
  the round end the regained pips fill over 150 ms; with skip animations or reduce motion at once. With no party
  member selected the gauge is hidden.

Gauge for the worked examples of section 4 (F full, H hollow, Hg hollow with a ghost fill, Sg a slot with a ghost
fill, S an empty slot):

| Example | now, spent, left, next, max | Pips 1 to 10 |
| --- | --- | --- |
| Vanguard, Heavy Cleave | 10, 7, 3, 7, 10 | F F F Hg Hg Hg Hg H H H |
| same, Auto-fill off | 10, 6, 4, 8, 10 | F F F F Hg Hg Hg Hg H H |
| Archer, Aimed Shot, Snap Shot | 10, 8, 2, 6, 10 | F F Hg Hg Hg Hg H H H H |
| Vanguard at 3 AP, ghost only | 3, 3, 0, 5, 10 | Hg Hg Hg Sg Sg S S S S S |

Acceptance: the four rows above read exactly so in the agent-link `ap_gauge.pips`; hovering a row the hero cannot pay
sets `ap_gauge.red` and the label "-1" for a 4 AP row at 3 AP, and leaving it clears both; a hero at -2 AP shows
"-2" before ten DIM slots; a 14 AP hero shows 7 pips at 2 AP each; in playback a command paying 4 AP turns four pips
hollow; on a dummy fight (no AP effects) `next` equals the hero's AP at the next planning.

## 8. The full timeline view (INPUT1-T)

- TV1 (changed 2026-11-01, PB5). Key T (planning and playback, rule `round_intent` or any timeline battle) toggles an
  overlay over the battlefield panel: rect (126, 28, 388, 292) on the 640 x 360 canvas, the grid panel's own rect; T
  or Esc closes it (Esc closes it first, before SL3's steps). While planning it pauses nothing; during playback it
  pauses the playback at the end of the resolving block while open (PB5). While open the board under it takes no
  click.
- TV2. It draws the engine timeline widget (`aegis_ui::timeline::Timeline`): the ruler of the round's blocks (the
  playhead during playback), section "Party" (allies in roster order, guests too) above section "Enemies" (in the
  hostile list's order); one track per unit present or acting this round; a downed unit's track is hatched.
- TV3 (changed 2026-11-01). A party track holds the member's plan while planning (picks solid, ghost in the ghost
  style, GH8; parts and BREAK marks as the strip), its committed actions during playback; the label leads with the
  intent glyph's letter ("A Vanguard"); the selected hero's track is the selected track; its label tip carries AP9's
  lines.
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
- FL2 (changed 2026-11-01, Q2 b). The planner's fill (FU2) after an opener tries, block by block, the entries of
  `follow_ups[<opener family>]` in order, then those of `order[<intent>]` (else the auto order, FU4); the first entry
  that qualifies (FU2 a-e) is placed; a family in both is tried at its first place. The list is chosen by the opener
  (the first pick, or PL3's opener in a ghost-only plan), not by later picks; an item opener reads the order only. A
  family the unit cannot use in this battle (left the deck, no rank) is skipped. The same reader serves enemies and
  catalog units; no shipped data sets a list, so every plan the AI and the sims make today stays the same.
- FL3. Loadout (party screen, Skills tab deck view, RULESET1-DU): each entry of an intent's order list gets a
  "Follow-ups" toggle (`skills.follow.<family>`) that opens that opener's list under it: "After `<skill>`:", its
  entries (`skills.follow.<family>.<k>`, drag to reorder, drag keys `follow.<family>.<k>`, right click removes), the
  intent's actives and basics to add (`skills.follow.add.<family>.<family2>`, click or drag), and "Use the
  `<Intent>` order" (`skills.follow.clear.<family>`: empties the list). An entry with a set list shows a small "+"
  mark. A follow-up edit pins the automatic actives as an order edit does (`DeckTerms::base`); an actives edit that
  removes a family removes it from every list and drops its key. The Dojo's Deck window uses the same terms.
- FL4 (Q9 a). Save: `follow_ups` sits inside the member's saved `deck`, written only when non-empty (`deck` itself
  only when used, save v13); a save without it loads as empty, and a save with no list writes byte-identical. No save
  version bump (one save version at the end).
- FL5. Checks: the content check (catalog units) and `deck_errors` (the deck screen's Confirm) refuse "Follow-ups
  after `<opener>`: `<skill>` is `<Intent>`, not `<Intent>`", "... `<skill>` is not in the deck", "... `<skill>`
  twice", "Follow-ups for `<skill>`: not an opener in the deck" (key not in `active` or `basics`, or neutral).

Acceptance (changed 2026-11-01): on a test hero whose `order.assault` ranks A before B, a list `<opener> -> [B]`
fills with B where the order would place A, and when B cannot be placed (AP, blocks, reach) the block takes A from
the order; without the list the fill reads `order.assault`; a Guard family in an Assault opener's list fails both
checks; removing Heavy Cleave from the actives drops its key; a v13 save without `follow_ups` loads and re-saves
byte-identical; the timeline goldens and feel runs are unchanged by INPUT1-L (no data sets a list).

## 10. VISCLAR1 slice A: a calmer board (VISCLAR1-A)

Wherever the overlay groups apply (not classic; `Layers`, RULESET1-U slice 5).

### 10.1 The enemy first-move chip

- VC1 (Q5 a). The tell's kind chip on an enemy's token (INTENT1's `intent_chip`, the first action's kind, Flat and
  Stage) joins a new overlay group `first_move` ("Enemy first move", "The kind chip of each enemy's first action on
  its token (debug).") that is off by default. The round intent glyph (`glyphs`) stays on; the hostile card's line
  ("Assault: Warrior; Cut B1") and the area tell plates stay. Showing an enemy's known moveset without how it fills
  the timeline is a later idea of the user's (out of scope).

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
- AR3 (changed 2026-11-01, Q15 a). Default view: the selected member's arrow and the inspected unit's (IS1). With Alt
  held: every unit's single arrow.
- AR4. Look: dashed (4 px on, 3 off), a 4 px head, low opacity: alpha 90 of 255 (party GOLD, enemy FOE); Flat width
  1.0, Stage width 1.5 on the ground (under the tokens, the Arrows pass). The gold target ring stays as it is.
- AR5. It replaces every other line of moves and tells: the tell arrows (Flat and Stage), the planned-move lines of
  the `moves` group (walk, carried, after-move), the revealed enemy walks of `walks` (also the hovered enemy's walk
  above the tokens, RULESET1-U slices 5-6). Kept while planning: the `moves` group's destination plates (each move's
  end tile, so an after-move shows only as its destination plate, Q14 a) and the area tell plates. The groups
  `moves` and `walks` now switch the move leg (off: the arrow goes straight).
- AR6 (new 2026-11-01, Q6). Planning only: the arrows and the `moves` group's destination plates draw only while the
  party plans its round; during playback and after the battle the board draws no target arrow, move leg, destination
  plate, planned-move line or area tell plate (Q16 a). The fight shows itself through its walks, clips, FX and PB1's
  tracers. The arena telegraph marks stay through playback (decision "HOOKS1-UI slice 1 answers", until VISCLAR1
  decides).

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
revealed one may have three; every arrow's alpha is 90; during playback `overlay.arrows` is empty and no destination
plate draws; `party_foresight_off(settings_on, false)` is false; with the debug toggle on a Hawk Eye reveal leaves
the hostile card's badge row unfilled.

## 11. The Maneuver rework (INPUT1-M)

From "INPUT1 answers", parts 2, 3 and 4 (user 2026-10-31 and 2026-11-01). One range rule for every skill; a Maneuver
skill differs only by where it ends up: a skill with a `position` dashes to that spot by its target and hits from
there, so its range N is how far it dashes; the move is inside the skill and is not the round's regular move; the
regular move comes first only when the target is beyond range. Kit rules (`rpgkit_battle`), ruleset options
(`rpgkit_rules`), the skill field (`rpgkit_data`, schema); the game adds the Arena select, the hover text and the dash
draw hook.

### 11.1 Range and position

- MR1 (changed 2026-11-01, Q12 and Q13). No separate Maneuver range field. Every skill's range is its reach band
  (`reach_min`-`reach_max`, RULESET1 K1-K2; 0 = the unit's default band: melee 1, ranged 2-3, support 0-3), the same
  targeting rule for every skill, with the chase of MR16. New family field `position` ("", `flank`, `behind`; default
  ""; INPUT1_TASKS 1.7), allowed on an ATTACK whose round intent is Maneuver, melee or ranged: a skill with a
  position dashes to that spot by the target (MR2-MR4) and hits from there, so its range N is how far it dashes, N =
  its `reach_max`, else the unit's default band max (its `reach_min` is not read). A ranged skill with a position
  dashes too. A skill without a position hits from where it stands, within its band (a Maneuver skill without one
  included). Transition until INPUT1-MA moves the data: a Maneuver ATTACK with `dash_tiles` above 0 and no position
  reads as position `flank` with N = `dash_tiles` (content warning, INPUT1_TASKS 1.7). `dash_tiles` stays classic
  gap-close on Assault skills (decision "Gap-close") and retires on Maneuver skills when INPUT1-MA sets it to 0.
- MR16 (new 2026-11-01, Q12). The chase, for every skill: under `round_intent`, every skill command on a unit target
  (attacks, area skills aimed at a unit, Support skills on an ally) chases up to 1 tile: at its start, when the
  target stands outside the skill's band from the unit's tile, the unit steps 1 tile toward the band (or away when it
  is too close), today's `follow` with `follow_tiles` 1 (AS1), at today's chase cost (1 AP a tile for a party unit,
  none for enemies), blocked by root, a hidden target (GD5), units and walls; log `[Chase]` as today. A skill with a
  position chases inside its dash (MR8: the budget N + 1, the extra tile at the chase's cost). A chase is not the
  round's regular move (ET5). Classic keeps its chase rules.

### 11.2 Positions

Let T be the target on tile t with facing f (MV1, a sign vector; if T has none, f points from t toward the attacker).
For each of T's 8 neighbours n, let k be the number of 45-degree steps between the direction n - t and -f (0 to 4).

- MR2. Position tiles: `behind` prefers the neighbour with k = 0; `flank` prefers the two with k = 2 (either side).
  The fallback for both, "the nearest empty diagonal of that position", is the two neighbours with k = 1 (between the
  sides and the back). Neighbours with k = 3 or 4 are in T's front (MV3) and are never positions, not even as a
  fallback (Q10 a).

| f (T faces) | behind (k 0) | diagonals (k 1) | flanks (k 2) | front, never (k 3, 4) |
| --- | --- | --- | --- | --- |
| (1, 0) right | (-1, 0) | (-1, -1), (-1, 1) | (0, -1), (0, 1) | (1, -1), (1, 1), (1, 0) |
| (1, 1) down-right | (-1, -1) | (-1, 0), (0, -1) | (-1, 1), (1, -1) | (0, 1), (1, 0), (1, 1) |

(Offsets from t; the other six facings rotate these.)

- MR3. A tile is open when it is on the grid, walkable, not an exit, no unit other than the attacker stands on it,
  and (while planning) no earlier member's planned move claims it (PLAN1). `melee_diagonal` (MR10) drops the tiles the
  skill cannot hit from: under `none` the diagonal neighbours of t are never positions (with f = (1, 1) that removes
  behind and both flanks, so the k = 1 tiles stand alone). The candidates are the open preferred tiles, else the open
  fallback tiles. With no open tile in either tier the flank is protected (MR5, MR8 c).

### 11.3 Range and planning

- MR4 (changed 2026-11-01). Range of a skill with a position: from the tile s the unit stands on at the skill's start
  (after the round's regular move when one comes first, MR7), a candidate is in range when it is s itself (cost 0) or
  has a route of move cost at most N in one block (`dash_fits`); the skill is in range when one candidate is. The
  tile it takes: the open preferred tiles in range, else the open fallback tiles in range (Q11 a: an open preferred
  tile out of range does not stop a fallback in range); inside the preferred tier PL4 ranks them, inside the fallback
  tier fewer path tiles first ("nearest"), then PL4. When the taken tile is s there is no move. It is stored on the
  step (`Step.cell`, MV6). The hit is from that neighbour of t, so at distance 1 (a diagonal under MR10), whatever
  the skill's band. Row tip and hover card: "Range `<N>`: dash `<to a flank | behind>`, then hit" (replaces MV5's
  "Range: move N + reach a-b"); a skill without a position keeps its band line ("Range 2-4").
- MR5. Flank protected while planning: an Engage whose target has no open candidate is not planned (PL3 and the AI
  skip it) and its menu row (and today's intent tip until INPUT1-U) is disabled "Flank protected". Disengage and
  Utility rows are not affected.
- MR6 (changed 2026-11-01: skills with a position). Follow-ups (MV7's "each moving only when its tile no longer meets
  the condition"): a later skill with a position moves only when the unit's tile at its start is not a position tile
  (preferred or fallback, MR2-MR3) of that skill around T's tile then; else it hits from there.
- MR7. The regular move (MV9 a, TQ22 a, kept): only when the target has open candidates but none in range from the
  unit's tile, a standard plain move comes first (1 AP a tile, at most 2 move blocks, reason `approach: out of range`)
  to the PL4 tile from which a candidate is in range (preferred tier first). It is the round's one regular move
  (ET5). With no open candidate no move is planned (MR5).

Worked examples (Wolf on (6, 4) faces right, engaged with tl_vanguard on (7, 4): behind (5, 4), diagonals (5, 3) and
(5, 5), flanks (6, 3) and (6, 5); `melee_diagonal` maneuver; N is the skill's `reach_max` after INPUT1-MA):

| Attacker, skill (position, range N), from | Board | Plan and hit |
| --- | --- | --- |
| tl_spiker, Hamstring Cut (behind, 3), (3, 4) | open | dash (4, 4), (5, 4) (cost 2); hits from behind: a flank (MV3) |
| same | a goblin on (5, 4) | dash to (5, 3) or (5, 5) (3 tiles each; PL4 ranks); diagonal hit |
| same | units on (5, 4), (5, 3), (5, 5) | row disabled "Flank protected" (the flank tiles are not behind's fallback) |
| tl_spiker, Flank Cut (flank, 2), (5, 2) | open | dash to (6, 3) (cost 2); side hit |
| same | a unit on (6, 3) | (6, 5) is 4 away: fallback (5, 3), 1 tile; diagonal hit |
| same, from (2, 4) | open | out of range (the nearest candidate is 4 tiles away): b1 plain move 2 tiles (2 AP; PL4's tile, such as (4, 3)), then Flank Cut dashing to (6, 3); "Step here after" not offered (ET5) |

### 11.4 Run time

- MR8 (changed 2026-11-01, Q19 a). On the wind-up's tick, the skill's start block (its `on_start`, after its AP,
  FU6; today's timing), the candidates are read around T's tile then (T may have moved, and its facing is re-read,
  MV1) with the budget N, plus 1 when T no longer stands on its tile at planning (MR16's chase; that tile at the
  chase's cost). The stored tile is kept when it is in the first tier with a tile within the budget, else that tier's
  first by MR4's ranking. (a) A candidate within the budget: the unit dashes there (hazards on entry as today;
  `round_intent::note_moved`), log `[Dash] <unit> dashes at <target>: <from> to <to>`, and the hit lands at the
  impact. (b) Open candidates but none within the budget: no move,
  `[Dash Blocked] <unit> can't reach <target>'s flank`, and the hit lands at its impact only if T is adjacent (MR10)
  to where the unit stands, as MV6 today. (c) No open candidate:
  `[Flank Protected] <unit> finds <target>'s flank protected: <skill> misses (<n> AP spent)`; no move, and at its
  impact the skill does nothing (no damage, stagger, status, rider, hit event, riposte, `flanked` exploit or flank
  count). The command keeps its blocks: its wind-up plays and can still be broken, and an enemy books its recovery
  as after any impact. Rooted and a hidden target (GD5) keep today's lines.
- MR9. The cost of a protected flank (ruleset field `flank_protected_cost`, a rule value): `full` (default: the AP
  paid at the start stays spent), `half` (it pays `floor(cost / 2)`; the rest is refunded with the log line), `fixed`
  (it pays `min(cost, flank_protected_ap)`, default 1), `none` (refunded). `<n>` in the log is what it pays.
  Enemy-side units pay no AP (AI2).

### 11.5 Diagonal melee (rule-library option)

- MR10 (changed 2026-11-01: skills with a position). Ruleset field `melee_diagonal` (`none`, `maneuver`, `all`;
  default `maneuver`; classic has no row: `none`; a row without `round_intent` reads `maneuver` as `none`). For the
  skills it covers (`maneuver`: skills whose round intent is Maneuver, hitting from a position or with a melee band;
  `all`: every melee ATTACK skill, every skill with a position and the riposte counter), a diagonal neighbour of the
  target (one step on each axis, Manhattan 2) counts as distance 1 in the skill's band test (in reach, the planner's
  band tiles, the chase). Reach 1-2 bands reach diagonals already and do not change; ranged bands do not change.
  Under `all` MV2's engaged test counts diagonals too; PL4's threat estimate stays Manhattan under every value.
  Under `maneuver` a riposte does not answer a diagonal Maneuver hit (the counter is not a Maneuver skill; a flank hit
  gets none anyway, MV3). A diagonal attacker faces T by MV1's sign vector; the board picks its view as for any
  diagonal facing today (DIR4), with no new art.
- MR11. Switching it to compare: the Scene Runner's Arena card gets a "Diagonal melee" select (Ruleset, None,
  Maneuver, All; default Ruleset: the row's value), kept on Restart and in playtest playback; the request carries
  `melee_diagonal`; the ruleset chip adds "diagonal: `<value>`"; the host's `new` takes `melee_diagonal`; the sim
  takes `melee_diagonal=none|maneuver|all`; the battle state's `ruleset` gains `melee_diagonal`.

### 11.6 Display, AI and text

- MR12 (changed 2026-11-01, Q19 a). The dash is drawn with its own dash animation, distinct from a regular move: the
  kit marks the carried move's board event as a dash (`MoveKind::Dash` beside the walk, on the event the battle
  screen animates; INPUT1_TASKS 1.8), and the battle screen (the draw hook is INPUT1-M's) plays the rig's `dash` clip
  from the start block's start along the route, faster than a walk (the route in 250 ms at 1x; `anim_speed` and PB3
  apply), then the wind-up pose holds on the position until the impact (WINDUP1), or the strike follows for a
  one-block skill. A rig without a `dash` clip (every rig until DASH1 lands) plays its walk at that speed. Skip
  animations: the unit appears on the position. It is not the round's regular move (ET5); the strip draws it inside
  the skill's span, not as a move block. Art: DASH1 (the dash clip for the humanoid rigs, front, side and back).
- MR13. Hovering a Maneuver Engage row (IF5; INPUT1-U, since the menu is its) marks T's open candidates on the board:
  preferred tiles with a GOLD ring plate, fallback tiles with a DIM one; none when the flank is protected (the row
  says so). Group `moves`; planning only (AR6).
- MR14 (changed 2026-11-01). AI: AI table row 4 reads "a Maneuver attack in range (MR4) of an open candidate on a
  ranked target engaged with another unit"; the AUTO1 tactics are unchanged here (TACTIC2 revisits them). Disengage
  (MV8): the hit from the unit's tile (the target in band, or adjacent with a diagonal per MR10 for a skill with a
  position), then the retreat up to N tiles (the skill's range, MR1).
- MR15 (changed 2026-11-01). Skill text (`rpgkit_world/src/skill_text.rs`): `{range}` renders N for a skill with a
  position (else the band, "2-4"), `{position}` renders "to a flank" or "behind the target"; `{dash}` on a Maneuver
  skill renders the transition's N (MR1), so today's descriptions keep working until INPUT1-MA.

Acceptance (rule-derived; INPUT1_TASKS 3): each row of the worked examples; with Wolf facing (1, 1) a behind skill
prefers t + (-1, -1); under `none` a Flank Cut never ends on a diagonal of t and the f = (1, 1) case uses only the
k = 1 tiles; filling the candidates between planning and the start gives `[Flank Protected]` with the AP of each
`flank_protected_cost` value (4 AP skill: full 4, half 2, fixed 1, none 0) and no hit event; Wolf stepping 1 tile
before the start is followed with budget N + 1 and the chase's 1 AP when the extra tile is used; a ranged skill with
a position and `reach_max` 4 dashes up to 4 to the position and hits from that neighbour; a ranged Aimed Shot whose
target stepped 1 tile out of its band follows 1 tile at 1 AP (MR16), and so does a heal whose ally stepped out of
support reach; a Strike from (5, 3) on Wolf does not reach under `maneuver` and does under `all`, where a riposte also
answers it; under `maneuver` a diagonal Flank Cut draws no riposte; the dash moves the unit at the skill's start, its
board event is a dash (not a walk) and no regular move counts it (ET5); the transition reads a Maneuver skill's
`dash_tiles` 2 as flank, N 2, with the warning; the classic Godot parity suites and the inert-extensions check pass
unchanged.

### 11.7 Assets (INPUT1-MA, asset lane, after INPUT1-M merges)

Through `aegis content` (a seed `games/emberwake/data/seeds/input1_ma.json` or upserts; older seeds stay as they
are). Positions as answered (Q17 a). `dash_tiles` is set to 0, its default: the content writer leaves default values
out of a row, so the key leaves these three rows (the field itself stays for classic gap-close). Ranges: today the
three rows set no `reach_max` (0: the unit's melee band, 1).

| Row | Field | Today | New |
| --- | --- | --- | --- |
| `skills.flank_cut` | `dash_tiles` / `reach_max` / `position` | 2 / 0 / "" | 0 / 2 / `flank` |
| `skills.flank_cut` | `description` | "Maneuver, Snap: {name}: {power} power. Dashes up to {dash} to a flank, {stagger} stagger." | "Maneuver, Snap: {name}: {power} power, {stagger} stagger. Dashes up to {range} {position}, then hits." |
| `skills.tl_hamstring_cut` | `dash_tiles` / `reach_max` / `position` | 3 / 0 / "" | 0 / 3 / `behind` |
| `skills.tl_hamstring_cut` | `description` | "... Dashes up to {dash}; {exploit}." | "Maneuver, Snap: {name}: {power} power, {stagger} stagger. Dashes up to {range} {position}; {exploit}." |
| `skills.tl_slip_strike` | `dash_tiles` / `reach_max` / `position` | 2 / 0 / "" | 0 / 2 / `flank` |
| `skills.tl_slip_strike` | `description` | "... Dashes up to {dash}." | "Maneuver, Standard: {name}: {power} power, {potency} to Slow for {rounds}. Dashes up to {range} {position}." |
| `rulesets.timeline` | `melee_diagonal` / `flank_protected_cost` | - / - | `maneuver` / `full` (written out, though the defaults) |

No other shipped skill is a Maneuver skill: `lunge`, `pounce`, `charge_in` (Assault by kind, classic gap-close) and
`backflip`, `leap_back` (DISENGAGE jumps, `jump_tiles`) keep their fields. Acceptance: the content check prints none
of 1.7's transition warnings; the three descriptions render "Dashes up to 2 to a flank" and "Dashes up to 3 behind
the target".

### 11.8 Balance (INPUT1-MB, balance lane, after INPUT1-MA on `main`)

A feel run as RULESET1-B's (`tools/fc.sh sim feel seeds=100 party=both` on the five `tl_` fights and
`tl_flank_elite`), plus `melee_diagonal=none` and `=all` on `tl_flank` and `tl_skirmish`. Report against the last
run (BALANCE_REPORT: `tl_flank` 92 auto / 98 timed against its 50-85 band, known over band; carried moves 222 / 224 in
`tl_flank` and 478 / 502 in `tl_skirmish`): win rates, the carried-move floors (still above 0), protected flanks per
100 fights and their share of Maneuver commands, diagonal hits, Maneuver rounds, and chases per 100 fights by skill
kind (MR16 adds ranged and Support chases). No band is set for the new counts; the user judges them (decision
"Timeline test fights after RULESET1-A3": the `tl_` fights are judged on floors).

## 12. Playback aids (INPUT1-PB)

From "INPUT1 answers, part 2" (Q8): aids to follow the simultaneous playback; the user trims them after playing.
Timeline battles only (a ruleset row); classic playback is unchanged.

- PB1. Impact tracers: at each impact (the hit frame, FX1-L1's moment of the hurt flash) the board draws a straight
  line from the actor's token centre to each unit the command landed on: an attack's hit (damage or stagger), each
  unit an area hit catches, a riposte counter, and a support landing on another unit (heal, buff, cleanse); a hazard
  placement ends at its tile; a miss (MR8 c, GD5's "loses") draws it dashed (4 on, 3 off). No tracer for self-only
  commands, ground or arena hazard ticks. Look: 1 px Flat, 1.5 px Stage, above the tokens, shortened 3 px at each
  end; party GOLD, enemy FOE, a support landing in the heal float's colour; alpha 200 fading to 0 over 400 ms (wall
  time, `anim_speed` and PB3 apply). Skip animations: the block's tracers draw together for 400 ms when its result
  shows; reduce motion: no fade (hold 400 ms, then gone). Overlay group `tracers` ("Impact tracers", "A short line
  from each actor to what its action hit, during playback."), on by default.
- PB2. Strip cursor: during playback the strip frames the resolving block (a 1 px GOLD outline around its column on
  every strip lane shown, and its ruler number in GOLD) from the block's start until the next block's; none at the
  round's start and end steps. The timeline view's playhead (TV2) marks the same block.
- PB3. Playback speed: three chips shown during playback at the strip's tactic menus' place, "1x"
  (`playback.speed.1`), "0.5x" (`playback.speed.half`), "Block" (`playback.speed.block`), and in Block mode "Next
  block" (`playback.next`); key P cycles 1x, 0.5x, Block; key N is Next block. Setting `playback_speed` ("1x",
  "0.5x", "block"; default "1x"; also on the Settings Battle tab) is remembered. 0.5x doubles every playback duration
  (walks, clips, floats, tracers, the pacing between blocks) on top of `anim_speed`. Block: after each block's
  animations end the playback waits, the message bar reading "Block `<b>` of `<n>`: N for the next"; N or Next
  block plays the next one. Skip animations: 0.5x plays as 1x; Block still waits. Autoplay rounds wait in Block mode
  too.
- PB4. Log grouped by block: the combat log (compact and expanded) groups lines by the round and block they were
  written in (`log::Entry.round`, `.block`): a "Round `<r>`" header, then a DIM "Block `<b>`" sub-header with a 1 px
  rule and its lines indented 6 px; lines outside the round's blocks group under "Round start" or "Round end" (the
  "[Round]" summary line sits there); a block with no line has no header. During playback the resolving block's
  group is the one the compact log shows. Headers and lines pass `text_audit` at the four text sizes.
- PB5. The timeline view while paused: during playback T opens the view and pauses the playback at the end of the
  resolving block (the view shows what has started, TV4); closing it resumes; in Block mode the view may stay open
  while N steps (its playhead and tracks follow).

Acceptance: a block with a landed hit puts one entry per hit in the agent-link `overlay.tracers` (from, to, alpha,
dashed), a `[Flank Protected]` miss one dashed entry, and a self Defend none; with the group off none; `playback`
{speed, waiting, block} reads "block", true and the block number after a block in Block mode, and N advances it by
one; at 0.5x a block takes twice the 1x time in a timed test; every log line sits under its block's header; T during
playback sets `playback.waiting` at the block's end and closing the view clears it.

## 13. RULESET1 rules: stay, change, retire

| Rule | Under INPUT1 |
| --- | --- |
| PL1 | changes: inputs are the unit, the target and its picks (the first skill or item pick is the opener) and Auto-fill; the intent and kind come from the first intent pick, else the current intent |
| PL2 | stays: the target must fit the intent; the refusal is on the menu row (IF2, IF6) |
| PL3 | stays for the ghost with a player-set target and no pick (GH2 b); with picks the opener is the first pick (FU1); a protected Engage is skipped (MR5) |
| PL4 | stays for implied moves; ranks position tiles inside MR4's tiers; Move here and Step here after use the clicked tile |
| PL5 | changes: before-move, the picks in order (each with its carried move), a Step-here-after pick or AS3's reserve, the ghost fill (FL2), Rest; ghost marked (GH1-GH4); one regular move (ET5) |
| PL6 | stays; new tags `pick k` (a pick after the opener), `move here`, `step after`, `flank protected` (the Rest cause); ghost entries keep theirs |
| PL7 | changes: no intent buttons to disable; a row is disabled with its reason and the menu title says when nothing reaches; never AP debt stays |
| PL8 | stays, with picks: later members are rebuilt from their picks; a pick that no longer fits is dropped (GH5) |
| MM1 | changes: the toggle retires; its refusals (another intent, another target) are pick refusals (IF6 a, b) |
| MM2 | retires as a mode; its uses: skills in any order and repeat, Rest anywhere, items after the opener are picks; the plain move first is Move here; bundled moves (approach, kite) are implied by the pick; the clicked carried-move tile retires (MR4 picks it) |
| MM3 | retires: Fill is the ghost (Auto-fill on); "Manual off rebuilds the auto plan" is Clear |
| MM4 | changes: a delayed opener (Rest picks, then the skill), an out-of-order follow-up, a Snap between two Rests and items as follow-ups are picks; a carried move to a tile PL4 ranks lower retires |
| MV1 | stays; positions read the facing (MR2) |
| MV2 | stays; under `melee_diagonal` `all` a diagonal neighbour is in the melee band (MR10) |
| MV3 | stays; every position tile is outside the front, so an Engage hit on an engaged target is a flank |
| MV4 | changes: no carried-move field; a skill with a `position` dashes up to its range N (`reach_max`, MR1) to that position (MR2-MR4) on the wind-up's tick with its own dash animation (MR8, MR12); a skill without one carries no move; "not the round's move" stays (ET5); Disengage's retreat goes up to N (MR14); `dash_tiles` retires on Maneuver skills (INPUT1-MA) |
| MV5 | changes: one range rule for every skill (the reach band, MR1); a positioned skill is in range when an open position tile is within move cost N in one block (MR4), melee or ranged alike; the hover line changes; Disengage's condition stays |
| MV6 | changes: the destination is MR4's tier and rank; run time re-reads around the target with the chase (MR8); no open tile is `[Flank Protected]`, a miss with its AP (MR8 c, MR9); none in budget is `[Dash Blocked]` as today |
| MV7 | changes: "with no flank tile in range, Engage is not offered" becomes "Flank protected" with no open tile (MR5) and MV9 a's move when open tiles are out of range (MR7); follow-ups move only off a position tile (MR6) |
| MV8 | stays (the hit diagonal per MR10; the retreat within N, the skill's range, MR14); Step here after may follow it (ET4, ET5) |
| MV9 | stays as decided (TQ22 a, MR7); its option (b) retires |
| MV10, MV11 | stay |
| AS1-AS3 | stay; their moves are the round's one regular move (ET5); AS1's chase applies to every skill on a unit target (MR16) |
| K4 (RULESET1 5) | its carried-move clause gives way to MR2-MR4 (a positioned skill ends on a position tile, not the nearest band tile) |
| GD3 | stays; under `melee_diagonal` `all` the counter answers diagonal melee hits (MR10) |
| UI1 | retires (the buttons, menus and toggle); its "loses Riposte" warning moves to the menu rows (IF3) |
| UI2 | stays; ghost spans get GH8's look |
| UI3 | changes: the planned-move lines give way to AR1-AR6; hovering an Engage row marks its positions (MR13) |
| UI4 | stays (the Dojo is out of scope) |

The kit keeps its `manual` flag and the host keeps `round_intent`, `manual` and `fill` (recordings, sims and the Dojo
send them); the battle screen no longer sends them (INPUT1_TASKS 1.3).

## 14. Out of scope

The Dojo's planner controls (UI4); classic input and classic playback; the sims' drivers (`feel`, `timed`, autoplay);
enemy AI and its order lists beyond MR14; controller input for the battle (UI-REVIEW1 Q6 later); drag and drop on
the strip; a block snapshot in the battle's timeline view; VISCLAR1's later slices (badges, forecasts, glyph
placement, the poise bar's place); showing enemies' known movesets (the user's later idea, Q5); halving the AP scale
(a balance option the user keeps open, Q7); balance numbers (INPUT1-MB reports, nothing is tuned here); new art (the
arrow, ghost, pip and tracer looks are drawn; the dash clip is DASH1); the tactics' link to the intents (TACTIC2); classic gap-close and DISENGAGE jumps; mid-battle dialogue and scripted
rounds (ENCOUNTER2-C); the terms table (TERMS1).

## 15. Open questions (the planner asks the user; each has a recommendation)

All answered: Q1-Q9 by "INPUT1 answers", parts 1-3 (user 2026-10-31), Q10-Q19 by part 4 (user 2026-11-01), applied as
listed under the status. The questions and their options are in the decision rows; none is open.
