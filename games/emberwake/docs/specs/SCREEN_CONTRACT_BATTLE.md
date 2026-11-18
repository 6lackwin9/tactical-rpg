# SCREEN_CONTRACT_BATTLE: the battle screen layout contract (AI-21, UX-14..17)

Status: signed off by Tushar 2026-11-18 (battle mock takes 1-16 in the Art/UI thread, take 17 his last fixes); he passed take 16 ("rest all seem
fine") and take 17 applies his last fixes. Owner: Art/UI. Design turns it
into READY rows; Build implements it; after UISPLIT1 lands (PR7-38, PR7-39) the layout part moves into
`games/emberwake/assets/ui/layouts/battle.json`.

Mocks (scripts in `art_src/style/`, outputs in the project files `art-ui/battle_ux14/`): `battle_layout_mock.py`
(takes 1-10), `battle_take11_mock.py` .. `battle_take17_mock.py`. Take 17 (`battle_take17_720p.png`) is the combined
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

- One full plate per hero (up to 4): portrait, name with the round intent tile at the end of the name row (take 17),
  HP bar with current/max ("180/242") and incoming damage as a red loss at the bar's end, AP pips, poise pips along
  the bottom.
- A status column inside the plate on its right edge (26 px), never outside the frame: 4 slots, so up to 4 tabs, or 3
  tabs and "+N" (Tushar, take 17).
- Guests (AI allies): under a teal "Guests" divider, compact 52 px plates with a teal rim, an "AI" tag, HP, intent, and
  a gold "Protect" tag for a protect objective. 4 heroes and 2 guests fit 720p with room to spare.

## 4. Foe panel (right)

- Header: "Hostiles N" and, when a phase brings reinforcements, an amber "+2 round 3" tag.
- Up to 6 foes: a full plate each. From 7: the current target keeps the full plate, the rest go compact (52 px: name,
  HP, poise, up to 2 status tabs). From 11: identical foes group into one row ("Ash Crawler x4", stacked portraits, one
  HP bar each; a click expands the group). The shipped data fields at most 7 foes at once (plus up to 4 from phases).
- Foe plates keep the "when it lands" row (take 17, restored): the blow's kind glyph, its target's face and a 6-block
  ruler with its impact block in red. The status column sits inside the plate on its left edge.

## 5. Foe card (docked, not a hover popup)

- Docked at the top right of the board. While a hero is planning, it shows the hero's current target and stays
  there; with no hero selected, a click on a foe pins it (hover previews it).
- Header: portrait, name, HP bar with current/max, poise bar (5 pips) with the per-pip tag (the pip threshold T, e.g.
  13), then the tags "KO in block 5" (oxblood) and "BREAK in block 4" (amber).
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
  full bar). 5x4 pips at 1:1.
- Order: the soonest to run out first, ties to the one applied first; statuses a planned move adds come last.
- The full list (hover "+N" or press the plate): icon, name, a one-line effect, rounds left as a number ("new" for a
  planned one).

## 7. Skill menu (on a strip block)

- Opens on the block; the foe panel and foe card stay visible beside it. One row per skill: icon, name, AP pips, block
  span, HP and poise damage against the current target, status chips; hover lights the move's blocks on the strip.

## 8. Board

- Ground hazards: tinted tiles with the hazard's icon and rounds left; push or pull: a dashed gold arrow to a dashed
  landing tile on the target.

## 9. Open items (not yet mocked)

- Victory and defeat report (UX-19), the full timeline view (T) and playback controls.
- Not expressed yet: collision stagger from push or pull on the card row, hazard damage in the HP preview and the
  block ruler, reactions (riposte, counters) as a "returns -n" note, flank, guard, cover and windup modifiers in the
  skill hover, channel rounds, unit meters, weapon wear.
- Legibility at 1:1 (take 16): the 64 px status icons blur at 16 px; they need hand-made 16 px versions with simple
  silhouettes (a kit icon batch). Skill icons at 32 px (half the source) read well.
