# warrior/kaelen: Kaelen's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-06 v3, consistency pass on all 20 frames, accepted by Art/UI 2026-11-21), keyed and
cut to the game's layout. A placeholder until the Flow finals (decision "Placeholder-first assets"); character looks
are not locked.

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (notched billhook chop, one-handed with the shield, impact on frame 2), `hurt` 2, `death` 2 (knocked down).
- `clips.json` marks it `painted: true`: the stage draws it as is (no palette swap) and every clip it lacks falls
  back to `idle` (SCREEN_CONTRACT_BATTLE 8.1 D5, row PR7-65). The unit row still names `warrior/2h_axe` until Build points
  `founder_kaelen` here.
