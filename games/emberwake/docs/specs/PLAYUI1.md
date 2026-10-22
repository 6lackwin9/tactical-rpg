# PLAYUI1: play screens from the playtest of 2026-10-22 (saves, region map, dialogue, shop, note editor)

Status: approved (user 2026-10-22, section 9 answered; see "9b. Answers"). Was: proposed (spec 2026-10-22; the user
answers section 9 before any build)
Depends on: see each build row (section 8); nothing here blocks a row already approved
Lane: logic (engine kit, save locks, text editor, game screens, one shop rule), asset (icons), design (PROPS1 spec)
Touches: `engine/crates/aegis_ui/src/{ui,shell,specimens,notes}.rs`, `engine/crates/aegis_save` (slot locks),
`games/emberwake/crates/emberwake_gui/src/{menus,region,worldmap,hub}.rs`, a new `emberwake_gui/src/shop.rs`,
`games/emberwake/crates/emberwake_world/src/{shop,explore}.rs`, icon files in `engine/assets/ui/` and
`kits/rpg/assets/ui/`

Sources: recording `games/emberwake/playtests/inbox/20261021-193519/` (5 notes, all `ui`, screenshots `note_1.jpg` to
`note_5.jpg`; notes 1 to 4 in the UI Gallery, note 5 in the Playtests browser); preferences `ui.md` ("Navigation and
popups", "Maps", "Text size"); `UI_REVIEW1.md` (UIR-2, UIR-6), `MAPREAD1.md` (Q1), `DESIGN_REVIEW2.md` 6 (MAPNAV1),
`DRAG1.md` (C4); the screens as they are on 2026-10-22 (`menus.rs` `Slots`, `region.rs`, `aegis_ui::notes`).

## Goal

Make the five play screens the user tried read and work like a finished game: a save slot is one click target with
lock and delete icons; the region map is a full-width map with a collapsible icon legend, service icons on NPCs, names
on hover, sprites and props; the conversation is a smaller centred box that keeps both speakers in view; the shop is a
three-column trade (shop, trade, stash) settled by one Confirm; the playtest note box is a real text editor.
Coordinates below are logical pixels (the 640x360 canvas, 1280x720 window at 2x) at text size Normal; every screen
grows with the text size (TEXTFIT1 rule) and is checked with the agent link's `text_audit` at all four sizes.

## 0. Layers and rows that already cover parts

| Piece | Layer | Covered by (approved row) | This spec adds |
| --- | --- | --- | --- |
| Icon button, collapsible section header | engine (kit) | none | PLAYUI1-K |
| Text editor (up / down, selection, clipboard) | engine (kit) | none | PLAYUI1-T |
| Save slot locks | engine (`aegis_save`) + game | none | PLAYUI1-S |
| Legend collapse toggle, legend row click, edge arrows | game | MAPNAV1-U (collapse is preference "Maps") | icons, floating pane (fold) |
| Log as its own pane | game | MAPLOG1-U (ticker bottom left, L expands) | placement with the dialogue box (fold) |
| Quest badges `!` and `?` on NPCs | game | MAPREAD1 (done), QUESTTRACK1-U | shop and recruit icons |
| Party marker as a character image | game | MAPNAV1-U ("Bryn's token") | the leader's sprite instead (PLAYUI1-R) |
| Character sprites | kit + game art | SPRITE1-A2, SPRITE1-L (battle), SPRITE2 (NPC bodies) | region map use (PLAYUI1-R) |
| Skill and item icons in the shop rows | game | ICONUI1 (idea) | nothing; the shop rows leave a 12 px icon slot |
| Shop drag and drop | game | DRAG1 candidate C4 | replaced by clicks (section 4, Q-list) |
| Controller in dialogue and shop | engine + game | UIR-7 | every new control is focusable |

## 1. Save slots (note 1)

Today (`menus.rs` `Slots`, shot `note_1.jpg`): six rows (Autosave, Slot 1-5), each a card with text and buttons at the
right (Save here or Load, Delete); the card itself does nothing; no lock exists anywhere.

Request: select a save by clicking anywhere on the slot; two icon buttons, lock and delete, in the slot's upper right.

Options:

- A (recommended): the whole row is the button (save mode: Save here; load mode: Load), lock and delete are icon
  buttons in the row's top right corner; the lock lives in a sidecar file next to the saves, enforced by `aegis_save`.
- B: as A, but the lock is a field in the save envelope. It travels with an exported save, but toggling it rewrites the
  save file and the save format and its fixtures change. Not recommended.
- C: row click only, no lock (delete stays a button). Cheapest; drops half the note.

Rules (option A):

1. Row click: save mode on a manual slot = today's Save here (empty slot saves at once; a filled one asks the existing
   overwrite question); load mode on a readable save = today's Load question. The row is not clickable when the action
   does not apply (autosave in save mode, an empty or unreadable slot in load mode, a locked slot in save mode); its
   tooltip says why ("The game writes the autosave.", "Empty slot.", "Locked: unlock it to save over it.").
