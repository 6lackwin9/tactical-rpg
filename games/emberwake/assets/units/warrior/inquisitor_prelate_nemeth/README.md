# warrior/inquisitor_prelate_nemeth: the Inquisitor-Prelate Nemeth's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-30, accepted by Art/UI 2026-11-21): Act 1 boss: black-lacquered plate over a crimson cassock, gorget with a violet phial, ash-grey skin, long white hair and a long two-handed blade. Keyed, mirrored to face right (the stage
mirrors it for enemies) and cut to the game's layout. A placeholder until the finals (decision "Placeholder-first
assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (heavy diagonal and rising cuts, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `inquisitor_prelate_nemeth` still names `warrior/2h_sword` until
  Build points it here.
