# high_bastion_stage: Act 2 High Bastion battle stage set (Style 2 diorama, mock)

Placeholder set (decision "Placeholder-first assets"), cut by Art/UI 2026-11-22 from the Gemini Act 2 ramparts battle
mock (item SW-48 of the Gemini feedback loop; Design approved the Act 2 mock pick). Finals replace these files in
place. No data names it yet: no Act 2 world exists, so the game never reads this set until a world names it
(`stage_set`) or a zone does (`stage.fx.json` `zones.<zone>.set`). What it does not hold falls back to `common_stage`.

| File or folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Pale grey-violet clerical flagstones with frost-white grout, one stone per cell, four cells evened to one brightness |
| `stone/` | 64 x 48 | The floor darker and mirrored |
| `wall/` | 64 x 48 | Block cap: the flat dark top of a masonry blocker seen from above (three caps from the mock, v3 is v0 turned) |
| `wall_face/` | 64 x 24 | The blocker's dark brick face |
| `front_face/<terrain>/` | 64 x 24 | The crenellated rampart parapet under the last row, for `floor` and `stone` |
| `cover/` | 64 x 72 | Standing cover, bottom aligned: v0 stacked ballista-bolt crate, v1 carved stone merlon |
| `backdrop_ramparts.png` | 384 x 52 | The watch tower far wall (cyan lancet windows, red Clerisy banners, curfew bell). It moves to `backgrounds/battle/<zone>.png` once an Act 2 zone exists |

Region-map reference for the same node (SW-47, Watch Ramparts / Slag Breach) and the mock shots are in project files
`art-ui/act2_mock/`; it is a design reference, not game art.
