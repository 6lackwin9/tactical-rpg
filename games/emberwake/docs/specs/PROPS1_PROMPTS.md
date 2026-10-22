# PROPS1: prop sheets

Status: prompts ready, each sheet needs the user's yes (task PROPS1-A2 asks before every run: the prompt, the model, one image). Spec `PROPS1.md` sections 6.2 and 7: 28 files (21 kit, 7 game) and one spare take from three sheets on a flat magenta key. After the user's yes, generate a sheet with `tools/portraits/generate.py run <model> <sheet_id> --file games/emberwake/docs/specs/PROPS1_PROMPTS.md --no-ref`; cut it with `tools/portraits/sheets.py props art_src/<sheet_id>.png <sheet_id> --contact <out.png>`, which keys the magenta out, trims each prop, fits it to its PNG size keeping its aspect (standing props on the bottom edge, flat props centred), cuts it to at most 16 colours with a dark 1 px outline and writes it to its layer path.

## Sheet S1 kit standing props (`props_kit`)

4 x 4 cells, row by row (cell: layer path, PNG size, layer):

1. `tent` v0: `kits/rpg/assets/props/common/tent/v0.png`, 96 x 144, sort
2. `tent` v1: `kits/rpg/assets/props/common/tent/v1.png`, 96 x 144, sort
3. `broken_pillar` v0: `kits/rpg/assets/props/common/broken_pillar/v0.png`, 64 x 112, sort
4. `broken_pillar` v1: `kits/rpg/assets/props/common/broken_pillar/v1.png`, 64 x 112, sort
5. `broken_pillar` v2: `kits/rpg/assets/props/common/broken_pillar/v2.png`, 64 x 112, sort
6. `fallen_column` v0: `kits/rpg/assets/props/common/fallen_column/v0.png`, 96 x 48, sort
7. `rubble_wall` v0: `kits/rpg/assets/props/common/rubble_wall/v0.png`, 128 x 64, sort
8. `boulder` v0: `kits/rpg/assets/props/common/boulder/v0.png`, 64 x 80, sort
9. `boulder` v1: `kits/rpg/assets/props/common/boulder/v1.png`, 64 x 80, sort
10. `rock_spire` v0: `kits/rpg/assets/props/common/rock_spire/v0.png`, 64 x 144, sort
11. `cliff_face` v0: `kits/rpg/assets/props/common/cliff_face/v0.png`, 128 x 64, sort
12. `cliff_face` v1: `kits/rpg/assets/props/common/cliff_face/v1.png`, 128 x 64, sort
13. `signpost` v0: `kits/rpg/assets/props/common/signpost/v0.png`, 32 x 64, sort
14. `banner_pole` v0: `kits/rpg/assets/props/common/banner_pole/v0.png`, 32 x 96, sort
15. `gatepost` v0: `kits/rpg/assets/props/common/gatepost/v0.png`, 32 x 80, sort
16. `crate_stack` v0: `kits/rpg/assets/props/common/crate_stack/v0.png`, 64 x 64, sort

```text
A sheet of 16 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta, pink or purple anywhere in the props. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing, anything above the base roof, chimney, banner or canopy; a flat prop is a decal lying on the ground seen from above, filling its proportions, with ragged edges and nothing rising from it. The props are generic camp, ruin and rock pieces of weathered dark stone, wood and hide. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a hide tent on poles with rope pegs, tan hide (standing, 3 wide by 4.5 tall); the same hide tent on poles with rope pegs, grey hide (standing, 3 wide by 4.5 tall); a broken stone pillar on a square base, rubble at its foot, broken off high (standing, 2 wide by 3.5 tall); a broken stone pillar on a square base, rubble at its foot, broken off at half height (standing, 2 wide by 3.5 tall).
Row 2: a broken stone pillar on a square base, rubble at its foot, broken off low to a stump (standing, 2 wide by 3.5 tall); a toppled stone column lying east to west (standing, 3 wide by 1.5 tall); a low broken wall of fitted stones (standing, 4 wide by 2 tall); a rounded dark rock boulder (standing, 2 wide by 2.5 tall).
Row 3: a split, angular dark rock boulder (standing, 2 wide by 2.5 tall); a tall jagged dark rock spire (standing, 2 wide by 4.5 tall); a south-facing dark rock cliff face with horizontal strata, its left and right edges cut straight so it tiles left to right (standing, 4 wide by 2 tall); a second take of the same cliff face, other cracks, the strata at the same heights so the two tile left to right (standing, 4 wide by 2 tall).
Row 4: a wooden post with two blank arrow boards (standing, 1 wide by 2 tall); a tall wooden pole with a plain red cloth banner (standing, 1 wide by 3 tall); a squat stone gatepost with a cap and a lit torch bracket (standing, 1 wide by 2.5 tall); two wooden crates and a barrel (standing, 2 wide by 2 tall).
```

