# SCREEN_CONTRACT_BATTLE: the battle screen layout contract (AI-21, UX-14..17)

Status: signed off by Tushar 2026-11-18 (battle mock takes 1-16 in the Art/UI thread, take 17 his last fixes); he passed take 16 ("rest all seem
fine") and take 17 applies his last fixes. Sections 3-5 updated for takes 18-21 (plate re-layout, panels 184 px, AP
bar, short names on plates), signed off 23:14. Owner: Art/UI. Design turns it
into READY rows; Build implements it; after UISPLIT1 lands (PR7-38, PR7-39) the layout part moves into
`games/emberwake/assets/ui/layouts/battle.json`.

Mocks (scripts in `art_src/style/`, outputs in the project files `art-ui/battle_ux14/`): `battle_layout_mock.py`
(takes 1-10), `battle_take11_mock.py` .. `battle_take21_mock.py`. Take 17 (`battle_take17_720p.png`) is the combined
screen at 1280x720, 1:1.

## 1. Frames

- The battle screen's main panels keep the current ornate frame; popups, cards and the HUD use frame A (1 px bronze
  outer `frame_a_outer`, dark gap, 1 px iron inner `frame_a_inner`, oxblood top line `frame_a_top`, no corners; theme
  game colours). Typed 2026-11-18 20:42, "Main current pop-up A".

## 2. Top bar

- Left: the selected hero's portrait, then "Round 1 of 8" in Alegreya SC gold (text, not pips).
  The objective sits under it in two short lines: the goal ("Rout them", "Survive 6 rounds", "Reach
  the exit") and a protect target in gold ("Protect Hollis") when the encounter has one.
- Strip: one tile per planned move, sized by its blocks (1 to 6), with a block ruler under the strip. A tile is filled
  in its skill kind's colour (the game's `kind_color`: attack red, move blue, defend teal, support green, focus ochre,
  rest grey, status purple); the selected tile has a 2 px gold rim. Skill icon centred; AP pips under the icon, as
  many as the move costs (no cap at 4).
- Ghost (the planner's suggestion): the same tile in its kind colour faded (`GHOST_ALPHA` 0.45, icon at alpha 150),
  hollow AP pips, no dashed outline. A pick turns its tile solid and the ghost refills the blocks around it. Undo (Z or
  right click) removes the last pick; Clear wipes the selected hero's picks and brings the ghost back.
- Impact: the block ruler under the strip turns red on each block where a foe's blow lands on the selected hero (only
  the thin ruler bar, not the tile; take 17).
- Effect chips ride on the tile's top edge (centred, half above the tile) so a 1-block tile's icon stays clear: a moss
  heart for a heal, a status chip for each status the move applies.
- AP line beside the strip: a row of 12 pips (the cap) where planned pips are amber, the rest of the start AP lit, the
  missing ones dim stubs; under it "AP 10/12 -6", and "(-n)" in grey for the ghost's cost.
- Controls: a tactic button for the party stance and one for the hero stance (portrait plus a pennant), Auto as a play
  button with an "AUTO" tag, Clear, then Confirm as the one amber button.

## 3. Party panel (left)

