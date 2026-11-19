# BIGUNIT1: units that fill 2 x 2 cells

Status: active (user 2026-11-17: "build the 2x2 unit work if the placeholder makes no sense, its better to have the actual
version than a test version which gets replaced later"). Supersedes `ACTS23-R7` `B1` Option B and lore ruling `F6`
(the 1 x 1 cell with an oversized sprite). Answers `ACTS23` Q-A6 (a): a real footprint in the battle grid. Layer: kit
(`rpgkit_battle`, `rpgkit_data`, `rpgkit_content` check, kit `schema.json`), game UI (`emberwake_gui` battle drawing)
and game data (three units). Numbers and choices marked BU are `[PROVISIONAL]` defaults (`P-COMBAT`, lowest undo cost).

## 1. Data

- BU1. Units gain an optional `footprint` (`1` or `2`, default `1`) in the unit catalog row and, when a swap or an
  encounter spawns a unit, from that row. `2` means the unit fills a 2 x 2 square. Larger sizes are out of scope.
- BU2. A unit's position stays one cell, its anchor: the top-left cell of the square. Its cells are the anchor plus
  `(+1, 0)`, `(0, +1)` and `(+1, +1)`. Saves keep the anchor only, so no save migration is needed.
- BU3. Content check: a placed big unit must fit (all four cells inside the arena, walkable, not an exit, no overlap with
  any other unit's cells). An encounter that fails is an error, not a warning.

## 2. Grid rules

- BU4. Occupancy: all four cells are occupied by the unit. `World::occupant` and `is_occupied` answer for every cell.
- BU5. Distance: from a cell or a unit to a big unit is the least distance to any of its four cells, using whichever
  metric that check already uses (Manhattan `V2::dist`, or `band_dist` with diagonals under `melee_diagonal`). A big
  unit's own reach and range count from its nearest cell to the target. One helper per metric replaces direct
  `pos(a).dist(pos(b))` calls so every reach, range, threat, adjacency and AI check agrees.
- BU6. Movement: a big unit moves its anchor one step at a time; a step is legal only if all four destination cells
  are walkable, inside the arena, not exits and not occupied by another unit. Step cost is the highest move cost among
  the cells it enters. It cannot leave by an exit.
- BU7. Hazards and ground traps: entering a step rolls each hazard tile newly covered, once per tile; a trap springs as
  for a normal unit and stops the move.
- BU8. Forced movement (`forced.rs`): a big unit is pushed or pulled only if the whole square fits each step (FM3 applied
  to all four cells); otherwise the step is a collision as usual. A 2-wide edge can meet two blockers: each takes the
  FM4 blocker hit (widen `Moved.blocker` to a list, reading order).
- BU9. Areas: an area skill hits a big unit once if any of its cells is in the area, never once per cell.
- BU10. Cover: the kit has no line of sight and this adds none. `cover_for` (target tile plus walls next to it on the
  attacker's side) is taken for each of the four cells and the least cover applies.
- BU11. Flanking keeps MV3 (`is_flank`, `in_front`): a target engaged with another unit, attacked from outside its
  facing front. For a big unit, `in_front` and `engaged_with` measure from the cell of the square nearest the other
  unit, and its facing front is the side of the square it faces.
- BU12. Swap (`PHASEFX1` `swap`): when the new unit is bigger than the old one, it is placed with the old cell as one of
  its four cells, trying the four squares in a fixed order (old cell as top-left, top-right, bottom-left, bottom-right);
  if none fits, the nearest fitting anchor on the board by distance then reading order. If no 2 x 2 space is free
  anywhere (rare, it depends on where units stand), the unit makes room (user 2026-11-17: "it should just push back all
  units on board to free up space"): it takes the first of the four squares around the old cell whose terrain is
  walkable (else the nearest such square on the board), and every other unit on those cells is pushed out to the nearest
  free walkable cell outside the square (distance, then reading order), pushing further units back in turn if needed.
  These pushes are not attacks: no collision damage, hazards roll as a forced move does, and the log names each moved
  unit. The new unit's footprint comes from the swap target's unit row (`to`), so a boss built from a
  1-cell `"$captive_unit"` snapshot still gets `footprint: 2`.
- BU13. AI: pathing and target choice for and against big units use BU5 and BU6. AI picks attack cells next to any cell.

## 3. Drawing and input (game UI)

- BU14. The battle screen draws a big unit's sprite over its 2 x 2 square (sprite scaled from the unit's sheet until a
  2 x 2 sheet exists), with one shadow, one health bar and one intent marker centred on the square.
- BU15. Selection, hover, targeting highlights and move previews show all four cells. Clicking any of the four cells
  selects or targets the unit.

## 4. Data to seed (through `aegis content`)

- BU16. `footprint: 2` on `calcified_amalgam` (Act 1 Interlude, `enc_evt_05`), on `saint_aberration_<class>` (node
  20) and on the Dreaming Core / titan unit (node 22) when `PR6-13` seeds them. Check the Interlude arena fits it.

## 5. Tests

- Kit: occupancy, distance, movement, forced moves, areas once, flanking and swap placement, each with a fixture.
- Content check: a big unit that overlaps, leaves the arena or sits on an exit is an error.
- World: the Interlude battle with the Amalgam at `footprint: 2` resolves through the scenario runner.
- Goldens: classic and rules goldens must not move (no big units there). The Amalgam change may move the
  `emberwake_sim` sim golden and the journey; such a re-record is named in the commit and a `HISTORY.md` row.

## 6. Art

The Amalgam, the Saint-Weapon aberration and the Dreaming Core need sprite sheets drawn for a 2 x 2 body. Until they
land, BU14 scales the existing sheet in place (decision "Placeholder-first assets"); Art/UI request `AI-16`.
