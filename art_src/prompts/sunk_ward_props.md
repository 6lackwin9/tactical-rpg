# Sunk Ward props: draft list and prompt sheets (AI-12, for Design review)

Drafted by Art/UI 2026-11-14 for Design to approve into `PROPS1.md` section 6.2 and `PROPS1_PROMPTS.md` (decision
AO-04 (2)). Nothing is generated before that approval and the user's go-ahead on the model and the image count
(`P-UI-06`, PRINCIPLES 0.5). Until the art lands, a template draws the kit's `common` art where the art name matches
and a solid block where it does not (`P-UI-04`, placeholder-first).

The arts come from the nine draft layouts in `art_src/region_maps/` and from their act-map
landmarks (`art_src/lm_<location>.jpg`), which are the style and colour reference. Paths:
`games/emberwake/assets/props/sunk_ward/<art>/v<n>.png`, 32 px per tile; a standing prop's picture may be one tile
wider and up to four taller than its footprint (PROPS1 6.1). Style A as in PROPS1 6.1.

**Picture boxes widened (2026-11-15, after the first hub render)**: a standing prop's picture may be one tile wider than its
footprint (PROPS1 6.1). The front-on art is wider than the narrow boxes first listed, so 21 small props (racks, posts,
lamps, the cot, table, bunk, slab, anvil, brazier, strongbox and so on) now use that extra tile; footprints are unchanged.

## 1. Arts for the hub and the Grate (26 arts: 24 new, 2 reused from the kit)

| Art | PNG px | Footprint | Layer | Where | What |
| --- | --- | --- | --- | --- | --- |
| `sluice_arch` | 64 x 128 | 2 x 1 | sort | hub | one half of the barred sluice gate arch in a soot-black brick wall: brick jamb, the arch curving inward at the top, iron bars raised into the arch; `flip` makes the other half |
| `vault_pillar` | 64 x 160 | 2 x 2 | sort | hub | a squat soot-black brick pier with a stone footing, rising into the vault |
| `retort_shelf` | 96 x 96 | 3 x 1 | sort | hub | a dark wooden shelf of glass retorts, jars and bone pins, a dripping oil lamp |
| `apothecary_slab` | 96 x 112 | 3 x 2 | sort | hub | a stone dissection slab on iron legs, a folded oiled-green cloth, scalpels and pipettes, a hanging lamp |
| `herb_rack` | 64 x 96 | 1 x 2 | sort | hub | a narrow drying rack of grey herbs and rock-salt sacks |
| `tinker_hearth` | 96 x 160 | 3 x 2 | sort | hub | a brick forge hearth with a glowing tallow-yellow fire, bellows, a hooded flue rising into the vault |
| `anvil` | 64 x 64 | 1 x 1 | sort | hub | a blackened anvil on a stump, a hammer on it |
| `scrap_heap` | 64 x 96 | 2 x 2 | sort | hub | a heap of salvaged scrap: bent pipe, valve wheels, chain, cracked brass |
| `brazier` | 64 x 80 | 1 x 1 | sort | hub | an iron brazier on three legs with a low tallow-yellow fire |
| `kiln_bunk` | 80 x 80 | 2 x 1 | sort | hub | a bunk set into a brick kiln niche: straw pallet, grey blanket, a candle stub |
| `stasis_slab` | 80 x 64 | 2 x 1 | sort | hub | a stone slab with leather restraints under a sheet, chalk marks on the stone |
| `crypt_bars` | 64 x 160 | 1 x 3 | sort | hub | a run of rusted iron cell bars in a brick frame, running north to south |
| `mission_board` | 64 x 96 | 2 x 1 | sort | hub | a wooden board on two posts hung with bone-cipher slips and pinned scraps (no letters) |
| `iron_ladder` | 64 x 128 | 1 x 2 | sort | hub | a rusted iron ladder bolted up a brick wall, going out of sight |
| `varka_skiff` | 64 x 32 | 2 x 1 | ground | hub | a small flat skiff moored on black water, crates and a lantern on it, seen from above |
| `crate_stack` | (kit) | 2 x 1 | sort | hub | kit `common` art reused |
| `drain_arch` | 96 x 128 | 3 x 2 | sort | Grate | a small arched drain mouth in a brick wall, a trickle of black water from it |
| `sluice_tunnel` | 64 x 128 | 2 x 1 | sort | Grate | one half of the big sluice tunnel mouth: a deep brick arch, dark water running out; `flip` makes the other half |
| `ring_post` | 32 x 64 | 1 x 1 | sort | both | a square stone mooring post with an iron ring |
| `lock_tablet` | 64 x 96 | 1 x 1 | sort | Grate | an upright bronze flood-lock tablet on a stone plinth, half buried in silt (no letters) |
| `lantern_pole` | 64 x 112 | 1 x 1 | sort | Grate | a crooked iron pole with a hanging tallow-yellow lantern |
| `culvert_gate` | 64 x 128 | 1 x 2 | sort | Grate | a barred iron culvert gate set in a brick wall, a chain and padlock |
| `drain_grate` | 256 x 128 | 8 x 4 | ground | Grate | a big square iron grate over swirling black water, rust at the bolts, seen from above |
| `silt_bank` | 128 x 64 | 4 x 2 | ground | Grate | a bank of grey river silt with ripples and a broken lock-plate edge |
| `stone_steps` | 96 x 96 | 3 x 3 | ground | Grate | worn stone steps going up to the south, soot in the corners, seen from above |
| `rubble_wall` | (kit) | 4 x 1 | sort | Grate | kit `common` art reused |

## 2. Arts for the other seven Act 1 places (43 arts: 39 new, 4 more reused from the kit)

Each art has one footprint, since a prop draws at its picture's size and is never stretched (PROPS1 4.3); walls,
fences and shack rows repeat one piece. Kit arts reused here: `gatepost`, `cliff_face`, `boulder`, `banner_pole`
(with `crate_stack` and `rubble_wall` from section 1). The violet light at the Crossroads and in the Crucible's
tower window is asked for as a cold blue-violet, which keyed apart from the magenta on the landmark cuts.

