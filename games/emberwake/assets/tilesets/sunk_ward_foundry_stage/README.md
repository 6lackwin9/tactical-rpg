# sunk_ward_foundry_stage: the foundry zone's stage set (Style 2 diorama)

Placeholder set (decision "Placeholder-first assets"), cut by Art/UI 2026-11-22 from the Gemini Style 2 Sunk Ward kit
(items SW-39 and SW-40) for Fable's second pass (AI-42 items 2, 3 and 5). Finals replace these files in place.

The battle stage reads it when `sunk_ward_stage/stage.fx.json` names it for the zone (`zones.foundry.set`, and for a
terrain `zones.foundry.terrain.<terrain>`, `PR7-74` DIORAMA4). What it does not hold falls back to `sunk_ward_stage`.

| Folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Riveted cast-iron floor plates with soot and rust (SW-39 top left), average about 31; one base tile `v0` with its broad tone flattened, no alternating variants (Tushar's diorama notes, 2026-11-22) |
| `stone/` | 64 x 48 | Another crop of the floor at the same tone |
| `lava/` | 64 x 48 | Cooling foundry slag: dark crusted plates with thin dull orange cracks (SW-40 right), average about 56; the calmer lava for the foundry zone; one flattened tile |
| `lava/front_face/` | 64 x 24 | The rim under the slag: the world set's stone face with a glowing slag lip and drips; read through `zones.foundry.terrain.lava` |
| `rubble/` | 64 x 48 | The `floor/` tile, so the stones prop sits on the zone's own floor |
| `floor/over_<terrain>/` | 64 x 48 | Transition pieces m01-m14: the floor's ragged edge over `lava` and `stone` |
| `<terrain>/sheet.png` | 256 x 192 | Ground master for the composer (`PR7-77` R1): 4 x 4 cells, ten of the base tile and two each of the older crops, the whole sheet flattened at gain .9 so no tile reads as a second set (liquids: the base tile 4 x 4); placeholder |
| `floor/decal/` | 64 x 48 | Accent marks for the composer (`PR7-77` R1), transparent: v0 a hairline crack, v1 a soft stain, v2 loose grit; drawn by Art/UI, placeholder |
| `lip/lava/` | 64 x 12 | The slag pit's lip (`PR7-77` R2): the world set's lip stone with an orange glow below; placeholder |