2. A clickable row has the hover and focus border of a dialogue choice (GOLD); Enter or Space on a focused row clicks it.
3. Icons, 12x12 inside a 16x16 hit box, 4 px from the row's top and right edges, right to left: delete, lock. Delete
   shows on any existing save (today's question); lock shows on existing manual saves only (never on the autosave).
   A click on an icon never triggers the row. Tooltips: "Lock (protect from overwrite and delete)", "Unlock", "Delete".
4. A locked save: lock icon closed and GOLD; delete icon disabled ("Unlock the save to delete it"); row click in save
   mode disabled (rule 1); load works as usual.
5. Engine (`aegis_save`): `is_locked(dir, slot) -> bool` and `set_locked(dir, slot, on) -> Result`. Storage
   `<save dir>/locks.json` `{"locked": [<slot>, ...]}` (sorted, unique, manual slots only). A slot counts as locked
   only while its save file exists (a stale entry is dropped on the next `set_locked`). A missing or unreadable
   `locks.json` means nothing is locked. `write` and `delete` on a locked slot fail with `Slot <n> is locked.`;
   `set_locked` on the autosave slot fails with `The autosave can't be locked.`. Save file import (SAVE-IO, not yet in
   the engine) must refuse a locked slot when it lands.

Acceptance: an `aegis_save` test writes, locks, fails a write and a delete, unlocks, writes; locks survive a restart
(read back from disk); over the agent link: `slot.2` in save mode on an empty slot saves, on a filled slot asks;
`slot.2.lock` then `slot.2` does nothing and the tip names the lock; `slot.0` has no `.lock` widget; text audit clean at
four sizes.

## 2. Region map (note 2)

Today (`note_2.jpg`): top bar with the name, supplies and a help line; the map left of a 138 px right column
(`SIDE_X` 502) holding the "Here" list (coloured squares) and the event log under it; markers are circles with a
portrait or letter; NPC and exit names float above their tiles at all times (MAPREAD1 Q1 A); tiles only.

### 2.1 Legend and log

- A (recommended): the map takes the full width; the legend is a floating pane at the map's top right under the HUD
  bar, header "Here (N)" with a chevron, collapsed to the header by a click (state kept in settings as
  `map_legend_open`, default true); each row shows the marker's own map icon (12x12) instead of a box, then the label,
  then the NPC's service icons (2.2) right-aligned; spent markers dim. The log is MAPLOG1-U's ticker at the map's
  bottom left (L expands it to the last 12), hidden while the dialogue box is open. Width from text widths, at most
  170 px.
- B: keep the right column, split into two collapsible panes (Here on top, Log under it). Less map room; no overlap.
- C: the right column with two tabs (Here, Log). One pane at a time; the log is hidden while you read the legend.

Recommendation A: it is what MAPLOG1-U (approved) and the legend collapse (MAPNAV1-U, approved) already point to, and
the map gains 138 px. A legend row click keeps today's behaviour; MAPNAV1-U's "row click pans" rule stays in MAPNAV1-U.

### 2.2 NPC service icons

- A (recommended): derived from what the NPC offers now, no data: `turn_in` (a ready quest turns in here), `offer` (an
  available quest starts here), `shop` (keeper of an open shop), `recruit` (a choice in the NPC's dialogue tree with a
  `recruit` effect for a character not in the party, its `requires_flags` holding). The map badge shows the first of
  that order (turn_in and offer reuse today's `ready` and `quest` badges); the legend row and the hover tip show all
  ("Shop, Recruits").
- B: an authored `services` list on characters (shop, recruit, tavern, trainer, quest). Allows roles with no system
  behind them yet; data can drift from what the NPC really offers.
- C: A plus an optional authored override for flavour roles. Most flexible, most work.

Rule (A): a new read-only `explore::npc_services(d, gs, character_qid) -> Vec<&'static str>`; `marker_view` is NOT
changed (the journey parity compares its output against Godot). There is no tavern, inn or trainer service in the game
today, so no icon for them (Q4).

### 2.3 Names on hover

MAPREAD1 Q1 A (approved 2026-10-18) chose NPC and exit names always shown "because they are what players look for
first". The user now asks for names on hover, maybe as a setting. What changed: the legend (2.1) and the service icons
(2.2) now answer "who is here" without labels, so the original reason holds less; the change supersedes that answer
(HISTORY row, section 7).

- A (recommended): a Gameplay setting "Map names: On hover / Always" (`map_names`: `hover` | `always`), default `hover`
  (Q3). On hover: no floating labels; hovering a marker or its legend row, or facing it from an adjacent tile, shows
  `label` above the marker (as the hover line does today). Always: today's labels.
- B: hover only, no setting. C: keep always (no change).

### 2.4 Character sprites instead of tokens

Covered for battle by SPRITE1-U and for NPC art by SPRITE2. New: PLAYUI1-R draws the party leader (first active member)
and every NPC marker with its body sheet: `walk` clip while moving, `idle` otherwise, flipped horizontally by the last
horizontal step (an up or down step keeps the facing); an NPC turns to face the party while talking. The 48 px sprite
stands on its tile, bottom-centred, 1.5 tiles tall (24 logical px on a 16 px tile), drawn in row order so lower rows
cover upper ones. Fallback: the character's body (SPRITE2), then its class body, then today's token. This replaces
MAPNAV1-U's "Bryn's token as the party marker".

### 2.5 Buildings and props

- A (recommended): a props layer on the map templates (`data/maps/maps.json`): a list of `{id, art, x, y, w, h,
  solid}` drawn over the tiles and under the characters; collision stays in `rows` (a solid prop's footprint must lie on
  wall tiles, a validation error otherwise), so saves, anchors and the journey parity are untouched. Art: generic props
  in the kit (`kits/rpg/assets/props/<id>/v<n>.png`: house, tent, well, crates, barrels, brazier, banner, tree, rock,
  fence), world props in the game (`games/emberwake/assets/props/ashlands/<id>/v<n>.png`: basalt tower, forge,
  palisade). Its own spec (PROPS1-S) once the user picks A.
- B: more tile kinds (roof, wall-top, door tiles) in the world tileset. No schema change; buildings look blocky.
- C: the painted location backgrounds (`backgrounds/location/<id>.png`, already made) as the map image, the tile grid
  kept for collision only. Richest look; art and grid never line up exactly and every change needs a new painting.

## 3. Dialogue, camera and HUD (note 3)

Today (`note_3.jpg`): a full-width dock over the bottom 43% of the screen (UIR-6), portrait left, name and faction band
right of it, text in the middle, choices in a right column, a key hint line; the camera is clamped to the map, so at a
map edge the party can sit under the dock; the shop popup overlaps the dock (`note_4.jpg`).

### 3.1 Dialogue box

Recommended (one layout; the bottom position of preference "Dialogue sits in a bottom dock" stays):

- Box 400 wide, centred horizontally (gaps equal within 1 px), 6 px above the bottom edge; height from content, at
  least 104, at most 45% of the screen; width `min(W - 40, 400 x text scale)`.
- Left column 76 wide: portrait 64x64 at the top; the speaker's name under it (BODY, GOLD, centred, up to 2 lines);
  the faction band under the name (SMALL, DIM, "Ember Wardens: neutral").
- Right part in two sections split by a 1 px line: top, the NPC's line (BODY, wraps, scrolls past 4 lines); bottom,
  the player's choices as numbered rows (14 tall; locked rows dim with their reason as today; Continue is a row when the
  node goes on). Choices scroll past 5 rows.
- The key hint line goes (the numbers on the rows show 1-9; Esc is in the help icon, 3.4).

### 3.2 Camera and map edges

- A (recommended): no clamp. The camera centres on the party while walking and on the midpoint of the party and the
  speaker while talking, inside the map area above the dialogue box. Outside the map the view draws the location's
  wall tile (the world tileset's `wall`, the `#` of `rows`): render only, no data change, nothing walkable.
- B: pad every map with wall rows and columns in data. Shifts every anchor and saved position; breaks the journey
  parity and old saves. Not recommended.
- C: keep the clamp, shift the camera up only while talking. Fails at a map's bottom edge (the keep's gate).

Acceptance (A): with the party on any corner tile of an Ashlands map, the party is drawn at the view centre within one
tile; for every NPC marker in the Ashlands maps, opening its dialogue from any adjacent tile leaves both the party and
the NPC tiles fully inside the map area above the box at all four text sizes.

### 3.3 Shop placement

Recommended: while the shop is open the dialogue box is hidden, the map is dimmed, and the shop sits centred (4.2);
closing it brings the dialogue back on the node it left (today's `close_shop` flow). Alternative (not recommended): a
smaller shop above the box, which leaves no room for three columns at 640x360.

### 3.4 HUD bars

- Top left: only the location name (TITLE, GOLD). The supplies text and the help line go.
- Top right, left to right: gold (icon and amount), supplies (icon and `10/99`), then icon buttons Party (P), Journal
  (J), Stash (I), Help (?), Menu (Esc). Bar 20 tall; icons 14x14 in 18x18 hit boxes; tooltips name the button and its
  key. Menu opens the game menu popup as the Menu button does today; Party opens the Character popup on the first active
  member's Stats tab; Journal opens the journal; Stash opens the Character popup's Gear tab (Q2); Help's tooltip lists
  the controls (arrows or WASD move, E interact, click to walk, L log, 1-9 choose, Esc back).
- While a dialogue, shop, turn-in or lore popup is open the four buttons are disabled; gold and supplies stay live.
- Keys P, J and I do not clash with movement (WASD), E or L. The world map gets the same bar (Q8).

## 4. Shop as a trade (notes 4 and 5)

Today (`region.rs` `draw_shop`, `note_4.jpg`): a popup with Buy and Sell tabs over one list; each click buys or sells
one at once (supplies x1 or x5); the detail line shows the hovered item.

### 4.1 Options

- A (recommended): three columns, Shop | Trade | Stash, with clicks only (note 5): a click on a row adds 1 to the trade,
  `+5` and `+10` buttons add more, each row reads "X of N" (X in the trade, N available), a click on a trade line puts
  it back, Confirm settles the whole trade at once.
- B: as A with a quantity stepper per row ("X of Stock" with up and down arrows, note 4) and an Add button. Two clicks
  for one item; kept as a fallback if `+5` / `+10` feel coarse.
- C: as A plus drag and drop between the columns (DRAG1 C4 reshaped). The user said clicks are fine; drag can come
  later as a DRAG1 pick.

### 4.2 Layout (A)

Modal 600x300, centred under the HUD bar, over a dim of the map. Header: shop name (GOLD) and the keeper's faction band
(DIM). Columns at Normal: Shop 230, Trade 150, Stash 200, 6 px gaps; widths from text widths at other sizes; a long
name shortens with the full text on hover (TEXTFIT1 exception for list rows). Rows 14 tall, a 12 px icon slot at the
left (filled when ICONUI1 lands). Under the columns the detail line of the hovered item (today's `item_line`). Action
bar: Close at the left; Clear and Confirm trade at the right (UIR-1 bar convention).

- Shop column, a row per offer: icon, name, unit price (`30g`), "X of N" (N = stock left; an unlimited offer shows X
  only, blank at 0), `+5` and `+10` buttons (not on loot offers or rows with N at most 1). Supplies is a row with N =
  room left (`99 - supplies`). A row's price shows in the warning colour when it is above the gold left after the
  current trade (still clickable: a sale may pay for it).
- Stash column: every stash item the shop buys (today's `sell_list`), name, unit sell price, "X of N" (N = owned),
  `+5` and `+10`. Items the shop does not buy are listed below a divider, dimmed, tip "Not bought here".
- Trade column: buy lines first, then sell lines, one line per item and side (adding more raises its count), in the
  order added. A line reads `Healing Draught x2` with the gold right-aligned, `-60` for a buy and `+40` for a sale. Buy
  lines use the kit's warning colour, sale lines GOOD. Click a line: the whole line goes back (its row's X drops). At
  the bottom: `Total: -20 gold` and `Gold: 100 -> 80`.

### 4.3 Rules (A)

1. Adding: a row click adds 1; `+5` and `+10` add `min(n, N - X)`; nothing is added at `N - X = 0` (buttons disabled).
   A buy line is capped at `BUY_MAX` (99); loot offers add 1 at most (one at a time, as today).
2. Prices are the shop's prices when the shop opened (reputation does not change while it is open).
3. Confirm is enabled when the trade has a line and `gold + total >= 0`, `total = sales - purchases` (Q1: sales pay
   for purchases, "net"); otherwise its tip says "Not enough gold: N more needed.".
4. Confirm calls one new rule, `shop::trade` (4.4); on success the trade clears, the HUD gold updates and the log gets
   `Trade: bought N items for G gold, sold M items for H gold.`; on failure the errors show above the action bar and
   the trade stays.
5. Close, Esc or right click with lines in the trade asks "Discard this trade?" (No focused); with no lines it closes.
6. Focus order: shop rows, their `+5` / `+10`, trade lines, stash rows, Clear, Confirm, Close (UIR-7 builds on it).
   Widget ids for the agent link: `shop.row.<id>`, `shop.add5.<id>`, `shop.add10.<id>`, `stash.row.<item>`,
   `stash.add5.<item>`, `stash.add10.<item>`, `trade.buy.<id>`, `trade.sell.<item>`, `trade.clear`, `trade.confirm`,
   `shop.close`.

### 4.4 Rule contract: `shop::trade` (emberwake_world, game layer)

`pub fn trade(d: &Data, gs: &mut GameState, r: &str, buys: &[(String, String, i64)], sells: &[(String, i64)]) -> Dict`
with buys as `(kind, id, count)` (kind `item` | `loot` | `supplies`, as `offers`) and sells as `(item, count)`.
Returns `{ok, errors, spent, earned, gold, notes}` (`gold` = gold actually added or removed).

- Validation, all before any change (all-or-nothing): the shop is open; counts summed per id and side; each buy passes
  today's `buy` checks (count 1..=99, loot count 1, the offer exists, stock left and supplies room cover the summed
  count); each sale passes today's `sell` checks (in `sell_list`, owned count covers it); `gold + earned - spent >= 0`
  else `Not enough gold.`. A trade with no lines fails `Nothing to trade.`.
- Apply: sales leave the stash; buys enter the stash (loot claimed, the shop's sold counts raised, supplies added);
  gold changes once by `earned - spent`, capped at `GOLD_MAX` with today's cap note.
- `buy` and `sell` stay unchanged (the journey parity calls them); the screen no longer calls them.

Acceptance: tests show a one-line trade leaves the same state as today's `buy` or `sell`; a trade that sells 2 items
for 40 and buys one for 120 with 90 gold succeeds (net -80); the same with 70 gold fails and changes nothing; a buy of
6 against 5 left fails; a loot count of 2 fails; over the agent link `shop.row.<id>` twice and `shop.add5.<id>` give
`7 of N` (or N), `trade.buy.<id>` empties it, `trade.confirm` is disabled at negative gold-after.

## 5. Playtest note editor (note 5, engine)

Today (`aegis_ui::ui::text_field`, `Edit`, shell keys): a 96 px tall text area; typing, Backspace, Delete, Left, Right,
Home, End; a click starts editing with the caret at the end; no Up or Down, no selection, no clipboard, no undo.

- A: Up and Down, click to place the caret, selection (Shift with the moves, mouse drag, double click a word, Ctrl+A),
  Ctrl+C, Ctrl+X, Ctrl+V on the system clipboard.
- B (recommended): A plus Ctrl+Left and Right word jumps, Ctrl+Backspace word delete, Ctrl+Home and End, Page Up and
  Down, mouse wheel scroll with a scroll bar, Ctrl+Z and Ctrl+Y undo and redo. The text field is a kit widget the
  Content Forge's future editing will use too, so it pays once.
- C: Up and Down only.

Rules (B): Up and Down move by wrapped visual line and keep the column x (sticky until a horizontal move); on the first
or last line they go to the start or end. Typing or pasting replaces a selection. A paste into a one-line field joins
lines with a space; control characters other than newline are dropped. Undo groups a typing run (until a move, a pause
of 1 s, or a delete), at most 100 steps per edit. The clipboard is the `arboard` crate in `aegis_ui` (desktop only; the
engine has no web target); if it fails, an in-process clipboard keeps copy and paste working inside the app. The note
box's text area fills the popup's height (at least 160 at Normal). The Kit widgets gallery page shows the new keys.

Acceptance: `Edit` unit tests for up and down on wrapped text with the sticky column, selection replace, word moves,
undo grouping and paste normalising (in-process clipboard); the note box keeps Ctrl+Enter save and Esc cancel.

## 6. Assets (asset lane, PLAYUI1-I)

Every icon is a 32x32 PNG, transparent background, 16-colour palette (preference `assets-world.md`), readable at 12 to
16 logical px. Until a file lands each use falls back to a drawn shape (the ASSETORG2 chain ends in one), so no logic
row waits for art. Source: one sheet through the image API (asked first, per the standing rule) or `tools/gen_art.py`
drawn icons (Q7).

| Path | Ids | Used by |
| --- | --- | --- |
| `engine/assets/ui/<id>.png` (toolkit glyphs) | `lock`, `unlock`, `delete`, `help`, `chevron` | save slots, legend header, HUD help |
| `kits/rpg/assets/ui/hud/<id>.png` | `gold`, `supplies`, `party`, `journal`, `stash`, `menu` | HUD bar (3.4) |
| `kits/rpg/assets/ui/markers/<kind>.png` | `npc`, `chest`, `chest_open`, `lore`, `exit`, `world_map`, `encounter`, `roamer`, `point` | map markers and legend rows (2.1) |
| `kits/rpg/assets/ui/services/<id>.png` | `shop`, `recruit` | NPC badges and legend (2.2); `offer` and `turn_in` reuse `ui/badges/quest.png`, `ready.png` |

Total 22 icons (5 + 6 + 9 + 2). Props art belongs to PROPS1, sprites to SPRITE1-A2 and SPRITE2.

## 7. Superseded choices (HISTORY rows wanted when approved)

- UIR-6 "dialogue as a full-width bottom bar" becomes the 400 px centred bottom box (3.1); preference "Dialogue sits in
  a bottom dock" stays, refined.
- MAPREAD1 Q1 A "NPC and exit names always shown" becomes the `map_names` setting with default per Q3 (2.3).
- UI_REVIEW1 shop "Buy / Sell tabs" becomes the three-column trade (4); DRAG1 C4 (drag between counter and bag) is
  retired or reshaped as 4.1 C.
- MAPNAV1-U "Bryn's token as the party marker" becomes the leader's sprite (2.4).
- MAPVIEW1's clamped region camera becomes the centred camera with drawn wall margins (3.2).

## 8. Out of scope

New services (tavern, inn, trainer), buyback of sold items, shop restocking rules, prices and any balance number,
the battle screen, the world map beyond the HUD bar (Q8), MAPNAV1-U's edge arrows and pan rule, QUESTTRACK1-U's quest
card, region map art beyond icons (PROPS1, SPRITE2), drag and drop, save export and import.

## 9. Open questions (the user decides)

1. Shop gold check: net, sales pay for purchases in the same trade (recommended), or purchases alone must fit the gold
   you hold before selling?
2. Stash icon: open the Character popup's Gear tab, where the stash already lives (recommended), or a new stand-alone
   Stash pane?
3. Map names: default of the new setting On hover (recommended, your note) or Always?
4. NPC service icons: derived from what the NPC offers today (shop, recruit, quest offer, turn-in; recommended), or an
   authored role field? A tavern or inn has no game service yet: do you want one (its own spec)?
5. Legend and log: A floating legend plus the MAPLOG1 ticker (recommended), B two panes in the right column, or C tabs?
6. Buildings and props: A props layer on the map templates (recommended, then a PROPS1 spec), B more tile kinds, or C
   painted backgrounds?
7. Icon art: one 22-icon sheet through the image API (recommended; asked before generating) or drawn by
   `gen_art.py`?
8. The same HUD bar (gold, supplies, icons) on the world map (recommended) or the region map only?
9. Note editor scope: B with undo and word moves (recommended), A, or C? And should the Playtests browser show a note's
   full text on click (today it cuts each note to one line) with a Copy button?
10. Quantity: `+5` and `+10` per row (A, recommended) or the "X of Stock" stepper (B)?

## 9b. Answers (user 2026-10-22)

1. Shop gold: net (sales pay for purchases in the same trade).
2. Stash: the stash is the party's, shared; the Stash icon opens that shared stash. Loadouts stay on the Party screen,
   per selected member and tab (the user's words: "the stash is common for the party ... the loadout management is in
   the party screen for the selected member and tab").
3. Map names: On hover by default (setting `map_names`).
4. NPC icons: derived from what the NPC offers today; no tavern or inn service.
5. Legend and log: A, the floating legend plus the MAPLOG1 ticker.
6. Props: A, a props layer (PROPS1-S spec first).
7. Icon art: one 22-icon sheet through the image API (ask before generating; drawn shapes stand in until then).
8. World map: the same HUD bar.
9. Note editor: B, the full editor (undo, word moves), and the Playtests browser shows a note's full text with Copy.
10. Quantity: `+1` (click), `+5` and `+10` per row.

## 10. Build rows wanted (status `spec`; the planner writes them)

| Id | Title | Layer | Lane | Depends on |
| --- | --- | --- | --- | --- |
| PLAYUI1-K | Kit: icon button (image, tooltip, focus, toggled, disabled) and collapsible section header, on the Kit widgets page | engine | logic | - |
| PLAYUI1-T | Kit text editor: up and down, click caret, selection, clipboard, word moves, undo; bigger note box (section 5) | engine | logic | - |
| PLAYUI1-S | Save slots: row click, lock and delete icons, `aegis_save` slot locks (section 1) | engine + game | logic | PLAYUI1-K |
| PLAYUI1-H | HUD bars on the region map (and world map, Q8): name only top left; gold, supplies, icon buttons top right (3.4) | game | logic | PLAYUI1-K |
| PLAYUI1-M | Region map: full-width map, floating icon legend, `npc_services`, names setting, centred camera with wall margins (2.1-2.3, 3.2); folds MAPLOG1-U and MAPNAV1-U's collapse | game | logic | PLAYUI1-K, PLAYUI1-H (region.rs, in sequence) |
| PLAYUI1-D | Dialogue box: 400 px centred bottom box, name under the portrait, two sections, camera on both speakers, shop hides it (3.1, 3.3) | game | logic | PLAYUI1-M |
| PLAYUI1-P | Shop trade: three columns, trade lines, `+5` / `+10`, Confirm via `shop::trade`; the shop moves to `emberwake_gui/src/shop.rs` (section 4) | game | logic | PLAYUI1-D |
| PLAYUI1-I | 22 UI icons (section 6) | engine + kit | asset | Q7 answered |
| PLAYUI1-R | Region map sprites: leader and NPC bodies, idle and walk, facing (2.4) | game | logic | SPRITE1-L, SPRITE1-A2, PLAYUI1-M; NPC bodies with SPRITE2 |
| PROPS1-S | Spec: props and buildings on region maps (2.5, the picked option) | game | design | Q6 answered |

Touches: K `engine/crates/aegis_ui/src/{ui,specimens}.rs`; T `engine/crates/aegis_ui/src/{ui,shell,notes,specimens}.rs`,
`engine/crates/aegis_ui/Cargo.toml` (`arboard`); S `engine/crates/aegis_save`, `emberwake_gui/src/menus.rs`; H
`emberwake_gui/src/{region,worldmap}.rs`; M `emberwake_gui/src/region.rs`, `emberwake_gui/src/menus.rs` (setting),
`emberwake_world/src/explore.rs` (`npc_services`); D `emberwake_gui/src/region.rs`; P `emberwake_gui/src/{region,shop}.rs`,
`emberwake_world/src/shop.rs`; I `engine/assets/ui/`, `kits/rpg/assets/ui/{hud,markers,services}/`; R
`emberwake_gui/src/region.rs`. K, T and I run beside anything; H, M, D, P and R share `region.rs` and run one at a
time in that order.