| Art | PNG px | Footprint | Layer | Where | What |
| --- | --- | --- | --- | --- | --- |
| `tenement_block` | 192 x 288 | 6 x 5 | sort | Junction 7 | a crooked three-storey tenement of soot-black brick and timber, slate roof, shuttered windows, two lit |
| `watch_armory` | 192 x 288 | 6 x 5 | roof | Junction 7 | a squat brick Watch armory, an iron-bound door, barred windows, a rust-crimson banner (enterable: the cache inside) |
| `watch_torch` | 64 x 80 | 1 x 1 | sort | Junction 7, Slag Gate | an iron torch bracket on a short post, a red-orange flame |
| `rust_pennant` | 64 x 96 | 1 x 1 | sort | Junction 7 | a pole with a rust-crimson pennant |
| `spiked_barricade` | 96 x 64 | 3 x 1 | sort | Junction 7 | a spiked timber barricade lashed with chain |
| `street_grate` | 64 x 64 | 2 x 2 | ground | Junction 7 | a square iron street grate chained shut, seen from above |
| `ash_shack` | 128 x 160 | 4 x 3 | sort | Crossroads | a ramshackle plank shack, a sagging roof of tin sheets under grey ash |
| `cage_platform` | 96 x 128 | 3 x 2 | sort | Crossroads | two iron cages on a wooden platform, chains, a titration bench of glass and tubes |
| `pen_fence` | 96 x 64 | 3 x 1 | sort | Crossroads | a rough fence of iron stakes and timber rails |
| `toll_pen_gate` | 64 x 96 | 2 x 1 | sort | Crossroads | the chained Toll-Pen gate, a padlock, a blank tally board |
| `violet_lamp_post` | 64 x 128 | 1 x 1 | sort | Crossroads | a crooked iron lamp post with a cold blue-violet flame |
| `violet_pool` | 64 x 64 | 2 x 2 | ground | Crossroads | a soft pool of cold blue-violet light on ash cobbles |
| `ash_drift` | 96 x 64 | 3 x 2 | ground | Crossroads | a drift of grey ash and cinders |
| `brick_wall_run` | 128 x 128 | 4 x 2 | sort | Corvo's Gate | a run of crumbling warm red-brown brick wall, moss in the cracks |
| `brick_breach` | 64 x 128 | 2 x 2 | sort | Corvo's Gate | the broken end of the brick wall at a breach, bricks spilling; `flip` for the other side |
| `striped_barrier` | 96 x 64 | 3 x 1 | sort | Corvo's Gate | a red-and-white striped barrier pole on a trestle, lowered |
| `toll_booth` | 64 x 128 | 2 x 2 | sort | Corvo's Gate | a small timber guard booth, a slanted roof, a lantern by the window |
| `toll_drum` | 64 x 64 | 1 x 1 | sort | Corvo's Gate | an iron-banded toll drum on a stand, a slot in its lid |
| `tenement_row` | 192 x 224 | 6 x 3 | sort | Corvo's Gate | a row of narrow two-storey soot-black tenements |
| `fourth_kiln` | 448 x 320 | 14 x 6 | sort | Kiln | the bone-mill kiln: a soot-black brick block, three chimney tops glowing inside with no smoke, pipes and iron bands on its face |
| `furnace_mouth` | 96 x 96 | 3 x 1 | sort | Kiln | a glowing furnace mouth in a brick arch, ash at its sill |
| `conveyor_chute` | 64 x 160 | 2 x 3 | sort | Kiln | an inclined wooden conveyor chute on a trestle, bones on the belt |
| `brass_tank` | 64 x 112 | 2 x 2 | sort | Kiln | a riveted brass tank with two gauges |
| `pipe_run` | 64 x 96 | 2 x 2 | sort | Kiln, Vats | a run of rusted pipes on brackets, a valve wheel |
| `bone_meal_heap` | 64 x 64 | 2 x 2 | ground | Kiln | a heap of pale bone meal |
| `wedge_pile` | 32 x 32 | 1 x 1 | ground | Kiln | a small pile of black-iron wedges |
| `tallow_vat` | 96 x 160 | 3 x 3 | sort | Vats | a big riveted iron vat with a glowing sickly-yellow tallow top and a ladder up its side |
| `catwalk_stair` | 64 x 160 | 1 x 3 | sort | Vats | a steep iron catwalk stair with a rail |
| `tallow_spill` | 96 x 64 | 3 x 2 | ground | Vats | a spill of pale yellow tallow, congealed at the edges |
| `still_room` | 352 x 320 | 11 x 7 | roof | Vats | a low brick still-room, copper still pipes through its roof (enterable: Elyse's counter-still) |
| `guild_store` | 352 x 288 | 11 x 6 | roof | Vats | a Guild storehouse of brick, wide double doors, sickly-yellow Guild marks, no letters (enterable: Orlov's stores) |
| `barrel_stack` | 64 x 64 | 2 x 1 | sort | Vats | tallow barrels stacked two high |
| `winch_drum` | 64 x 128 | 2 x 2 | sort | Crucible | a big iron winch drum in a timber frame, its chain running down |
| `watch_wall` | 128 x 128 | 4 x 2 | sort | Crucible | a run of fortress wall, rust-trimmed battlements, one red-orange slit window |
| `crane_arm` | 64 x 128 | 1 x 1 | sort | Crucible | an iron crane arm with a pulley and hook |
| `slit_window_tower` | 160 x 256 | 5 x 3 | sort | Crucible | a stone watch tower, a row of red-orange slit windows, battlements, a cold blue-violet glint in its top window |
| `cliff_bastion` | 160 x 192 | 5 x 3 | sort | Slag Gate | a fortified stretch of dark cliff faced with masonry, a red banner; `flip` for the other side |
| `portcullis` | 128 x 160 | 4 x 2 | sort | Slag Gate | a shut iron portcullis in a stone gate arch |
| `slag_channel` | 128 x 256 | 4 x 8 | ground | Slag Gate | an L-shaped channel of glowing slag: down the left side, turning right along the bottom, seen from above |
| `gatepost` | (kit) | 1 x 1 | sort | Corvo's Gate, Crucible | kit `common` art reused |
| `cliff_face` | (kit) | 4 x 1 | sort | Crucible | kit `common` art reused |
| `boulder` | (kit) | 2 x 2 | sort | Crucible | kit `common` art reused |
| `banner_pole` | (kit) | 1 x 1 | sort | Slag Gate | kit `common` art reused |

## 2b. Enterable buildings (the `roof` layer, REVEAL1 draft; mission maps at full size, 2026-11-14)

The mission grounds were redrawn at two to three screens (Tushar, "Go large for missions"), with buildings the party can
walk into. Such a building is a wall ring with an open floor and one doorway in the map's `rows`; its picture is a prop
on a `roof` layer over the whole footprint, drawn above the characters, showing the closed building (roof and front
wall). It fades while the party stands in the building's indoor zone (`roof_of`), so the floor and walls drawn by the
tiles show through. The art is a normal building picture; only its layer is new (REVEAL1 is Build's).

| Art | PNG px | Footprint | Layer | Where | What |
| --- | --- | --- | --- | --- | --- |
| `doss_house` | 352 x 256 | 11 x 5 | roof | Junction 7 | a sagging two-storey doss house of soot-black brick and timber, patched slate roof, a lantern over its door (the contact inside) |
| `rag_hovel` | 352 x 224 | 11 x 5 | roof | Crossroads | a long ragpicker's hovel of planks and tin under grey ash, bundles of rags on its roof (the contact inside) |
| `bonewright_shop` | 320 x 288 | 10 x 6 | roof | Kiln | a bone-wright's brick workshop, a squat chimney, pale bone tools hung by the door (Skerrit inside, after M3) |
| `sentry_hut` | 224 x 256 | 7 x 6 | roof | Crucible | an old stone sentry hut set into the cliff, a slate roof, a shuttered window (a safe stop) |
| `ward_house` | 224 x 288 | 7 x 6 | roof | Junction 7, Crossroads, Kiln, Vats | a narrow soot-black brick and timber house, a slate roof, one lit window; v0-v2 (three takes: other roofs, doors and windows), picked by position |
| `ward_shop` | 224 x 288 | 7 x 6 | roof | Junction 7, Crossroads, Kiln, Vats | a small shop front of soot-black brick, a hanging sign with a picture and no letters, a lit display window; v0-v1 |
| `ward_shrine` | 224 x 256 | 7 x 6 | roof | Junction 7, Crossroads, Kiln, Vats, Crucible | a small wayside shrine of dark stone, a pitched roof, candles at its open door |
| `guardroom` | 224 x 256 | 7 x 6 | roof | Crucible | a squat Watch guardroom of dark fortress stone, a barred window, a red-orange lamp at the door |

**Flavour buildings (Tushar, 2026-11-14: more explorable indoor areas for side quests and lore NPCs)**: each mission
street map has three enterable buildings off its streets (a shop, a house, a shrine), and the Crucible has two (a
shrine, the guardroom), each with an `npc_<zone>` anchor inside for Design to bind a side-quest giver or a lore
character. They share the four pictures above, with takes for variety, so a map edit can add more without new art.

## 2d. Interiors: furniture, floors and walls

Every enterable building is furnished (Tushar, 2026-11-14: "the indoor maps would need their own interiors and
props"). `build_layouts.py` `FURNISH` gives each building type its pieces; they stand along the walls, farthest from
the door first, and a piece is kept only if the door, the anchors and every floor tile stay reachable. Under the roof
they are hidden; with the party inside, the roof fades and they show.

| Art | PNG px | Footprint | Layer | Where | What |
| --- | --- | --- | --- | --- | --- |
| `shop_counter` | 96 x 64 | 3 x 1 | sort | shops | a worn wooden shop counter, a lamp and a few wares on it |
| `goods_shelf` | 64 x 96 | 2 x 1 | sort | shops, hovel, workshop, still-room | a tall wooden shelf of jars, bundles and tins |
| `cot` | 96 x 64 | 2 x 1 | sort | houses, doss house, hovel, guardroom, sentry hut | a low cot of planks with a grey blanket |
| `table_stools` | 96 x 80 | 2 x 1 | sort | houses, doss house, hovel, guardroom, sentry hut | a rough table with two stools, a candle stub |
| `hearth_stove` | 64 x 96 | 1 x 1 | sort | houses, sentry hut | a small iron stove with a pipe going up and a low tallow-yellow glow |
| `shrine_altar` | 96 x 80 | 3 x 1 | sort | shrines | a stone altar with a cloth, a bone reliquary and candles |
| `candle_stand` | 64 x 80 | 1 x 1 | sort | shrines | an iron candle stand with a cluster of tallow candles |
| `weapon_rack` | 64 x 80 | 2 x 1 | sort | armory, guardroom | a wooden rack of polearms, cudgels and shields |
| `strongbox` | 64 x 64 | 1 x 1 | sort | armory, Orlov's store | an iron-banded strongbox with a heavy lock |
| `workbench` | 96 x 64 | 3 x 1 | sort | Skerrit's workshop | a bone-wright's workbench: saws, drills and pale bone pieces |
| `bone_rack` | 64 x 96 | 2 x 1 | sort | Skerrit's workshop | a rack of cleaned bones sorted by size |
| `copper_still` | 64 x 96 | 2 x 2 | sort | the still-room | a copper still with coiled pipes over a low fire |
| `rag_pile` | 64 x 64 | 2 x 2 | ground | the hovel | heaped rags and sacks on the floor, seen from above |

Reused inside: `crate_stack` (kit), `barrel_stack`, `brazier` and `anvil`.

- **Floors**: homes, shops, the doss house, the hovel and the workshop use a proposed ground, `floorboards`
  (colour `#3a2c22`: dark worn planks, soot in the seams, no shine), a `grounds.json` row for Design and Build; the
  shrines and the Watch, Guild and Kiln buildings keep `flagstone`. Its texture is drawn with the Sunk Ward grounds.
- **Walls**: an indoor room's wall ring is ordinary `#` today, drawn with the location's wall faces (street brick).
  Seen from inside, it should be an interior wall: REVEAL1 should draw an indoor zone's ring with an interior wall
  set, `interior_plaster` (sooty limewash over brick, a skirting of planks; homes and shops) or `interior_stone`
  (dressed dark stone; shrines, Watch, Guild and Kiln buildings), at
  `games/emberwake/assets/tilesets/sunk_streets/walls/<set>/{wall,wall_face}/v<n>.png`.

## 2c. Boundary sets (the screen past a small map)

A map smaller than the screen is centred and the rest of the screen shows the location's boundary art: flat border
tiles, never walkable (REGIONVIEW1; the camera shows one cell past a larger map's edge the same way). The hub, the
Grate, Corvo's Gate and the Slag Gate are smaller than a screen at 1x zoom, so their border is a large part of what is
seen. No Sunk Ward boundary art exists yet. Three sets, chosen per location with its `boundary` field
(REGIONGROUND1_SCENE section 1: a set of its own in the tile set, its own style allowed), at
`games/emberwake/assets/tilesets/sunk_streets/boundary/<set>/{wall,wall_face}/v<n>.png`, drawn after the
REGIONGROUND1 R1 tuning and the user's go-ahead (Design held the Sunk Ward ground batch, AO-04 (1)):

| Set | Locations | What |
| --- | --- | --- |
| `drain_brick` | the hub, the Grate | soot-black storm-drain brick, damp, a few iron rungs and pipe stubs |
| `blind_tenements` | Junction 7, the Crossroads, the Kiln, the Vats, Corvo's Gate | the blind back walls of tenements: soot-black brick, bricked-up windows, drainpipes |
| `gorge_rock` | the Crucible, the Slag Gate | dark gorge rock in tall broken faces, a slag-orange glint in the cracks |

## 3. Prompt sheets (for `PROPS1_PROMPTS.md`)

### Sheet S4 Sunk Ward undercroft, standing (`props_sunk_undercroft`)

4 x 4 cells, row by row (cell: art, PNG size, layer); all `games/emberwake/assets/props/sunk_ward/<art>/v0.png`:

1. `sluice_arch`, 64 x 128, sort
2. `sluice_tunnel`, 64 x 128, sort
3. `drain_arch`, 96 x 128, sort
4. `vault_pillar`, 64 x 160, sort
5. `retort_shelf`, 96 x 96, sort
6. `apothecary_slab`, 96 x 112, sort
7. `herb_rack`, 32 x 112, sort
8. `tinker_hearth`, 96 x 160, sort
9. `anvil`, 32 x 48, sort
10. `scrap_heap`, 64 x 96, sort
11. `brazier`, 32 x 64, sort
12. `kiln_bunk`, 64 x 64, sort
13. `stasis_slab`, 64 x 64, sort
14. `crypt_bars`, 32 x 160, sort
15. `mission_board`, 64 x 96, sort
16. `iron_ladder`, 32 x 128, sort

References, in attach order: `lm_sluice_undercroft.jpg` (style, brick and light), `lm_drowned_runoff_grate.jpg`
(style, iron and water).

```text
References: lm_sluice_undercroft.jpg (the style, the soot-black brick and the warm lamplight), lm_drowned_runoff_grate.jpg (the style, the rusted iron and the black water). A sheet of 16 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art and matching the two reference pictures. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta, pink or purple anywhere in the props. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing, anything above the base arch, flue, shelf or lamp. The props furnish a storm-drain vault under a sunless slum: soot-black brick, rusted iron, dark wood, grey cloth, brass, and warm tallow-yellow fire and lamplight. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: one half of a barred sluice gate arch in a soot-black brick wall, the brick jamb on the left, the arch curving in toward the right at the top, iron bars raised up into the arch (standing, 2 wide by 4 tall); one half of a big sluice tunnel mouth, a deep brick arch on the left side, dark water running out at its foot (standing, 2 wide by 4 tall); a small arched drain mouth in a soot-black brick wall, a thin trickle of black water from it (standing, 3 wide by 4 tall); a squat soot-black brick pier with a stone footing rising straight up (standing, 2 wide by 5 tall).
Row 2: a dark wooden shelf of glass retorts, jars and bone pins with a dripping oil lamp (standing, 3 wide by 3 tall); a stone dissection slab on iron legs with a folded oiled-green cloth, scalpels and pipettes, a lamp hanging above it (standing, 3 wide by 3.5 tall); a narrow drying rack of grey herbs with rock-salt sacks at its foot (standing, 1 wide by 3.5 tall); a brick forge hearth with a glowing tallow-yellow fire, leather bellows at its side and a hooded flue rising straight up (standing, 3 wide by 5 tall).
Row 3: a blackened anvil on a tree stump with a hammer on it (standing, 1 wide by 1.5 tall); a heap of salvaged scrap: bent pipe, valve wheels, chain and cracked brass (standing, 2 wide by 3 tall); an iron brazier on three legs with a low tallow-yellow fire (standing, 1 wide by 2 tall); a bunk set into a brick kiln niche, a straw pallet with a grey blanket and a candle stub (standing, 2 wide by 2 tall).
Row 4: a stone slab with leather restraints under a grey sheet, chalk marks on the stone (standing, 2 wide by 2 tall); a run of rusted iron cell bars in a brick frame, seen running away from the viewer, north to south (standing, 1 wide by 5 tall); a wooden board on two posts hung with blank bone slips and pinned blank scraps (standing, 2 wide by 3 tall); a rusted iron ladder bolted up a brick wall, going up out of the tile (standing, 1 wide by 4 tall).
```

### Sheet S5 Sunk Ward drains, small and ground (`props_sunk_drains`)

3 x 3 cells, row by row:

1. `ring_post`, 32 x 64, sort
2. `lock_tablet`, 32 x 80, sort
3. `lantern_pole`, 32 x 96, sort
4. `culvert_gate`, 32 x 128, sort
5. `varka_skiff`, 64 x 32, ground
6. `drain_grate`, 256 x 128, ground
7. `silt_bank`, 128 x 64, ground
8. `stone_steps`, 96 x 96, ground
9. `ring_post`: spare take (scratch folder), 32 x 64, sort

References, in attach order: `lm_drowned_runoff_grate.jpg` (style, iron, water and silt).

```text
References: lm_drowned_runoff_grate.jpg (the style, the rusted iron grate, the black water and the grey silt). A sheet of 9 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art and matching the reference picture. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta, pink or purple anywhere in the props. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing; a flat prop is a decal lying on the ground or the water seen from above, filling its proportions, with ragged edges and nothing rising from it. The props belong to a drowned drain outlet under a sunless slum: soot-black brick and stone, rusted iron, bronze, grey silt, still black water and warm tallow-yellow lamplight. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a square stone mooring post with an iron ring (standing, 1 wide by 2 tall); an upright bronze flood-lock tablet on a stone plinth, half buried in grey silt, blank (standing, 1 wide by 2.5 tall); a crooked iron pole with a hanging tallow-yellow lantern (standing, 1 wide by 3 tall).
Row 2: a barred iron culvert gate set in a soot-black brick wall, a chain and a padlock (standing, 1 wide by 4 tall); a small flat skiff moored on black water, crates and a lantern on it (flat, 2 wide by 1 deep); a big square iron grate over swirling black water, rust at the bolts (flat, 8 wide by 4 deep).
Row 3: a bank of grey river silt with ripples and the broken edge of a bronze plate (flat, 4 wide by 2 deep); worn stone steps going up toward the top of the tile, soot in the corners (flat, 3 wide by 3 deep); a second take of the square stone mooring post with an iron ring, other chips (standing, 1 wide by 2 tall).
```

### Sheet S6 Sunk Ward streets and walls, standing (`props_sunk_streets`)

4 x 4 cells, row by row: `tenement_block`, `tenement_row`, `watch_armory`, `ash_shack`; `watch_torch`,
`rust_pennant`, `spiked_barricade`, `tenement_block` (a second take); `cage_platform`, `pen_fence`, `toll_pen_gate`, `violet_lamp_post`;
`brick_wall_run`, `brick_breach`, `striped_barrier`, `toll_booth` (sizes and layers in section 2).

References, in attach order: `lm_alleyway_junction_7.jpg` (style, tenements and torchlight),
`lm_ash_crawl_crossroads.jpg` (style, shacks, cages and the violet lamp), `lm_corvos_gate.jpg` (style, warm brick).

```text
References: lm_alleyway_junction_7.jpg (the style, the crooked tenements and the torchlight), lm_ash_crawl_crossroads.jpg (the style, the shacks, the cages and the cold lamp), lm_corvos_gate.jpg (the style and the warm red-brown brick). A sheet of 16 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art and matching the three reference pictures. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta and no pink anywhere in the props, and any violet light leans cold blue. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing, anything above the base roof, chimney, banner or lamp. The props belong to the drowned slums of a sunless city: soot-black brick and timber, slate and tin roofs under grey ash, rusted iron, rust-crimson cloth, red-orange torchlight. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a crooked three-storey tenement of soot-black brick and timber with a slate roof and shuttered windows, two of them lit (standing, 6 wide by 9 tall); a row of narrow two-storey soot-black tenements (standing, 6 wide by 7 tall); a squat brick Watch armory with an iron-bound door, barred windows and a rust-crimson banner (standing, 6 wide by 8 tall); a ramshackle plank shack with a sagging roof of tin sheets under grey ash (standing, 4 wide by 5 tall).
Row 2: an iron torch bracket on a short post with a red-orange flame (standing, 1 wide by 2.5 tall); a pole with a rust-crimson pennant (standing, 1 wide by 3 tall); a spiked timber barricade lashed with chain (standing, 3 wide by 2 tall); a second take of the crooked three-storey tenement, other windows lit, a different roof line (standing, 6 wide by 9 tall).
Row 3: two iron cages on a wooden platform with chains and a titration bench of glass and tubes (standing, 3 wide by 4 tall); a rough fence of iron stakes and timber rails (standing, 3 wide by 2 tall); a chained pen gate with a padlock and a blank tally board (standing, 2 wide by 3 tall); a crooked iron lamp post with a cold blue-violet flame (standing, 1 wide by 3.5 tall).
Row 4: a run of crumbling warm red-brown brick wall with moss in the cracks, its left and right edges cut straight so runs join (standing, 4 wide by 4 tall); the broken end of the same brick wall at a breach, bricks spilling to the right (standing, 2 wide by 4 tall); a red-and-white striped barrier pole lowered on a trestle (standing, 3 wide by 2 tall); a small timber guard booth with a slanted roof and a lantern by its window (standing, 2 wide by 4 tall).
```

### Sheet S7 Sunk Ward industry and fortress, standing (`props_sunk_works`)

4 x 4 cells, row by row: `furnace_mouth`, `conveyor_chute`, `brass_tank`, `pipe_run`; `tallow_vat`, `catwalk_stair`,
`tallow_vat` (a second take), `watch_wall` (a second take); `barrel_stack`, `toll_drum`, `winch_drum`, `crane_arm`; `watch_wall`,
`slit_window_tower`, `cliff_bastion`, `portcullis`.

References, in attach order: `lm_fourth_kiln.jpg` (style, brick, brass and furnace glow), `lm_tallow_vats.jpg` (style,
vats and the Guild's sickly yellow), `lm_crucible_watchpost.jpg` (style, fortress stone and slit windows),
`lm_slag_gate.jpg` (style, cliff masonry and the portcullis).

```text
References: lm_fourth_kiln.jpg (the style, the soot-black brick, the brass and the furnace glow), lm_tallow_vats.jpg (the style, the vats and the sickly-yellow tallow), lm_crucible_watchpost.jpg (the style, the fortress stone and the slit windows), lm_slag_gate.jpg (the style, the cliff masonry and the portcullis). A sheet of 16 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art and matching the four reference pictures. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta and no pink anywhere in the props, and any violet light leans cold blue. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing, anything above the base roof, pipe, battlement or frame. The props belong to the mills, rendering vats and watch fortress of a sunless industrial slum: soot-black brick, dark fortress stone, rusted iron, riveted brass, copper, sickly-yellow tallow, red-orange furnace and window light. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a glowing furnace mouth in a soot-black brick arch, ash on its sill (standing, 3 wide by 3 tall); an inclined wooden conveyor chute on a trestle with bones on the belt (standing, 2 wide by 5 tall); a riveted brass tank with two gauges (standing, 2 wide by 3.5 tall); a run of rusted pipes on brackets with a valve wheel (standing, 2 wide by 3 tall).
Row 2: a big riveted iron vat with a glowing sickly-yellow tallow top and a ladder up its side (standing, 3 wide by 5 tall); a steep iron catwalk stair with a rail (standing, 1 wide by 5 tall); a second take of the big riveted iron vat, its ladder on the other side (standing, 3 wide by 5 tall); a second take of the run of dark fortress wall, its slit window further left, edges cut straight so runs join (standing, 4 wide by 4 tall).
Row 3: tallow barrels stacked two high (standing, 2 wide by 2 tall); an iron-banded toll drum on a stand with a slot in its lid (standing, 1 wide by 1.5 tall); a big iron winch drum in a timber frame, its chain running down (standing, 2 wide by 4 tall); an iron crane arm with a pulley and a hook (standing, 1 wide by 4 tall).
Row 4: a run of dark fortress wall with rust-trimmed battlements and one red-orange slit window, its left and right edges cut straight so runs join (standing, 4 wide by 4 tall); a stone watch tower with a row of red-orange slit windows, battlements and a cold blue-violet glint in its top window (standing, 5 wide by 8 tall); a fortified stretch of dark cliff faced with masonry, a red banner on it (standing, 5 wide by 6 tall); a shut iron portcullis in a stone gate arch (standing, 4 wide by 5 tall).
```

### Sheet S8 Sunk Ward ground decals (`props_sunk_ground`)

3 x 3 cells, row by row: `street_grate`, `violet_pool`, `ash_drift`; `bone_meal_heap`, `wedge_pile`, `tallow_spill`;
`slag_channel`, `ash_drift` spare, `tallow_spill` spare.

References, in attach order: `lm_ash_crawl_crossroads.jpg` (style, ash and the violet pool), `lm_slag_gate.jpg`
(style, the slag glow).

```text
References: lm_ash_crawl_crossroads.jpg (the style, the grey ash and the pool of cold lamplight), lm_slag_gate.jpg (the style and the glowing slag). A sheet of 9 pixel-art ground decals for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art and matching the two reference pictures. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single flat decal lying on the ground, seen straight from above, filling the proportions named for it (width by depth), with ragged edges and nothing rising from it, clear dark outlines, at most 16 colours with 4 shades per colour; no magenta and no pink anywhere in the decals, and any violet light leans cold blue. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a square iron street grate chained shut (flat, 2 wide by 2 deep); a soft pool of cold blue-violet light on ash cobbles (flat, 2 wide by 2 deep); a drift of grey ash and cinders (flat, 3 wide by 2 deep).
Row 2: a heap of pale bone meal (flat, 2 wide by 2 deep); a small pile of black-iron wedges (flat, 1 wide by 1 deep); a spill of pale yellow tallow congealed at the edges (flat, 3 wide by 2 deep).
Row 3: an L-shaped channel of glowing orange slag running down the left side and turning right along the bottom (flat, 4 wide by 8 deep); a second drift of grey ash and cinders, another shape (flat, 3 wide by 2 deep); a second spill of pale yellow tallow, another shape (flat, 3 wide by 2 deep).
```

### Single image S9 the Fourth Kiln (`props_sunk_kiln`)

`fourth_kiln`, 448 x 320, sort: one wide image (16:9), its landmark as the reference, cut to 448 x 320.

```text
References: lm_fourth_kiln.jpg (the building to draw, its style and colours). One pixel-art building for the top-down region map of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art: the bone-mill kiln from the reference picture, seen in the three-quarter top-down view of a tile map (the top and the south face visible), standing on the bottom edge of the image with its base as wide as the drawing, 14 units wide and 10 tall. A long soot-black brick block, three squat chimneys whose tops glow inside with no smoke, iron bands and pipes on its face, a gap at the middle of its south face where a separate furnace mouth will stand. Clear dark outlines, hand-placed pixel clusters, at most 16 colours with 4 shades per colour, light from the upper left. The whole background is flat, solid pure magenta (#FF00FF) with no ground and no cast shadow; no magenta or pink in the building. No text, no letters, no numbers, no runes, no labels, no frame, no watermark.
```

### Sheet S11 Sunk Ward interiors (`props_sunk_interiors`)

4 x 4 cells, row by row: `shop_counter`, `goods_shelf`, `cot`, `table_stools`; `hearth_stove`, `shrine_altar`,
`candle_stand`, `weapon_rack`; `strongbox`, `workbench`, `bone_rack`, `copper_still`; `rag_pile`, `cot` (a second take),
`goods_shelf` (a second take), `candle_stand` (a second take). References, in attach order: `lm_sluice_undercroft.jpg`
(the style, warm lamplight indoors), `lm_fourth_kiln.jpg` (the style, iron and brass).

```text
References: lm_sluice_undercroft.jpg (the style and the warm lamplight), lm_fourth_kiln.jpg (the style, the iron and the brass). A sheet of 16 pixel-art furnishings for the interiors on the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art and matching the two reference pictures. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single piece drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per piece with 4 shades per colour, and light from the upper left; no magenta and no pink anywhere in the pieces. A standing piece stands on the bottom edge of its tile; a flat piece lies on the floor seen from above, with ragged edges. The pieces furnish the poor homes, shops, shrines and Watch rooms of a sunless slum: dark worn wood, grey cloth, rusted iron, tallow candles with warm yellow light. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a worn wooden shop counter with a lamp and a few wares on it (standing, 3 wide by 2 tall); a tall wooden shelf of jars, bundles and tins (standing, 2 wide by 3 tall); a low cot of planks with a grey blanket (standing, 2 wide by 1.5 tall); a rough table with two stools and a candle stub (standing, 2 wide by 2 tall).
Row 2: a small iron stove with a pipe going up and a low warm glow (standing, 1 wide by 2.5 tall); a stone altar with a cloth, a small bone reliquary and candles (standing, 3 wide by 2.5 tall); an iron candle stand with a cluster of tallow candles (standing, 1 wide by 2 tall); a wooden rack of polearms, cudgels and shields (standing, 2 wide by 2.5 tall).
Row 3: an iron-banded strongbox with a heavy lock (standing, 1 wide by 1.5 tall); a bone-wright's workbench with saws, drills and pale bone pieces (standing, 3 wide by 2 tall); a rack of cleaned bones sorted by size (standing, 2 wide by 3 tall); a copper still with coiled pipes over a low fire (standing, 2 wide by 3 tall).
Row 4: heaped rags and sacks lying on the floor (flat, 2 wide by 2 deep); a second take of the low plank cot, the blanket thrown back (standing, 2 wide by 1.5 tall); a second take of the tall shelf, other jars and bundles (standing, 2 wide by 3 tall); a second take of the candle stand, its candles burnt lower (standing, 1 wide by 2 tall).
```

### Single images S10 enterable buildings (`props_sunk_houses`)

One wide image each (16:9), the place's landmark as the reference, cut to the size in section 2b: `doss_house` and
`watch_armory` (`lm_alleyway_junction_7.jpg`), `rag_hovel` (`lm_ash_crawl_crossroads.jpg`), `bonewright_shop`
(`lm_fourth_kiln.jpg`), `still_room` and `guild_store` (`lm_tallow_vats.jpg`), `sentry_hut` and `guardroom`
(`lm_crucible_watchpost.jpg`), and the flavour buildings `ward_house` (three takes), `ward_shop` (two takes) and
`ward_shrine` (`lm_alleyway_junction_7.jpg`). The prompt is the same for each, with the building's line from the table in its place.

```text
References: <the landmark named above> (the style and colours). One pixel-art building for the top-down region map of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art, seen in the three-quarter top-down view of a tile map (the top and the south face visible), standing on the bottom edge of the image with its base as wide as the drawing, <width> units wide and <height> tall: <the building's description from the table>. Its door is in its south face. Clear dark outlines, hand-placed pixel clusters, at most 16 colours with 4 shades per colour, light from the upper left. The whole background is flat, solid pure magenta (#FF00FF) with no ground and no cast shadow; no magenta or pink in the building. No text, no letters, no numbers, no runes, no labels, no frame, no watermark.
```

## 4. Front-on re-run (Tushar, 2026-11-15: "Front-on, re-run those")

The first run (S4-S7, S11, S9) drew about half the props on the landmarks' diagonal, because the landmark pictures
were attached as style references and the model took their angle too. A region map is a square grid seen from the
front and above, so those props are redrawn here. The only reference is a front-on sheet,
`props_buildings_gen1793556098_1.png` (the Ashlands buildings), named for its viewing angle. Kept from the first run:
the ground decals (S8) and the props that read the same from any angle (see `ART_UI_STUDIO.md` 3.10).

View paragraph (opens each front-on prompt):

```text
References: props_buildings_gen1793556098_1.png (the viewing angle only: every building there is seen from the front and slightly above, its front wall flat to the viewer; do not copy its buildings, colours or subject).
```

### Sheet S12 front-on structures (`props_sunk_front_a`)

4 x 4 cells, row by row: `sluice_arch`, `sluice_tunnel`, `drain_arch`, `vault_pillar`; `crypt_bars`, `iron_ladder`,
`furnace_mouth`, `portcullis`; `tenement_block`, `ash_shack`, `toll_booth`, `slit_window_tower`; `cliff_bastion`,
`cage_platform`, `winch_drum`, `catwalk_stair`.

```text
References: props_buildings_gen1793556098_1.png (the viewing angle only: every building there is seen from the front and slightly above, its front wall flat to the viewer; do not copy its buildings, colours or subject). A sheet of 16 pixel-art props for the top-down tile maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF), so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height). VIEW, the same for every prop: seen straight from the front and slightly from above, like an object on a classic top-down JRPG tile map; the front face is flat to the viewer, its bottom edge perfectly horizontal; a thin band of the top face shows above it; every vertical edge is vertical and every horizontal edge is horizontal. Not isometric: no diagonal edges, nothing rotated, no side face visible. Each prop stands on the bottom edge of its tile with its base as wide as the drawing. Clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, light from the upper left; no magenta and no pink anywhere in the props, no glow halos round lights. Materials of a sunless industrial slum: soot-black brick, dark fortress stone, rusted iron, dark timber, tin roofs under grey ash, red-orange torch and furnace light. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: the left half of a barred sluice gate arch in a soot-black brick wall: a brick jamb on the left, the arch curving in toward the right, iron bars raised into the arch (2 wide by 4 tall); the left half of a big sluice tunnel mouth: a deep dark brick arch, dark water at its foot (2 wide by 4 tall); a small arched drain mouth in a soot-black brick wall, a trickle of black water (3 wide by 4 tall); a squat soot-black brick pier with a stone footing (2 wide by 5 tall).
Row 2: a run of rusted iron cell bars in a brick frame, seen from the front (1 wide by 5 tall); a rusted iron ladder bolted flat to a brick wall, going up out of the tile (1 wide by 4 tall); a glowing furnace mouth in a soot-black brick arch, ash on its sill (3 wide by 3 tall); a shut iron portcullis in a stone gate arch (4 wide by 5 tall).
Row 3: a crooked three-storey tenement of soot-black brick and timber, slate roof, shuttered windows, two lit, its door in the front wall (6 wide by 9 tall); a ramshackle plank shack with a sagging tin roof under grey ash, its door in the front wall (4 wide by 5 tall); a small timber guard booth with a slanted roof, a window and a lantern on its front (2 wide by 4 tall); a stone watch tower with a row of red-orange slit windows, battlements and one cold blue-violet window near the top (5 wide by 8 tall).
Row 4: a fortified stretch of dark cliff faced with masonry, a red banner on it, its left and right edges cut straight (5 wide by 6 tall); two iron cages side by side on a low wooden platform with a small bench of glass tubes (3 wide by 4 tall); a big iron winch drum in a timber frame, its axle horizontal and facing the viewer side-on, a chain running down (2 wide by 4 tall); a steep iron stair with a rail, seen from the front, its steps as horizontal bars (1 wide by 5 tall).
```

### Sheet S13 front-on furniture and flat pieces (`props_sunk_front_b`)

4 x 4 cells, row by row: `shop_counter`, `cot`, `table_stools`, `shrine_altar`; `strongbox`, `workbench`,
`apothecary_slab`, `tinker_hearth`; `kiln_bunk`, `stasis_slab`, `striped_barrier`, `pen_fence`; `stone_steps`,
`drain_grate`, `varka_skiff`, `ring_post`.

```text
References: props_buildings_gen1793556098_1.png (the viewing angle only: every building there is seen from the front and slightly above, its front wall flat to the viewer; do not copy its buildings, colours or subject). A sheet of 16 pixel-art props for the top-down tile maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF), so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height). VIEW, the same for every standing prop: seen straight from the front and slightly from above, like furniture on a classic top-down JRPG tile map; the front face is flat to the viewer, its bottom edge perfectly horizontal; the top face shows above it as a horizontal band; every vertical edge is vertical and every horizontal edge is horizontal. Not isometric: no diagonal edges, nothing rotated, no side face visible. A flat piece is seen from straight above as an upright rectangle with horizontal and vertical sides. Clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, light from the upper left; no magenta and no pink anywhere in the props, no glow halos round lights. Materials of a sunless slum: dark worn wood, grey cloth, soot-black brick and stone, rusted iron, bronze, tallow candles. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a worn wooden shop counter with a lamp and a few wares on top (standing, 3 wide by 2 tall); a low plank cot with a grey blanket, its long side facing the viewer (standing, 2 wide by 1.5 tall); a rough table with a stool at each end and a candle stub (standing, 2 wide by 2 tall); a stone altar with a cloth, a small bone reliquary and candles (standing, 3 wide by 2.5 tall).
Row 2: an iron-banded strongbox with a heavy lock on its front (standing, 1 wide by 1.5 tall); a bone-wright's workbench with saws, drills and pale bone pieces on top (standing, 3 wide by 2 tall); a stone dissection slab on iron legs with a folded oiled-green cloth and a lamp hanging over it (standing, 3 wide by 3.5 tall); a brick forge hearth with a glowing fire in its front opening, bellows beside it and a hooded flue going straight up (standing, 3 wide by 5 tall).
Row 3: a bunk set into a brick niche in a wall: a straw pallet, a grey blanket and a candle stub, seen from the front (standing, 2 wide by 2 tall); a stone slab with leather restraints under a grey sheet, its long side facing the viewer (standing, 2 wide by 2 tall); a red-and-white striped barrier pole lowered across, perfectly horizontal, on two trestles (standing, 3 wide by 2 tall); a rough fence of upright iron stakes and two horizontal timber rails (standing, 3 wide by 2 tall).
Row 4: worn stone steps seen from straight above, each step a horizontal band, going up toward the top of the tile (flat, 3 wide by 3 deep); a big iron grate seen from straight above, an upright rectangle of straight bars over swirling black water (flat, 8 wide by 4 deep); a small flat skiff seen from straight above, lying left to right on black water, crates and a lantern in it (flat, 2 wide by 1 deep); a square stone mooring post with an iron ring on its front (standing, 1 wide by 2 tall).
```

### Single image S14 the Fourth Kiln, front-on (`props_sunk_kiln_front`)

`fourth_kiln`, 448 x 320: one wide image (16:9). The first run copied the landmark, angle and ground slab included.

```text
References: props_buildings_gen1793556098_1.png (the viewing angle only: every building there is seen from the front and slightly above, its front wall flat to the viewer; do not copy its buildings, colours or subject). One pixel-art building for the top-down tile map of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art: a bone-mill kiln. VIEW: seen straight from the front and slightly from above; its long front wall is flat to the viewer and its bottom edge is perfectly horizontal, with a band of roof showing above; not isometric, no diagonal edges, no side wall visible, no ground under it. It stands on the bottom edge of the image, 14 units wide and 10 tall: a long soot-black brick block with iron bands and pipes across its front, three squat chimneys in a row along its roof whose tops glow inside with no smoke, a small lit lamp at each end of the front wall, and a plain dark gap in the middle of the front wall at ground level where a separate furnace mouth will stand. Clear dark outlines, hand-placed pixel clusters, at most 16 colours with 4 shades per colour, light from the upper left. The whole background is flat, solid pure magenta (#FF00FF); no magenta or pink in the building. No text, no letters, no numbers, no runes, no labels, no frame, no watermark.
```

## 5. One view, two profiles (Tushar, 2026-11-15)

"The props should probably all be from one angle", "the gates for left and right should probably different and not have
front view", "if we are going for front view we need props in both side profile and front profile based on the
alignment". So every prop is drawn in the one top-down front view, and a prop whose look depends on which wall it
stands against has **two arts**: `<art>` (the front profile: against the north wall or free-standing, facing south)
and `<art>_side` (the side profile: against the west wall, its long axis north to south, facing east; `flip` gives the
east wall). Round and symmetric props (vats, barrels, brazier, posts, lamps, pillars, heaps) need only the one art. The
layout picks the profile from the prop's alignment (`build_layouts.py`).

| Art | PNG px | Footprint | Layer | What |
| --- | --- | --- | --- | --- |
| `crypt_bars_side` | 64 x 128 | 1 x 3 | sort | a run of iron cell bars set in a wall that runs north to south, seen from above: the bar tops in a row |
| `iron_ladder_side` | 48 x 96 | 1 x 2 | sort | an iron ladder fixed to the left wall, seen from its side |
| `culvert_gate_side` | 48 x 96 | 1 x 2 | sort | a barred culvert opening in the left wall, a chain and padlock |
| `barred_gate_side` | 48 x 96 | 1 x 2 | sort | a barred iron gate in the left wall, shut |
| `door_side` | 48 x 96 | 1 x 2 | sort | a plain wooden door in the left wall, in a brick frame |
| `kiln_bunk_side` | 64 x 96 | 1 x 2 | sort | a bunk in a brick niche in the left wall, its long side north to south |
| `cot_side` | 48 x 80 | 1 x 2 | sort | a plank cot along the left wall, head to the north |
| `goods_shelf_side` | 48 x 96 | 1 x 2 | sort | a tall shelf against the left wall, seen from its end and top |
| `retort_shelf_side` | 48 x 128 | 1 x 3 | sort | a shelf of retorts and jars against the left wall |
| `shop_counter_side` | 48 x 112 | 1 x 3 | sort | a shop counter running north to south |
| `workbench_side` | 48 x 112 | 1 x 3 | sort | a workbench running north to south |
| `weapon_rack_side` | 48 x 96 | 1 x 2 | sort | a weapon rack against the left wall |
| `mission_board_side` | 48 x 96 | 1 x 2 | sort | a notice board fixed to the left wall |
| `pen_fence_side` | 32 x 112 | 1 x 3 | sort | a fence of stakes and rails running north to south |
| `striped_barrier_side` | 32 x 112 | 1 x 3 | sort | a striped barrier pole lowered north to south |
| `stone_steps_side` | 96 x 96 | 3 x 3 | ground | stone steps going up toward the left, seen from straight above |

### Sheet S15 side profiles (`props_sunk_side`)

4 x 4 cells, row by row, in the table's order. No reference attached.

```text
A sheet of 16 pixel-art props for the top-down tile maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF), so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows. VIEW, the same for every prop: the camera looks down at a room from the front and above, as on a classic top-down JRPG tile map, and each of these props stands against the LEFT wall of the room, turned to face right. So each prop is tall and narrow in its tile: its long axis runs straight up and down the tile, we look down on its top surface, and we see its right-hand face as a narrow vertical band; its nearer (bottom) end is slightly lower in the tile than its far end is high. Every edge is vertical or horizontal: not isometric, no diagonal edges, nothing rotated. Clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, light from the upper left; no magenta and no pink anywhere in the props, no glow halos round lights. Materials of a sunless slum: soot-black brick, rusted iron, dark worn wood, grey cloth. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a run of iron cell bars set in a brick wall that runs up and down the tile, seen from above as a row of round bar tops between two brick ends (1 wide by 3 tall); an iron ladder fixed to a brick wall on its left, seen from the side as two rails close together with rungs between (1 wide by 2 tall); a barred culvert opening in a brick wall on its left, a chain and padlock across it (1 wide by 2 tall); a shut barred iron gate in a brick wall on its left (1 wide by 2 tall).
Row 2: a plain wooden door in a brick frame in a wall on its left (1 wide by 2 tall); a bunk in a brick niche on its left, a straw pallet and grey blanket lying up and down the tile, a candle stub at its head (1 wide by 2 tall); a plank cot lying up and down the tile, pillow at the top, grey blanket (1 wide by 2 tall); a tall wooden shelf of jars against a wall on its left, seen from above and from its end (1 wide by 2 tall).
Row 3: a long shelf of glass retorts and jars against a wall on its left (1 wide by 3 tall); a wooden shop counter running up and down the tile, a lamp and wares on top (1 wide by 3 tall); a workbench running up and down the tile, saws and bone pieces on top (1 wide by 3 tall); a wooden weapon rack of polearms against a wall on its left (1 wide by 2 tall).
Row 4: a wooden notice board fixed to a wall on its left, blank slips pinned to it (1 wide by 2 tall); a fence of iron stakes and timber rails running up and down the tile (1 wide by 3 tall); a red-and-white striped barrier pole lowered so that it runs up and down the tile, on two trestles (1 wide by 3 tall); worn stone steps seen from straight above, each step a vertical band, going up toward the left of the tile (flat, 3 wide by 3 deep).
```

### Sheet S16 front-on, the rest (`props_sunk_front_c`)

The first run's keepers that still stand at an angle, redrawn front-on so every prop shares one view. 3 x 3 cells, row by
row: `retort_shelf`, `herb_rack`, `anvil`; `scrap_heap`, `mission_board`, `lock_tablet`; `watch_torch`, `hearth_stove`,
`weapon_rack`. Reference: `props_buildings_gen1793556098_1.png`, for the viewing angle only.

```text
References: props_buildings_gen1793556098_1.png (the viewing angle only: every building there is seen from the front and slightly above, its front wall flat to the viewer; do not copy its buildings, colours or subject). A sheet of 9 pixel-art props for the top-down tile maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF), so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height). VIEW, the same for every prop: seen straight from the front and slightly from above, like furniture on a classic top-down JRPG tile map; the front face is flat to the viewer, its bottom edge perfectly horizontal; the top face shows above it as a horizontal band; every vertical edge is vertical and every horizontal edge is horizontal. Not isometric: no diagonal edges, nothing rotated, no side face visible. Each prop stands on the bottom edge of its tile. Clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, light from the upper left; no magenta and no pink anywhere in the props, no glow halos round lights. Materials of a sunless slum: dark worn wood, rusted iron, tarnished brass, bronze, pale bone, grey cloth. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a dark wooden shelf of glass retorts, jars and bone pins with an oil lamp on top, seen from the front (3 wide by 3 tall); a drying rack of grey herbs hung from a bar between two uprights, salt sacks at its foot (2 wide by 3 tall); a blackened anvil on a tree stump, the anvil seen from its side with its horn to the left (2 wide by 2 tall).
Row 2: a heap of salvaged scrap: bent pipe, valve wheels, chain and cracked brass (2 wide by 3 tall); a wooden notice board on two posts hung with blank bone slips and pinned blank scraps (2 wide by 3 tall); an upright bronze flood-lock tablet on a stone plinth, half buried in grey silt, blank (2 wide by 3 tall).
Row 3: an iron torch bracket on a short post with a red-orange flame (2 wide by 2.5 tall); a small iron stove with a door in its front showing a low warm glow, a pipe going straight up (2 wide by 3 tall); a wooden rack holding four polearms upright side by side and a shield (2 wide by 2.5 tall).
```