- Both side panels are 184 px wide (plates 176 px; Tushar, take 20, 2026-11-18 23:14: "for the enemies, I think we
  should go for 184 ... both of them are aligned"); the board view between them is 888 px at 1280x720.
- One full plate per hero (up to 4), 84 px tall: portrait 48 px; the name with the round intent tile (18 px) at the end
  of the same row; the HP bar with current/max ("180/242") and incoming damage as a red loss at the bar's end; the AP
  bar; poise pips along the bottom (takes 18 A, 20).
- Names: body font, the small font when the name does not fit beside the intent tile (Mara Voss), then cut with "…"
  (only test units). Every story hero in the data fits.
- AP (take 21; Tushar 23:11 and 23:14, "The amber looks better. It aligns with the timeline pips as well"): a
  continuous bar like HP, the cap (12) as its full width, the start AP lit in AP blue and the planned part in amber
  (`PLANNED`, the strip's planned pip colour); beside it the balance after the plan as "4/12" (10 AP, plan 6). At 0 the
  number turns grey. The plate shows the balance, the strip's AP line shows how it is spent; no team AP gauge.
- A status column inside the plate on its right edge (26 px), never outside the frame: 4 slots, so up to 4 tabs, or 3
  tabs and "+N" (Tushar, take 17).
- Guests (AI allies): under a teal "Guests" divider, compact 52 px plates with a teal rim, an "AI" tag, HP, intent, and
  a gold "Protect" tag for a protect objective. 4 heroes and 2 guests fit 720p with room to spare.

## 4. Foe panel (right)

- Header: "Hostiles N" and, when a phase brings reinforcements, an amber "+2 round 3" tag.
- Up to 6 foes: a full plate each. From 7: the current target keeps the full plate, the rest go compact (52 px: name,
  HP, poise, up to 2 status tabs). From 11: identical foes group into one row ("Ash Crawler x4", stacked portraits, one
  HP bar each; a click expands the group). The shipped data fields at most 7 foes at once (plus up to 4 from phases).
- Foe plates (176 px, take 20): the round intent as a 16 px badge on the portrait's lower right corner (not on the
  name row); the unit's short name on the name row; HP bar and current/max; the "when it lands" row (take 17): the
  blow's kind glyph, its target's face and a 6-block ruler with its impact block in red; poise pips along the bottom.
  The status column sits inside the plate on its left edge.
- A boss plate: a gold crown on the portrait's top left, the name in gold and a gold outer rim around the plate.
- Plate names are the short name; the full name and title live in the foe card (5). Tushar 23:14: "in the hostile tab
  or like the enemy tab, it just shows you the shortened version of the name". The name room is 93 px (small font);
  six shipped unit names are longer (Goblin Skirmisher, Goblin Berserker, Castellan Footman, Watch Crossbowman,
  Overseer Grendel, Mortuary Flagellant), so units need a short plate name that fits (e.g. "Crossbowman", "Grendel",
  "Flagellant"; bosses already have one, `CAST1_BOSSES.md` section 1); anything still too long is cut with "…".

## 5. Foe card (docked, not a hover popup)

- Docked at the top right of the board. While a hero is planning, it shows the hero's current target and stays
  there; with no hero selected, a click on a foe pins it (hover previews it).
- Header: portrait, the full name in Alegreya SC gold (the character's `name`, e.g. "Inquisitor-Prelate Nemeth") and,
  when it has one, the title on its own line under it in the small font (the character's `title`, e.g. "Chieftain of
  the Horde"; take 20); a boss gets the crown on its portrait. Then the HP bar with current/max, poise bar (5 pips)
  with the per-pip tag (the pip threshold T, e.g. 13), then the tags "KO in block 5" (oxblood) and "BREAK in block 4" (amber).
- Hero selected: one row per move of that hero on this foe: skill icon, a 6-block mini ruler with the move's blocks lit,
  HP damage, the poise area, status chips. The HP and poise bars in the header preview the whole party's plan: this
  hero's share bright, the other allies' dark. Footer: the hero's total and the party's total with the KO block.
- No hero selected: one row per block (at most 6): the block badge, the faces of the allies hitting in it, the summed
  HP damage, the poise area (pip marks per hit), the status chips, "KO" on the block it falls in. Footer: the total.
- Poise area: fixed width, 3 slots (the per-hit cap). The pips each hit breaks, in amber. Poise does not pool: each
  hit settles on its own (`rpgkit_battle::poise::pip_loss`, `floor(D / T)` pips, at most 3, the rest lost), so the card
  shows pips, never summed poise numbers. The hit that empties the bar shows a split-shield icon in place of its pips;
  hits on an already broken foe show it dimmed. The header's poise glyph turns into the split shield only when the plan
  empties the bar; until then it stays blue, with the pips the plan breaks in amber.
- Status chips go right after the damage: at most 3, then "+N" (the hero view and the unit's list show them all).
- Hits after the forecast KO keep their normal look with a soft "after KO" outline tag: the forecast ignores crits,
  moves and enemy reactions, so they stay valid moves.
- Crits: the numbers are base damage (no ranges, no averages). The KO tag gains "(star N)" when the chance that the foe
  is down by block N, over every crit outcome of the planned hits (each crits on its own at its hero's crit chance, x1.5
  `CRIT_MULT`), reaches 50% before the base KO block; otherwise no star. Crit damage and chance show on the skill's
  hover in the skill menu.

## 6. Statuses

Signed off 2026-11-19 (takes 33 to 36, `STATUSVIS1` SV5 to SV15; mocks `art_src/style/battle_take33_status_mock.py`
to `battle_take36_locks_mock.py`, shots in the project files `art-ui/battle_ux14/`).

- Icons: every status uses its own kit icon (`kits/rpg/assets/statuses/`); the rim says buff (gold) or debuff
  (oxblood); dashed means it lands from a planned move; a heal is a moss heart (theme moss, not bright green) and
  previews as a pale moss gain on the HP bar. The legacy status icons are reworked later (`AI-24`).
- Tabs on plates and on the boss bar: the icon only, no pips and no digits (user 2026-11-19 18:02 and 18:09); the
  rounds left show only in the full list.
- Order: `STATUSVIS1` section 3 (priority 1, then 2, then 3); inside a priority the soonest to run out first, ties to
  the one applied first; statuses a planned move adds come last. Priority 3 statuses keep a tab when there is room.
  Steadfast (a `rule_flag`) shows on the plate only and stays first in the column.
- The full list (hover "+N" or press the plate): icon, name, a one-line effect, rounds left as a number ("new" for a
  planned one).
- Where each status acts (`STATUSVIS1` section 2), in its kind colour with its icon as the key:
  - Locks (`block_lock`, SV15): grey blocks with the status icon, built like the other tiles, on the top strip (a
    grey tile with the icon in the skill-icon frame and a grey bar under it) and in the full timeline view (grey tint,
    band and rim). Stun is one continuous grey block over its locked blocks (blocks 1 and 2); stagger is two separate
    one-block grey blocks (blocks 1 and 4). No hatch. AP follows the rules (SV11): a locked block leaves whatever AP
    the rules give, the display changes none.
  - One HP loss (SV6): hits and ticks are one planned-loss slice on the HP bar and one number; hovering the number
    lists each hit and each tick with the total. Regen previews as the moss gain.
  - Numbers after modifiers (SV7, SV13): every deterministic number (damage, heal) is the final number only, tinted
    when a modifier changed it (lowered in the theme's `foe` colour with a small down arrow, raised in the game colour
    `boss_gold` with a small up arrow; theme tokens, not hexes, `FB-370`). A hovered number gets no frame; its hover lists the base and each modifier's percent. No struck
    base, no shield glyph.
  - Movement (`move_rate`): lost move tiles dimmed at the range edge, gained ones gold-edged.
  - Taunt (SV8): a 20 px badge over the taunted unit's sprite (dark slate, crimson rim, the taunt icon, a pointer
    down), bobbing 1 px; hover names the unit it must target and the rounds left. No ring at the feet. A stealthed
    unit greys as a target.
  - Riposte (SV9): the counter chip in the hit preview reads "Riposte" with no number; hover gives the likely return
    on the selected hero.
- Loadout passives (SV10, SV12, SV13): two dedicated 18 px sockets centred under the plate's portrait (a loadout holds
  at most 2; an empty one stays a dim socket), each with its own hover (the passive's name and what it does). The icon
  is the passive's kit archetype icon (`kits/rpg/assets/passives/<archetype>.png`, `AI-39`). Passives never enter the
  status column, which is for statuses that last a few rounds.
- History (superseded, kept for the record): the 3-pip rounds on tabs (take 16), the bold digit (take 24 A), the hatched
  lock block (take 25, take 33), take 25's pending tick slice beside the hit, the struck base with the arrow, the shield
  glyph beside the incoming number, the taunter's hoop at the feet and the "always on" passives box (take 33).

## 7. Skill menu (on a strip block)

- Opens on the block; the foe panel and foe card stay visible beside it. One row per skill: icon, name, AP pips, block
  span, HP and poise damage against the current target, status chips; hover lights the move's blocks on the strip.
- Clear of the foe side (`FB-364`, BU12): the menu never covers the docked foe card, the foe panel or the selected
  target. Code keeps it clear: a menu opened beside a board tile flips side, or opens above or below the tile;
  `battle.json` may give the preferred side, and code does the avoiding (built with `PR7-54`).

## 8. Board

- Ground hazards: tinted tiles with the hazard's icon and rounds left; push or pull: a dashed gold arrow to a dashed
  landing tile on the target.

## 9. Boss bar and full timeline view

Signed off 2026-11-19: the boss UI from takes 26 to 32 (`BOSSUI1` BU9 to BU17, `ENRAGE2` EN5; mocks
`art_src/style/battle_take26_boss_mock.py` to `battle_take32_boss_mock.py`), the full timeline view from takes 34 to
36 (`STATUSVIS1` SV14, SV15). Shots in the project files `art-ui/battle_ux14/`.

### 9.1 Boss bar

- Who: a boss leaves the foe panel and takes the bar across the top of the board; mobs keep their plates. The bar sits
  in the HUD frame (`hud`, double line), inset over the board top: 84 px tall, plus 58 px of intent rows while
  planning (142 px of 720).
- Row 1: the portrait with the crown; the phase markers left of the title, one diamond per phase (at most 4): every
  phase still to fight lit gold, the current one brightest with a ring, a beaten one dimmed to iron; the title is the
  phase's name or form, else the unit's name and title; the boss passive tags right-aligned (one per passive, its
  display name; hover or focus shows its description in a HUD-frame tip). No phase-change cue and no enrage tag.
- Row 2: the current phase's HP span as a full bar with the planned impact; the number beside it is the span left
  ("160/450"). It refills when the next phase starts; a phase that brings a different boss unit swaps the bar to that
  unit (portrait, name, HP, poise, passives).
- Row 3: poise as one 80 px bar with a faint tick per pip and the count beside it ("6/10"), read like the plate AP bar
  (take 30 A, user 18:56; 10 pips for a boss, BU9 and BU10); the per-pip tag; icon-only status tabs; then the boss's
  own intent and its impact as on a foe card.
- Phase change, inside the existing phase transition (about 0.9 s): the span drains to 0; a 0.1 s gold flash on the
  bar while the marker dims and the new tags and statuses swap in; the portrait (new unit only) and title crossfade
  (0.3 s); the bar refills left to right (0.4 s). The transition is the place for a phase cutscene or dialogue (BU17).
- Enrage (`ENRAGE2`, BU13): the round counter in the top bar warns a round ahead ("Enrage next round", amber, under
  the objective) and from the enrage round reads "Foes enraged +n %"; an enraged unit's plate takes a 10 % amber tint
  and rim with an "up +10%" chip by its name, and its numbers show raised (section 6, numbers after modifiers).

### 9.2 Boss intent rows (planning only)

- When: while planning (a hero selected, plan not confirmed) and a hero targets the boss or the boss is pinned, the
  bar grows by the two intent rows in place of the docked foe card. In resolution and the enemy turn they fold away
  and the bar is 84 px (take 31; user 19:05: outside planning only the boss HP, poise and statuses matter).
- Layout: "Block 1" to "Block 6" headers over two rows (BU11, take 30 P2, user 18:57). Row 1: the selected hero (face,
  name) and their impact per block in a gold-ringed cell. Row 2: "Others", the rest of the party summed per block (no
  duplication of the selected hero). Hover or press the rows: one line per member in the same block columns, each with
  a Total column holding every chip, and a "Party" line under the members with the per-block and total impact.
- A cell (BU16): the block's impact as the small foe intent shows it: HP damage, poise as one shield glyph and the pips
  taken as a number, one 12 px status chip and "+N"; no skill names or icons (the strip above has them); "-" in the HP
  cell where the row does not hit.
- Fixed cells (take 32): every impact sits on hidden cells so nothing moves with the numbers: HP 36 px right aligned,
  poise (the glyph at a fixed x, the number after it), one 17 px cell per status chip, then a "+N" cell of its own.
  Block cells have one chip cell; the Totals (bar and hover) three, with wider gaps (`battle_take32_cells_guide_2x.png`).
- Numbers: whole up to 9999 (1250 sits beside 233); K from 10K (12K, 120K), M from 1M.
- Totals, at the right: each row's full impact over the blocks that run (HP, poise glyph and number, status chips).
  No "Party -N" (the bar's planned loss says it) and no "N to phase 2" (the HP bar shows it).
- Phase break (BU15, BU16): a hit that takes the span to 0 ends the round. The breaking block's header reads "Block N:
  ends" and is ringed gold with the next phase's diamond; the first cancelled block's header reads "Ph. 2 next round";
  the blocks after it grey out and their numbers are struck (they do not run, no AP spent); totals count only what
  lands. No carry into the next phase. BREAK with the broken-poise glyph marks the block where the boss's poise breaks.
- Clearance (BU12): the foe card, the boss intent rows and any intent box sit clear of the acting unit and its target.
  The foe card docks at the board's top right unless it would cover the acting unit or its target at 720p, then top
  left; the grown boss bar ends clear of the board's first unit row (Build checks it on the live board).
- Open after the playtest (user 19:15): the numbers may be too many. Fallback if play shows redundancy: drop the
  per-block cells and keep only the totals (the selected hero, Others and the party breakdown), leaning on the skill
  strip above for timing.
- History (superseded): take 22's continuous bar; option B's segment strip under the bar (read as a second HP bar);
  take 27's "Hits" and "After" rows, face chips and the carry into the next phase (BU14, replaced by BU15); the 5 + 5
  poise pips (take 27, replaced by take 30 A); P1's one party row; take 31's 5-character slots and K from 1000.

### 9.3 Full timeline view (T)

- Purpose (for the player, not a log): T opens the whole round at once, every unit's moves by block, to check the plan
  before Confirm and to follow playback (take 23 layout, `battle_take23_timeline_mock.py`).
- Rows: one per unit. A 150 px name column: face, name, a small arrow and the face of the unit's one target for the
  round (a unit keeps one target, SV14). An enemy target in a crimson frame; an ally (Support) or self in a teal frame
  with a teal arrow; an area skill's main target with a small amber ring on the face's corner; a ground tile (field
  spells) as a 3 x 3 floor patch with the target tile lit (hover rings that tile on the board); Maneuver, Rest or no
  target a dim dash. No per-blow ticks or target faces.
- Tiles (take 34): a move tile is a dark tint of its kind colour (30 %) with a 3 px kind band on top and a kind rim,
  the skill icon 24 px. Locked blocks are grey tiles with the status icon (section 6, SV15).
- User 2026-11-19 19:48: signed off (far less busy; the tile glyph with the board highlight on hover, the area mark and
  the teal ally frame all work). Playback controls stay as take 23.

## 10. Open items (not yet mocked)

- Victory and defeat report (UX-19; provisional mock in `screens_ai21_mock.py`).
- Not expressed yet: collision stagger from push or pull on the card row, hazard damage in the HP preview and the
  block ruler, reactions (riposte, counters) as a "returns -n" note, flank, guard, cover and windup modifiers in the
  skill hover, channel rounds, unit meters, weapon wear.
- Legibility at 1:1 (take 16): the 64 px status icons blur at 16 px; they need hand-made 16 px versions with simple
  silhouettes (`AI-24`, widened to a rework of the legacy status icons). Skill icons at 32 px (half the source) read
  well.
