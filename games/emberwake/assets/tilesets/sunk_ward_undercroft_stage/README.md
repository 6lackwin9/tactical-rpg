# sunk_ward_undercroft_stage: the undercroft zone's stage set (Style 2 diorama)

Placeholder set (decision "Placeholder-first assets"), cut by Art/UI 2026-11-22 from the Gemini Style 2 Sunk Ward kit
(items SW-39 and SW-40) for Fable's second pass (AI-42 items 2, 3 and 5). Finals replace these files in place.

The battle stage reads it when `sunk_ward_stage/stage.fx.json` names it for the zone (`zones.undercroft.set`, and for a
terrain `zones.undercroft.terrain.<terrain>`, `PR7-74` DIORAMA4). What it does not hold falls back to `sunk_ward_stage`.

| Folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Damp cobbles: the diorama reference's cobble quilt tinted green, nothing on the cell lines (the slabs drew the cell grid) |
| `stone/` | 64 x 48 | Another quilt of the same damp cobbles |
| `rubble/` | 64 x 48 | The `floor/` tile, so the stones prop sits on the zone's own floor |
| `floor/over_<terrain>/` | 64 x 48 | Transition pieces m01-m14: the floor's ragged edge over `stone` |
| `<terrain>/sheet.png` | 256 x 192 | Ground master for the composer (`PR7-77` R1): one continuous quilt of the zone's cobbles (`lava`: quilted from its four older crops); placeholder |
| `floor/decal/` | 64 x 48 | Accent marks for the composer (`PR7-77` R1), transparent: v0 a hairline crack, v1 a soft stain, v2 loose grit; drawn by Art/UI, placeholder |
