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
  `#74543c` outer, dark gap, 1 px iron `#4b5563` inner, oxblood `#991b1b` top line, no corners). Typed 2026-11-18
  20:42, "Main current pop-up A".

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

- Every status uses its own kit icon (`kits/rpg/assets/statuses/`); the rim says buff (gold) or debuff (oxblood);
  dashed means it lands from a planned move; a heal is a moss heart (theme moss, not bright green) and previews as a
  pale moss gain on the HP bar.
- Tabs on plates: icon plus the rounds left as up to 3 pips (the shipped data lasts 1 to 3 rounds; a permanent one is a
  full bar). 5x4 pips at 1:1. Superseded by the take 25 proposal below if signed off (user 2026-11-19 18:00: the pips
  read poorly at this size).
- Order: the soonest to run out first, ties to the one applied first; statuses a planned move adds come last.
- Proposed (Art/UI take 24 B and take 25, `STATUSVIS1`, `AI-34`; awaiting the user's sign-off):
  - Tabs carry the icon only, no rounds (user 2026-11-19 18:02); the rounds live in the full list. Take 24 A (a bold
    pixel digit beside the icon) is the alternative.
  - Each status also shows where it acts, by primitive (`STATUSVIS1` section 2), in its kind colour with its icon as
    the key: locked blocks hatched on the unit's timeline lane and the AP row (`block_lock`); the coming tick as a
    pending oxblood (regen: moss) slice on the HP bar with a drip and the amount, and a tick mark at the round's end on
    the lane (`tick`); the changed number with a small down (oxblood) or up (gold) arrow and the base struck through
    (`attack_power`); lost move tiles dimmed at the range edge, gained ones gold-edged (`move_rate`); a shield glyph
    and the cut beside the incoming number (`armor`, `guard`); the taunter's crimson hoop with the foe intents bent to
    it, a stealthed unit greyed as a target (`targeting`); a counter chip in the hit preview (`reactive`);
    `rule_flag` (steadfast) on the plate only.
  - Plate order: `STATUSVIS1` section 3 (priority 1, then 2, then 3; inside a priority the order above). Priority 3
    statuses keep a tab when there is room (`STATUSVIS1` section 4, first item).
  - A locked block at 1:1: a near-black block with 45 degree hatch lines 5 px apart in the status's rim colour and the
    status icon in a framed square on its first block; it is the only patterned fill on the strip, so it never reads
    as a planned move colour (`STATUSVIS1` section 4, second item).
- The full list (hover "+N" or press the plate): icon, name, a one-line effect, rounds left as a number ("new" for a
  planned one).
- Proposed, take 26, awaiting sign-off (`BOSSUI1` BU6, user 2026-11-19 18:09 "the no number look is probably better"):
  tabs on plates and on the boss bar are the icon only, no pips or digits; the rounds left show only in the full list.
  If signed off, this supersedes the pip line above (kept here as history).

## 7. Skill menu (on a strip block)

- Opens on the block; the foe panel and foe card stay visible beside it. One row per skill: icon, name, AP pips, block
  span, HP and poise damage against the current target, status chips; hover lights the move's blocks on the strip.

## 8. Board

- Ground hazards: tinted tiles with the hazard's icon and rounds left; push or pull: a dashed gold arrow to a dashed
  landing tile on the target.

## 9. Open items (not yet mocked)

- Victory and defeat report (UX-19; provisional mock in `screens_ai21_mock.py`). The full timeline view (T) and
  playback controls: mocked as take 23 (`battle_take23_timeline_mock.py`, `[PROVISIONAL]`), awaiting his review.
- Boss fight variant (Tushar 2026-11-18 23:19, low priority, current layout stays for now): a boss bar across the top
  of the board (full name and title, a long HP bar with phase marks, poise, statuses, the next phase or enrage cue);
  the boss leaves the foe panel, mobs keep their plates. Mocked as take 22 (`battle_take22_boss_mock.py`), awaiting
  his review.
- Proposed, take 26 (`battle_take26_boss_mock.py`, `BOSSUI1`, `ENRAGE2` EN5), awaiting sign-off; replaces take 22 and
  24's continuous bar. The boss bar sits in the HUD frame (`hud`, double line), 84 px tall, inset over the board top:
  - Row 1: portrait with the crown; the phase markers (one diamond per phase, at most 4: done filled gold, current
    lit, coming hollow iron) left of the title; the title is the phase's name or form, else the unit's name and
    title; the boss passive tags right-aligned (one per passive, display name; hover or focus shows its description
    in a HUD-frame tip). No phase-change cue and no enrage tag.
  - Row 2: the current phase's HP span as a full bar with the planned impact; the number beside it is the span left
    ("160/450"). It refills when the next phase starts; a phase that brings a different boss unit swaps the bar to
    that unit (portrait, name, HP, poise, passives).
  - Row 3: poise as the unit's `poise_pips` (10 for bosses, about 9 x 5 px each), the per-pip tag, icon-only status
    tabs, then the intent and its impact as in take 24.
  - Phase change (inside the existing phase transition, about 0.9 s): the span drains to 0; a 0.1 s gold flash on the
    bar while the marker fills and the new tags and statuses swap in; the portrait (new unit only) and title crossfade
    (0.3 s); the bar refills left to right (0.4 s).
  - Rejected option B: a segment strip under the bar instead of the diamonds (reads as a second HP bar).
  - Enrage (`ENRAGE2`): the round counter in the top bar warns a round ahead ("Enrage next round", amber, under the
    objective) and from the enrage round reads "Foes enraged +n"; an enraged unit's plate takes a 10 % amber tint and
    rim with an "up +10%" chip by its name, and its numbers show raised (`STATUSVIS1` attack_power row).
- Proposed, take 27 (`battle_take27_boss_mock.py`, user 2026-11-19 18:24 on take 26, `BOSSUI1` section 5), awaiting
  sign-off; amends take 26:
  - Phase markers flipped: every phase still to fight is lit gold from the start, the current one brightest with a
    ring, a beaten one dims to iron.
  - Poise: the pips in groups of 5 with a gap, the count beside them ("6/10"), then the per-pip tag.
  - Boss intent: while a hero targets the boss or the boss is pinned, the bar grows by two rows (58 px at 720p) in
    place of the foe card. "Hits": every planned hit on the boss by block as a face chip with HP and poise pips, the
    selected hero's ringed gold. "After": the phase span and poise left after each block, BREAK where it lands. At
    the right: the selected hero's total, the party total and what is left to the next phase.
  - Clear of units: the foe card docks at the board's top right unless it would cover the acting unit or its target
    at 720p; then it docks top left (`battle_take27_card_clear_720p.png`). The grown boss bar ends about 60 px above
    the board's first unit row in the shot; Build checks the same rule against the live board.
  - Carry (`BOSSUI1` BU14): a hit past the span shows in "After" as the span at 0, the next phase's marker, then the
    next span less the remainder (gold number); the bar's impact fills the span and its number gets the marker; the
    totals read "+6 to ph. 2" (`battle_take27_carry_2x.png`). A phase with a new boss unit takes no carry.
- Proposed, take 28 (`battle_take28_boss_mock.py`, user 2026-11-19 18:34: take 27's intent "too busy"; markers and
  clearance approved), awaiting sign-off; replaces take 27's "Hits" and "After" rows: two rows of damage by block
  under "Block 1".."Block 6" headers. Row 1: the selected hero (face, name), their damage per block in a gold-ringed
  cell, "-" where they do not hit. Row 2: "Party", the whole party's damage per block, the broken-poise glyph with
  "BREAK" in the block where the boss breaks, the next phase's diamond with "Ph. 2" where the span runs out. Right:
  "Total" with each row's sum and what is left to (or carried into) the next phase. No per-member chips.
- Proposed, take 29 (`battle_take29_boss_mock.py`, user 2026-11-19 18:38, `BOSSUI1` BU15 and BU16), awaiting
  sign-off; amends take 28: each cell shows the block's impact as the small foe intent does (HP, a poise glyph per
  pip taken, a 12 px chip per status landed), no skill names. Phase break: the block where the span reaches 0 is
  ringed gold with the next phase's diamond; the blocks after it are hatched and their numbers struck (they do not
  run, no AP spent); totals count only what lands and read "Ends block N" and "Ph. 2 next round". The take 27 carry
  line above is superseded by BU16 if this is signed off.
