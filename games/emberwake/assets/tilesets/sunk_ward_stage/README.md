# sunk_ward_stage: the Sunk Ward battle stage set (Style 2 diorama)

Placeholder set (decision "Placeholder-first assets"): cut from the Gemini Style 2 Sunk Ward kit (Art/UI thread,
2026-11-21, items SW-01 to SW-04, SW-16, SW-17 and SW-20 of the Gemini feedback loop). Finals replace these files in place.

The battle stage reads it when the Sunk Ward world names it (`stage_set`, LOOK1 5.4); terrains it does not hold fall
back to `common_stage`. Spec: `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 8.1.

| Folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Continuous cobbles cut from the diorama reference image's own floor (quilted from its prop-free patches, broad lamp light removed, tone set to the cool slate), seamless, with no slab or course on the cell lines so the cell grid does not show in the ground (Tushar 2026-11-22 08:45: "grid lines are clearly available"); replaces SW-17's cell-aligned setts (AI-41 item 5) |
| `stone/` | 64 x 48 | The same reference cobbles from another quilt at the same tone |
| `wall/` | 64 x 48 | Block cap: the flat top of a thick basalt wall seen from above, no baked light or grates (SW-38, AI-42 item 1), average about 25; v0 plain, v1 to v3 cracked, mossy and stained stones blended into v0's edges so any mix tiles |
| `wall_face/` | 64 x 24 | Front face and block face: the wall's plinth |
| `deep_water/` | 64 x 48 | Canal water only, one blue-green hue (SW-04 water band, AI-41), one flattened tile |
| `wall_foot/` | 64 x 16 | Row 0 strip where the backdrop meets the board (D4, drawn once Build adds it) |
| `front_face/<terrain>/` | 64 x 24 | The front face under row 9 per terrain: `floor`, `stone`, `rubble` and `burning_ground` the pale stone quay lip (SW-02 kerb), `deep_water` water, `lava` and `toxic_bog` the stone face with a molten or sludge lip and drips, `bridge` the timber deck's edge |
| `stage.fx.json` | - | Lantern spots per zone backdrop (AI-41 item 4, `PR7-68` R3): top-level `spots` for the default wall, `zones.<zone>.spots` per battle zone; since `PR7-74` (FB-443) also `zones.<zone>.set` (foundry, tallow vats, undercroft, rooftops), `terrain.lava` (foundry slag, boiling tallow, each with a `front_face/`) and `tint` (watchpost `#d4dae4` cool slate, alley `#c8beb4` dim soot) |
| `lava/` | 64 x 48 | Molten channels between cooled basalt (SW-16 band 1), one flattened tile |
| `toxic_bog/` | 64 x 48 | Dark olive sludge with ripples (SW-16 band 2), one flattened tile |
| `burning_ground/` | 64 x 48 | Ward brick with burning seams (SW-16 band 3), one flattened tile |
| `bridge/` | 64 x 48 | Damp timber deck from SW-41's plank grain: 12 planks a cell with joints staggered at random, v0 to v3 different staggers, no iron strap on the cell edge (the old straps drew a grid) |
| `rubble/` | 64 x 48 | The `floor/` tile; the broken basalt is the `props/common/stones` prop on it |
| `cover/` | 64 x 72 | Standing cover props, bottom aligned: v0 roped crates, v1 barrel stack, v2 banded stone pillar, v3 lamp post (SW-20) |
| `floor/over_<terrain>/` | 64 x 48 | Transition pieces m01-m14 (LOOK1 5.3, centred on cell corners): the floor's ragged edge over `lava`, `toxic_bog`, `deep_water`, `burning_ground` and `stone`, a boundary moved up to 9 px with a dark rim on the lower ground and a lit lip on the floor (scratchpad tool after `tools/tiles/compose.py`, larger amplitude for the stage's 64 x 48 art) |
| `<terrain>/sheet.png` | 256 x 192 | Ground master for the composer (`PR7-77` R1): `floor` and `stone` one continuous quilt of the reference cobbles, `bridge` the four decks, `lava`, `toxic_bog`, `deep_water` and `burning_ground` quilted from their four older crops so no cell repeats; placeholder |
| `floor/decal/` | 64 x 48 | Accent marks for the composer (`PR7-77` R1), transparent: v0 a hairline crack, v1 a soft stain, v2 loose grit; drawn by Art/UI, placeholder |
| `lip/<terrain>/` | 64 x 12 | The north lip of a sunken cell (`PR7-77` R2): the stone face with a lit rim falling into shadow and the liquid's light on its lowest rows, for `lava` (molten glow), `deep_water` and `toxic_bog`; placeholder |
