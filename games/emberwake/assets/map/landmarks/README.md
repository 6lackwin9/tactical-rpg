# Act map landmarks

One picture per location, drawn on the act's world map as its own layer over the act background (Outbox `AO-01` in
`games/emberwake/docs/specs/ART_UI_REQUESTS.md`; the game does not read this folder yet, so the kind pin still shows).

- `<location_id>.png`, transparent, square, cut at 96 x the location's map scale (2 pixels per logical pixel of the
  640 x 360 map; the map draws it at 48 x scale logical pixels, its box bottom 8 px below the location's ground point).
- Made with `art_src/prompts/blend_act_map.py codeblend art_src/prompts/act<n>_map_layout.json`: each landmark is
  toned toward the painting around it (its own lights kept bright) and its base darkened a little, while the act's
  background (`backgrounds/world_map/<region>.png`) carries its contact shadow and light spill. The alpha is the cut's
  own, so the selection glow follows a clean outline.
- `sluice_undercroft.png` (the hideout) is shared by every act: untoned, at its largest scale (x1.2).
- Sources, picks, fixes and layouts: `art_src/prompts/README.md`, `ART_UI_STUDIO.md` 3.10. Acts 2 and 3 are here;
  Act 1 follows its slab-removal pass.