- Proposed, take 30 (`battle_take30_boss_mock.py`, user 2026-11-19 18:43), awaiting sign-off; amends take 29:
  boss poise without the 5 + 5 split, recommended A: one 80 px bar with a faint tick per pip and the count ("6/10"),
  read like the plate AP bar (B: ten even pips with the count); a cell writes poise as one shield glyph and the pips
  taken as a number ("2"), and shows at most one status chip and "+N"; party row, recommended P1: the whole party in
  one row, hover or press it for one line per member in the same block columns (P2: the selected hero and "Others",
  the rest summed, with the party total beside the totals).
  - Picked on the user's card 2026-11-19 18:56: A (poise bar with ticks and the count) and P1 (one party row, hover
    by member); a UI preference, revisit at the UI review.
  - Then (user typed 2026-11-19 18:57, supersedes the P1 half of the card): A for poise; the bar shows P2 (the selected
    hero and "Others", no duplication) and the hover lists every member (`battle_take30_pick_others_hover_2x.png`).
  - User 2026-11-19 18:59: the Total column holds only the two row sums; no "Party -N" (the boss bar's planned loss
    says it) and no "N to phase 2" (the HP bar shows it). The phase break frame keeps "Ends block N" and "Ph. 2 next
    round".
  - User 2026-11-19 19:00: the Total column shows each row's full impact over the blocks that run: HP, poise (glyph and
    number) and the status chips (one and "+N" in the bar). The hover adds a Total column per member (every chip)
    and a "Party" line under the members with the per-block and total impact (`battle_take30_totals_hover_2x.png`).
