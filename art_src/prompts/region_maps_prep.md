# Region maps per landmark: prep for the next Art/UI session

Prepared 2026-11-13 at Tushar's request ("we will move to the region maps for each landmark, so have the prep done
for that for tomorrow's session"). The next session also does the Act 1 map cleanup first (below). Requests that need
Design or Build are in Outbox `AO-04` (`games/emberwake/docs/specs/ART_UI_REQUESTS.md`); check Design's answers in
`docs/READY.md` section 4 before starting.

## 1. How a region map is drawn today

A location's region map is the place you walk around in after picking it on the act map.

- Screen: `RegionMap` (`games/emberwake/crates/emberwake_gui/src/region.rs`), square or tilted (`mapview.rs`).
- It is a **grid, not a painting**: the map template's `rows` (`.` open, `#` wall, `~` water) drive walking. When a
  template has a `ground` layer, `ground.rs` composes a picture from ground textures (REGIONGROUND1) and draws it in
  place of the `.` and `~` tiles. Props (PROPS1) draw over it; characters and markers are sprites.
- Templates: `games/emberwake/data/maps/maps.json` (schema: `games/emberwake/data/maps/README.md`): `id, name, rows`
  (8..64 x 8..64), `zones, zone_legend, anchors [{id, x, y}], paths, props [{id, art, x, y, w, h, layer, variant,
  flip}], ground, ground_legend, ground_patches, wall_storeys, tags`. Edited only through `aegis content`.
- A painted per-location backdrop has a lookup, `rpgkit_content::art::location_background` (`backgrounds/location/<id>.png`),
  but nothing calls it (REGIONGROUND1 settled on the composed ground, so a map edit redraws it).

## 2. Art contract (where the art goes)

| Art | Path | Size | Spec |
| --- | --- | --- | --- |
| Ground texture | `games/emberwake/assets/tilesets/<set>/ground/<name>/v0.png` | 256 x 256, seamless | REGIONGROUND1 sections 4 and 7; cut with `tools/tiles/cut.py` |
| Wall face, boundary walls | `tilesets/<set>/wall_face`, `tilesets/<set>/boundary/<b>/{wall,wall_face}/v<n>.png` | per REGIONGROUND1_SCENE | REGIONGROUND1_SCENE, LOOK1-A as narrowed |
| Prop or building | `games/emberwake/assets/props/<set>/<art>/v<n>.png` (kit: `kits/rpg/assets/props/common/`) | 32 px per tile | PROPS1 section 6, PROPS1_PROMPTS |
| Map marker | `map/markers/<marker>.png` | as the kit's 10 | - |
| Mark sheet (footprints, scuffs) | REGIONGROUND1-A2 | sheet on magenta | REGIONGROUND1 section 7 |

A location picks its set with `tileset` (falls back to the world's lineage, then `common`). No Sunk Ward location sets
one yet, and no Sunk Ward set exists: the art so far is Ashlands (9 ground textures, 3 props) plus the kit's
`common` props (16).

## 3. Act 1 locations and their maps today

| Location | Act map pick (landmark) | Region map today |
| --- | --- | --- |
| 00 The Sluice Undercroft (hub) | slab-free hideout | `walled_town` (16 x 10, generic) |
| 01 Drowned Runoff Grate | revised take 2 | `gorge_ruins` (20 x 12, generic) |
| 02 Alleyway Junction 7 | take B | `gorge_ruins` |
| 03 Ash-Crawl Crossroads | take A | `gorge_ruins` |
| 04 The Fourth Kiln | take B | `gorge_ruins` |
| 05 The Tallow Vats | take A (tinted) | `gorge_ruins` |
| 06 Crucible Watchpost | take B | `gorge_ruins` |
| 07 The Slag Gate | take B | none (exit buttons) |
| 1d Corvo's Gate | take A | `gorge_ruins` |
| Alleyway of Cinders, Meat-Market Rooftops, Canal Bridge | no landmark (not on the act map) | none |

So 8 places share 2 generic templates. Each landmark picture is already the best brief for its region map: it is the
place seen from above, with its features (the Kiln's chimneys, the Vats' tanks, the Grate over black water). Acts 2
and 3 have no location data yet (`docs/specs/ACTS23.md`: "two act regions" to author), so their region maps wait.

## 4. Proposed plan for the session (pending Design's answers to AO-04)

1. **Sunk Ward ground set** (`sunk_streets`): REGIONGROUND1-A3's nine textures (soot_yard x2, cobble_road, flagstone,
   mud, moss, iron_catwalk, tallow, water; prompts and tones in REGIONGROUND1 section 7). Approved, but it waits on
   R1's tuned prompt (Ashlands review). AO-04 asks whether Art/UI may run it now and re-roll after R1.
2. **Sunk Ward props list**: PROPS1 section 6.2 lists Ashlands only. Draft the Sunk Ward list from the nine landmark
   picks (sluice gates, grates, canal planks, tenement fronts, kiln chimneys, tallow vats, barricades, toll booth,
   cages, watch lanterns), each with its footprint in tiles; Design approves it into PROPS1 (AO-04).
3. **One region map layout per landmark**: a mock per location, a top-down grid sketch that follows its landmark
   (entrances where the act-map routes arrive, the anchors its markers use: `gate`, `road`, `npc_a..`), checked in a
   preview before Design specs the templates and Build fills `maps.json`. Start with the hub (most visited) and the
   Grate (first mission ground).
4. Placeholder-first: anything a template needs before its art lands keeps the kit's `common` art.

## 5. Act 1 map cleanup (first item next session)

Act 2 and Act 3 are closed (`ART_UI_STUDIO.md` 3.10, "Act 2 and Act 3 maps closed"). Act 1 gets the same treatment:

1. Slab-removal Flow edit on the 8 Act 1 landmarks (Grate, Junction 7, Crossroads, Kiln, Vats, Crucible, Slag Gate,
   Corvo's Gate; the hideout is shared and its Drive copy is already slab-free). One prompt per place in
   [act1_slab_edits.md](act1_slab_edits.md); the Drive `flow/out/World Map` JPGs are the sources.
2. Act 1 in the Act Map Layout page (https://claude.ai/artifact/LvgjBQTj9enpJG6J5CQEBb, private): Tushar places and
   sizes; Art/UI writes `art_src/prompts/act1_map_layout.json`.
3. `blend_act_map.py codeblend` for Act 1, the Act 1 preview rebuilt from it, Outbox `AO-01` updated.
4. Full-size files: when they arrive (Drive), save each pick as `art_src/<id>.png` and run `codeblend` for all three
   acts; that writes every landmark and background in place.
