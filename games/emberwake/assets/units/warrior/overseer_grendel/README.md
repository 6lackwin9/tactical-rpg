# warrior/overseer_grendel: Overseer Grendel's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-34, accepted by Art/UI 2026-11-22): Act 1 boss: iron furnace half-helm, white mutton chops, foundry apron and a pneumatic piston-maul. Matches the boss portrait. Keyed,
mirrored to face right (the stage mirrors it for enemies) and cut to the game's layout. A placeholder until the
finals (decision "Placeholder-first assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (maul slam with a piston kick, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `overseer_grendel` still names `warrior/2h_hammer` until
  Build points it here.
