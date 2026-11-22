# sunk_ward_stage: the Sunk Ward battle stage set (Style 2 diorama)

Placeholder set (decision "Placeholder-first assets"): cut from the Gemini Style 2 Sunk Ward kit (Art/UI thread,
2026-11-21, items SW-01 to SW-04, SW-16, SW-17 and SW-20 of the Gemini feedback loop). Finals replace these files in place.

The battle stage reads it when the Sunk Ward world names it (`stage_set`, LOOK1 5.4); terrains it does not hold fall
back to `common_stage`. Spec: `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 8.1.

| Folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Squared basalt setts on the cell grid (SW-17), average about 30, four crops that tile without seams |
| `stone/` | 64 x 48 | The setts darker and mirrored |
| `wall/` | 64 x 48 | Block cap: the flat top of a thick basalt wall seen from above, no baked light or grates (SW-38, AI-42 item 1), average about 25; v0 plain, v1 to v3 cracked, mossy and stained stones blended into v0's edges so any mix tiles |
| `wall_face/` | 64 x 24 | Front face and block face: the wall's plinth |
| `deep_water/` | 64 x 48 | Canal water only, one blue-green hue (SW-04 water band, AI-41) |
| `wall_foot/` | 64 x 16 | Row 0 strip where the backdrop meets the board (D4, drawn once Build adds it) |
| `front_face/<terrain>/` | 64 x 24 | The front face under row 9 per terrain (AI-41 item 3, for `PR7-67` R2): `floor` and `stone` the pale stone quay lip (SW-02 kerb), `deep_water` water |
| `stage.fx.json` | - | Lantern spots per zone backdrop (AI-41 item 4, `PR7-68` R3): top-level `spots` for the default wall, `zones.<zone>.spots` per battle zone |
| `lava/` | 64 x 48 | Molten channels between cooled basalt (SW-16 band 1) |
| `toxic_bog/` | 64 x 48 | Dark olive sludge with ripples (SW-16 band 2) |
| `burning_ground/` | 64 x 48 | Ward brick with burning seams (SW-16 band 3) |
| `bridge/` | 64 x 48 | Damp timber plank deck with iron straps (SW-41 rows 1 and 2, AI-42 item 4), average about 40 |
| `rubble/` | 64 x 48 | Broken basalt on the `floor/` setts (SW-41 rows 3 and 4 pasted into the floor crops, edges kept), average about 32; the stones prop is `props/common/stones` |
| `cover/` | 64 x 72 | Standing cover props, bottom aligned: v0 roped crates, v1 barrel stack, v2 banded stone pillar, v3 lamp post (SW-20) |
