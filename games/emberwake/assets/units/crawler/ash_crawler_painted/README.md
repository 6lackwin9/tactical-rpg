# crawler/ash_crawler_painted: the Ash-Crawler's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-22, accepted by Art/UI 2026-11-21): a low quadruped with cracked soot-grey hide, clawed forelimbs, an eyeless head and a yellow-green throat gall. Keyed, mirrored to face right (the stage
mirrors it for enemies) and cut to the game's layout. A placeholder until the finals (decision "Placeholder-first
assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (burrow lunge, snap, caustic retch, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `ash_crawler` still names `crawler/ash_crawler` until
  Build points it here.
- A new folder beside the old rig's `crawler/ash_crawler` (kept for its tests and seeds until the key moves).