## Sheet S2 Ashlands buildings (`props_buildings`)

2 x 2 cells, row by row (cell: layer path, PNG size, layer):

1. `keep_hall` v0: `games/emberwake/assets/props/ashlands/keep_hall/v0.png`, 320 x 256, sort
2. `barracks` v0: `games/emberwake/assets/props/ashlands/barracks/v0.png`, 320 x 224, sort
3. `forge` v0: `games/emberwake/assets/props/ashlands/forge/v0.png`, 288 x 272, sort
4. `storehouse` v0: `games/emberwake/assets/props/ashlands/storehouse/v0.png`, 288 x 256, sort

```text
A sheet of 4 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 2 by 2 grid of 4 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta, pink or purple anywhere in the props. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing, anything above the base roof, chimney, banner or canopy; a flat prop is a decal lying on the ground seen from above, filling its proportions, with ragged edges and nothing rising from it. The props are the four buildings of a Warden keep yard in the Ashlands, a land of ash and black basalt: basalt walls, slate roofs, warm ember light, each building filling its tile's width. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a squat basalt keep hall with a slate roof, two short corner towers, red Warden banners with a gold ember sun, a glowing hearth door (standing, 10 wide by 8 tall); a long basalt barracks with a timber and slate roof, shuttered windows, weapon racks in front (standing, 10 wide by 7 tall).
Row 2: a basalt smithy with a tall chimney, an open front with a glowing forge and an anvil (standing, 9 wide by 8.5 tall); a basalt storehouse with wide double doors, crates and sacks in front (standing, 9 wide by 8 tall).
```

## Sheet S3 Ashlands and ground (`props_ashlands`)

3 x 3 cells, row by row (cell: layer path, PNG size, layer):

1. `war_tent` v0: `games/emberwake/assets/props/ashlands/war_tent/v0.png`, 96 x 160, sort
2. `toll_arch` v0: `games/emberwake/assets/props/ashlands/toll_arch/v0.png`, 64 x 192, sort
3. `cistern` v0: `games/emberwake/assets/props/ashlands/cistern/v0.png`, 224 x 160, ground
4. `fire_pit` v0: `kits/rpg/assets/props/common/fire_pit/v0.png`, 32 x 32, ground
5. `cobbles` v0: `kits/rpg/assets/props/common/cobbles/v0.png`, 128 x 96, ground
6. `cobbles` v1: `kits/rpg/assets/props/common/cobbles/v1.png`, 128 x 96, ground
7. `scorch_mark` v0: `kits/rpg/assets/props/common/scorch_mark/v0.png`, 64 x 64, ground
8. `scorch_mark` v1: `kits/rpg/assets/props/common/scorch_mark/v1.png`, 64 x 64, ground
9. `war_tent`: spare take (scratch folder), 96 x 160, sort

```text
A sheet of 9 pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and light from the upper left; no magenta, pink or purple anywhere in the props. A standing prop stands on the bottom edge of its tile, its base as wide as the drawing, anything above the base roof, chimney, banner or canopy; a flat prop is a decal lying on the ground seen from above, filling its proportions, with ragged edges and nothing rising from it. The props are Ashlands pieces (a goblin war camp, a ruined basalt toll arch, a cistern) and flat ground decals of sooty ash country. No text, no letters, no numbers, no runes, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a goblin warlord's large hide tent on bone poles, red goblin pennants, skulls (standing, 3 wide by 5 tall); the surviving half of a collapsed basalt toll arch: one tall column, a broken stub of the arch leaning west from its top (standing, 2 wide by 6 tall); a rectangular stone rim one unit thick around a dark pool of still water, the water 5 by 3 in the middle (flat, 7 wide by 5 deep).
Row 2: a ring of stones around glowing embers (flat, 1 wide by 1 deep); a patch of sooty cobblestones (flat, 4 wide by 3 deep); a second patch of sooty cobblestones, other stones and gaps (flat, 4 wide by 3 deep).
Row 3: burnt ground and ash (flat, 2 wide by 2 deep); a second scorch mark of burnt ground and ash, another shape (flat, 2 wide by 2 deep); a second take of the goblin warlord's war tent: large hide tent, bone poles, red pennants, skulls (standing, 3 wide by 5 tall).
```
