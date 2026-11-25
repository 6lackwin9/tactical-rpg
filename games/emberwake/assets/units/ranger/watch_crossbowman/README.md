# ranger/watch_crossbowman: the Watch Crossbowman's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-19, accepted by Art/UI 2026-11-21): red Watch coat, quilted brown jack, kettle
helm, crossbow. Keyed, mirrored to face right (the stage mirrors it for enemies) and cut to the game's layout. A
placeholder until the finals (decision "Placeholder-first assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (crossbow shot, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row still names `ranger/base` until
  Build points `watch_crossbowman` here. Its portrait shows a green hood; the sheet's red coat is the Watch look.

2026-11-22: values lifted one step (gamma 0.82) with a warm one-pixel rim light on the top-left edge, so the maroon coat reads on the dark stage floor at 1:1 (Fable second pass, AI-42 item 5).