- Proposed, take 31 (`art_src/style/battle_take31_boss_mock.py`, user 2026-11-19 19:02 and 19:03):
  - Fixed slots in every impact cell, so nothing shifts with the numbers: HP (5 mono characters, right aligned),
    poise (the glyph and 2 digits), statuses (one chip and "+N" in a block cell, three in the bar's Total, four in the
    hover's Total). An empty slot stays empty; a block with no hit shows "-" in the HP slot.
  - Numbers past 999 use K (1.2K, 12K, 120K) and past 999K use M, so HP never needs more than 4 characters and the
    minus.
  - The intent rows show only while planning (a hero selected, plan not confirmed). In resolution and the enemy turn
    they fold away and the boss bar is 84 px; planning adds 58 px (142 px of 720).
  - Phase break: the breaking block's header reads "Block N: ends" and the first cancelled block's header reads
    "Ph. 2 next round"; the totals keep their slots (`battle_take31_phase_break_2x.png`,
    `battle_take31_plan_vs_resolve.png`, `battle_take31_stress_hover_2x.png`).
  - User 2026-11-19 19:05: agreed: outside planning only the boss HP, poise and statuses matter; in planning the
    selected hero's row and the Others row, with the hover as the optional detail. Final feel to be checked in play.
- Proposed, take 32 (`art_src/style/battle_take32_boss_mock.py`, user 2026-11-19 19:08), replacing take 31's slots
  and K rule:
  - Every impact is laid on hidden cells: HP (36 px, right aligned), poise (glyph at a fixed x, number after it),
    one 17 px cell per status chip, then a "+N" cell of its own, so "+N" never overlaps a chip. Block cells have one
    chip cell; the Totals (bar and hover) three, with wider gaps. Nothing moves with the numbers
    (`battle_take32_cells_guide_2x.png` draws the cells).
  - Numbers stay whole up to 9999 (1250 sits beside 233); K starts at 10K, M at 1M.
- Not expressed yet: collision stagger from push or pull on the card row, hazard damage in the HP preview and the
  block ruler, reactions (riposte, counters) as a "returns -n" note, flank, guard, cover and windup modifiers in the
  skill hover, channel rounds, unit meters, weapon wear.
- Legibility at 1:1 (take 16): the 64 px status icons blur at 16 px; they need hand-made 16 px versions with simple
  silhouettes (a kit icon batch). Skill icons at 32 px (half the source) read well.
