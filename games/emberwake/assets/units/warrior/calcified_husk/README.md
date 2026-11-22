# warrior/calcified_husk: the Calcified Husk's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-27, accepted by Art/UI 2026-11-21): a skinless risen corpse hardened to black bone-mortar with calcite crust, shroud rags and a fused bone club. Keyed, mirrored to face right (the stage
mirrors it for enemies) and cut to the game's layout. A placeholder until the finals (decision "Placeholder-first
assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (slow bone-club slam, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `calcified_husk` still names `brawler/base` until
  Build points it here.
