# sunk_ward_tallow_vats_stage: the tallow vats zone's stage set (Style 2 diorama)

Placeholder set (decision "Placeholder-first assets"), cut by Art/UI 2026-11-22 from the Gemini Style 2 Sunk Ward kit
(items SW-39 and SW-40) for Fable's second pass (AI-42 items 2, 3 and 5). Finals replace these files in place.

The battle stage reads it when `sunk_ward_stage/stage.fx.json` names it for the zone (`zones.tallow_vats.set`, and for a
terrain `zones.tallow_vats.terrain.<terrain>`, `PR7-74` DIORAMA4). What it does not hold falls back to `sunk_ward_stage`.

| Folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Greased wooden boards with pale wax drips (SW-39 top right), average about 36; one base tile `v0` with its broad tone flattened, no alternating variants (Tushar's diorama notes, 2026-11-22) |
| `stone/` | 64 x 48 | Another crop of the floor at the same tone |
| `lava/` | 64 x 48 | Boiling tallow: pale rendered fat with dull amber depths and slow bubbles (SW-40 left), average about 120; the vats zone lava; one flattened tile |
| `lava/front_face/` | 64 x 24 | The vat rim under the tallow: the world set's stone face with a pale tallow lip and a few drips; read through `zones.tallow_vats.terrain.lava` |
| `rubble/` | 64 x 48 | The `floor/` tile, so the stones prop sits on the zone's own floor |
| `floor/over_<terrain>/` | 64 x 48 | Transition pieces m01-m14: the floor's ragged edge over `lava`, `stone` and `toxic_bog` |
