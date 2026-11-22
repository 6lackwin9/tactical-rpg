# warrior/proctor_quell: Proctor Quell's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-32, accepted by Art/UI 2026-11-22): Act 1 boss: bald crown, long dark back hair, blood-spattered surgeon's coat, beak mask at the chest, neck valve and two bone saws. Matches the boss portrait. Keyed,
mirrored to face right (the stage mirrors it for enemies) and cut to the game's layout. A placeholder until the
finals (decision "Placeholder-first assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (twin saw slashes, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `proctor_quell` still names `warrior/2h_sword` until
  Build points it here.
