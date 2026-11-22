# brawler/calcified_amalgam: the Calcified Amalgam's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-37, accepted by Art/UI 2026-11-22): a hunched mass of fused calcified corpses in
black-grey bone-mortar with skull heads, club arms and a ridge of crystal vertebrae. Keyed, mirrored to face right
(the stage mirrors it for enemies) and cut to the game's layout. A placeholder until the finals (decision
"Placeholder-first assets").

- `sheet.png`: 1344 x 960, one clip per row, frames left to right, each frame 224 x 192 (`clips.json` `frame`, the
  2 x 2 footprint size Design set on 2026-11-22), feet on pixel row 189 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (two-arm slam, impact on frame 2), `hurt` 2, `death` 2 (collapses to rubble).
- `clips.json` marks it `painted: true`; the stage scales painted art by footprint (PR7-65). The unit row
  `calcified_amalgam` still names `brawler/base` until Build points it here.
