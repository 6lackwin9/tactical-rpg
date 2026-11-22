# mantle_fissure_stage: Act 3 Mantle Fissure battle stage set (Style 2 diorama, mock)

Placeholder set (decision "Placeholder-first assets"), cut by Art/UI 2026-11-22 from the Gemini Act 3 chasm battle
mock (item SW-50 of the Gemini feedback loop, node 17 the Weeping Basalt Chasm; Design approved the pick). Finals
replace these files in place. No data names it yet: no Act 3 world exists, so the game never reads this set until a
world names it (`stage_set`) or a zone does (`stage.fx.json` `zones.<zone>.set`). What it does not hold falls back to
`common_stage`.

| File or folder | Art size | What |
| --- | --- | --- |
| `floor/` | 64 x 48 | Wet black-green basalt ledge slabs, one slab per cell with seep lines in the joints, four cells evened to one brightness |
| `stone/` | 64 x 48 | The floor darker and mirrored |
| `wall/` | 64 x 48 | Block cap: rough chasm-wall rock (two crops, v2 and v3 mirrored) |
| `wall_face/` | 64 x 24 | The rock's dark lower face |
| `front_face/<terrain>/` | 64 x 24 | The ledge lip of basalt column tops over the drop, for `floor` and `stone` |
| `cover/` | 64 x 72 | Standing cover, bottom aligned: v0 crate of survey gauges, v1 and v2 ossified rib stubs |
| `backdrop_chasm.png` | 384 x 52 | The chasm cliff: bone-ivory rib arches, the broken verdigris survey scaffold with gauges, pale green lanterns. It moves to `backgrounds/battle/<zone>.png` once an Act 3 zone exists |

Mock shots are in project files `art-ui/act3_mock/`.
