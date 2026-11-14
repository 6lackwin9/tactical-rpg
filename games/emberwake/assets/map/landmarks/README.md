# Act map landmarks

One picture per location, drawn on the act's world map as its own layer over the act background (Outbox `AO-01` in
`games/emberwake/docs/specs/ART_UI_REQUESTS.md`). **Empty for now**: the asset-tree test
(`emberwake_gui` `region::tests::the_assets_follow_the_layered_tree`, ASSETORG2 section 3) has no `map/landmarks/<id>.png`
pattern yet, so the cuts land with the Build task that reads them (AO-01 asks for the pattern). Until then they are
rebuilt on demand with the command below.

- `<location_id>.png`, transparent, square, cut at 96 x the location's map scale (2 pixels per logical pixel of the
  640 x 360 map; the map draws it at 48 x scale logical pixels, its box bottom 8 px below the location's ground point).
- Made with `art_src/prompts/blend_act_map.py codeblend art_src/prompts/act<n>_map_layout.json`: each landmark is
  toned toward the painting around it (its own lights kept bright) and its base darkened a little, while the act's
  background (`backgrounds/world_map/<region>.png`) carries its contact shadow and light spill. The alpha is the cut's
  own, so the selection glow follows a clean outline.
- `sluice_undercroft.png` (the hideout) is shared by every act: untoned, at its largest scale (x1.2).
- Sources, picks, fixes and layouts: `art_src/prompts/README.md`, `ART_UI_STUDIO.md` 3.10. All three acts are
  ready (Act 1 since 2026-11-14). The hideout ships as the one shared cut above, not Act 1's toned x1.0 copy.
