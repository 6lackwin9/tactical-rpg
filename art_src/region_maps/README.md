# Region maps: layouts, art and the real-art preview

The Art/UI working set for the Sunk Ward exploration maps (Inbox `AI-12`, look-and-feel session 2026-11-15). The full
history, with Tushar's feedback round by round, is in `games/emberwake/docs/specs/ART_UI_STUDIO.md` 3.10. Start here
next session.

## What is where

| Path | What |
| --- | --- |
| `build_layouts.py` | Writes and checks the nine draft layouts (`<location>.json`, the `maps.json` template shape plus `draft_markers`, `draft_zone_bindings`, `draft_room_styles`) |
| `render_map.py` | Draws a layout with real art at 32 px a tile: toned ground textures with wandering borders, patchy ash and moss, walls with top, face and cap line, wall-mounted props set into the wall, light pools |
| `previews/<location>.png` | The latest render of each place (the hub so far) |
| `../sunk_ward/sheets/` | The web-queue prop and icon sheets that were picked (the masters; the rest is on Drive `image_pipeline/output/`) |
| `../sunk_ward/tiles/` | The picked ground, boundary and interior wall textures |
| `../sunk_ward/props_cut/<art>/v0.png` | The 81 props cut from the sheets (`_contact.png` shows them all); not in the game yet |
| `../prompts/cut_props.py` | Cuts the sheets into props (`PICKS`: sheet, take and cell per art; sizes from `sunk_ward_props.md`) |
| `../prompts/queue_sunk_ward.py` | Queues props and tile requests on the Drive web queue (no API) |
| `../prompts/sunk_ward_props.md`, `sunk_ward_walls.md`, `sunk_ward_icons.md` | The prompts |
| `tools/mock_ui/previews/region_map.py` | The grid preview (coloured blocks, zones, anchors, zoom), published as "Sunk Ward Region Maps" |

## Commands

```bash
py -3.12 art_src/region_maps/build_layouts.py
```

```bash
py -3.12 art_src/region_maps/render_map.py sluice_undercroft
```

```bash
py -3.12 art_src/prompts/cut_props.py art_src/sunk_ward/sheets --out target/review/propcut
```

After a re-cut, copy `target/review/propcut/games/emberwake/assets/props/sunk_ward/*` into `art_src/sunk_ward/props_cut/`.

## Rules settled on 2026-11-15

- **One view**: every prop is drawn front-on from slightly above (front face flat, every edge horizontal or vertical).
  Never attach an isometric landmark as a style reference: a reference carries its viewing angle with it.
- **Two profiles**: a prop that lies on the floor has a side profile, `<art>_side`, for a west or east wall (flipped).
  Wall-mounted props (bars, ladders, gates, doors) only exist front-on so far, so they go on north walls.
- **Rooms, not one big room**: a place is broken into walled rooms, each its own zone with its own floor and style.
- **Quiet ground**: few grounds a room, low contrast, ash and moss in patches, borders that wander.
- **No cast shadows** under props; a thin dark line at the base. Wall-mounted art is fitted and toned to the wall.
- **Tile prompts say dark and low-contrast outright**; "sooty" and "pale" come back white.
- **Icons are scaled, not pixelated**, except 14 px chips, which are drawn as bold 16 x 16 sprites.

## Open, in order

Process for every map, the hub included (Tushar, 2026-11-17): first a layout reference image of the whole map, then
rebuild that look with terrain tiles and props, shown side by side with the reference (ART_UI_STUDIO 3.3).

1. The hub's gate arch and bunks were drawn for a taller wall and look cramped in a one-tile face: redraw them for
   that height (a full sluice gate arch 4 wide; a single-tier bunk niche 2 wide), or make walls two tiles tall.
2. Wall-mounted props on side walls: no art yet (the model draws them front-on). Try single-prop prompts that
   describe the top-down strip, or draw the openings in code.
3. Per-room wall styles: done 2026-11-17. `render_map.py` `ROOM_WALLS` gives the Slab whitewashed plaster and the
   Crypts cold stone (from `draft_room_styles`); every other room keeps the drain brick. A wall cell takes the set of
   the room it faces. The new preview is in the project files (`art-ui/hub_room_walls.png`; Git LFS uploads are
   blocked from the cloud), so `previews/sluice_undercroft.png` is still the 11-15 render until a desktop re-run.
4. The Grate next, then the five mission maps, each broken into rooms like the hub.
5. Place the cut props, tiles and picked textures in the game once the look is signed off (`PR5-54`, `PR5-55` import
   the layouts on placeholders first). The 11 single building images (S10) are not queued yet.
6. Waiting on Design: Outbox `AO-07` to `AO-10`.
