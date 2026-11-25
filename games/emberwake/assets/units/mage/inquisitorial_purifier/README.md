# mage/inquisitorial_purifier: the Inquisitorial Purifier's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-26, accepted by Art/UI 2026-11-21): black-lacquered plate over a warded apron, bird-beak iron mask, brass fuel tank and a pyre nozzle with a violet pilot light. Keyed, mirrored to face right (the stage
mirrors it for enemies) and cut to the game's layout. A placeholder until the finals (decision "Placeholder-first
assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (nozzle burst and sweep, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `inquisitorial_purifier` still names `priest/base` until
  Build points it here.
