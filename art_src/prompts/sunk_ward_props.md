# Sunk Ward props: draft list and prompt sheets (AI-12, for Design review)

Drafted by Art/UI 2026-11-14 for Design to approve into `PROPS1.md` section 6.2 and `PROPS1_PROMPTS.md` (decision
AO-04 (2)). Nothing is generated before that approval and the user's go-ahead on the model and the image count
(`P-UI-06`, PRINCIPLES 0.5). Until the art lands, a template draws the kit's `common` art where the art name matches
and a solid block where it does not (`P-UI-04`, placeholder-first).

The arts come from the two draft layouts in `art_src/region_maps/` (the hub and the Grate) and from their act-map
landmarks (`art_src/lm_<location>.jpg`), which are the style and colour reference. Paths:
`games/emberwake/assets/props/sunk_ward/<art>/v<n>.png`, 32 px per tile; a standing prop's picture may be one tile
wider and up to four taller than its footprint (PROPS1 6.1). Style A as in PROPS1 6.1.

## 1. Arts for the hub and the Grate (26 arts: 24 new, 2 reused from the kit)

| Art | PNG px | Footprint | Layer | Where | What |
| --- | --- | --- | --- | --- | --- |
| `sluice_arch` | 64 x 128 | 2 x 1 | sort | hub | one half of the barred sluice gate arch in a soot-black brick wall: brick jamb, the arch curving inward at the top, iron bars raised into the arch; `flip` makes the other half |
| `vault_pillar` | 64 x 160 | 2 x 2 | sort | hub | a squat soot-black brick pier with a stone footing, rising into the vault |
| `retort_shelf` | 96 x 96 | 3 x 1 | sort | hub | a dark wooden shelf of glass retorts, jars and bone pins, a dripping oil lamp |
| `apothecary_slab` | 96 x 112 | 3 x 2 | sort | hub | a stone dissection slab on iron legs, a folded oiled-green cloth, scalpels and pipettes, a hanging lamp |
| `herb_rack` | 32 x 112 | 1 x 2 | sort | hub | a narrow drying rack of grey herbs and rock-salt sacks |
| `tinker_hearth` | 96 x 160 | 3 x 2 | sort | hub | a brick forge hearth with a glowing tallow-yellow fire, bellows, a hooded flue rising into the vault |
| `anvil` | 32 x 48 | 1 x 1 | sort | hub | a blackened anvil on a stump, a hammer on it |
| `scrap_heap` | 64 x 96 | 2 x 2 | sort | hub | a heap of salvaged scrap: bent pipe, valve wheels, chain, cracked brass |
| `brazier` | 32 x 64 | 1 x 1 | sort | hub | an iron brazier on three legs with a low tallow-yellow fire |
| `kiln_bunk` | 64 x 64 | 2 x 1 | sort | hub | a bunk set into a brick kiln niche: straw pallet, grey blanket, a candle stub |
| `stasis_slab` | 64 x 64 | 2 x 1 | sort | hub | a stone slab with leather restraints under a sheet, chalk marks on the stone |
| `crypt_bars` | 32 x 160 | 1 x 3 | sort | hub | a run of rusted iron cell bars in a brick frame, running north to south |
| `mission_board` | 64 x 96 | 2 x 1 | sort | hub | a wooden board on two posts hung with bone-cipher slips and pinned scraps (no letters) |
| `iron_ladder` | 32 x 128 | 1 x 2 | sort | hub | a rusted iron ladder bolted up a brick wall, going out of sight |
| `varka_skiff` | 64 x 32 | 2 x 1 | ground | hub | a small flat skiff moored on black water, crates and a lantern on it, seen from above |
| `crate_stack` | (kit) | 2 x 1 | sort | hub | kit `common` art reused |
| `drain_arch` | 96 x 128 | 3 x 2 | sort | Grate | a small arched drain mouth in a brick wall, a trickle of black water from it |
| `sluice_tunnel` | 64 x 128 | 2 x 1 | sort | Grate | one half of the big sluice tunnel mouth: a deep brick arch, dark water running out; `flip` makes the other half |
| `ring_post` | 32 x 64 | 1 x 1 | sort | both | a square stone mooring post with an iron ring |
| `lock_tablet` | 32 x 80 | 1 x 1 | sort | Grate | an upright bronze flood-lock tablet on a stone plinth, half buried in silt (no letters) |
| `lantern_pole` | 32 x 96 | 1 x 1 | sort | Grate | a crooked iron pole with a hanging tallow-yellow lantern |
| `culvert_gate` | 32 x 128 | 1 x 2 | sort | Grate | a barred iron culvert gate set in a brick wall, a chain and padlock |
| `drain_grate` | 256 x 128 | 8 x 4 | ground | Grate | a big square iron grate over swirling black water, rust at the bolts, seen from above |
| `silt_bank` | 128 x 64 | 4 x 2 | ground | Grate | a bank of grey river silt with ripples and a broken lock-plate edge |
| `stone_steps` | 96 x 96 | 3 x 3 | ground | Grate | worn stone steps going up to the south, soot in the corners, seen from above |
| `rubble_wall` | (kit) | 4 x 1 | sort | Grate | kit `common` art reused |

## 2. Arts for the other Act 1 places (no prompts yet; drawn after their layouts)

From each landmark pick, for the layouts that follow the hub and the Grate:

| Place | Arts (footprint, layer) |
| --- | --- |
| 02 Alleyway Junction 7 | `tenement_front` (4 x 2, sort), `spiked_barricade` (3 x 1, sort), `watch_torch` (1 x 1, sort), `barred_gate` (2 x 1, sort), `rust_pennant` (1 x 1, sort) |
| 03 Ash-Crawl Crossroads | `cage_platform` (3 x 2, sort), `toll_pen_gate` (2 x 1, sort), `violet_lamp_post` (1 x 1, sort), `ash_shack` (3 x 2, sort), `ash_drift` (3 x 2, ground) |
| 04 The Fourth Kiln | `kiln_chimney` (2 x 2, sort), `furnace_mouth` (3 x 2, sort), `conveyor_chute` (2 x 3, sort), `bone_meal_heap` (2 x 2, ground), `brass_tank` (2 x 2, sort) |
| 05 The Tallow Vats | `tallow_vat` (3 x 3, sort), `catwalk_stair` (1 x 3, sort), `barrel_stack` (2 x 1, sort), `tallow_spill` (3 x 2, ground) |
| 06 Crucible Watchpost | `watch_wall` (4 x 1, sort), `winch_drum` (2 x 2, sort), `crane_arm` (1 x 1, sort), `slit_window_tower` (2 x 2, sort) |
| 07 The Slag Gate | `portcullis` (3 x 1, sort), `slag_channel` (1 x 4, ground), `cliff_bastion` (4 x 2, sort) |
| 1d Corvo's Gate | `toll_booth` (2 x 2, sort), `striped_barrier` (3 x 1, sort), `toll_drum` (1 x 1, sort), `brick_breach` (3 x 1, sort) |

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
