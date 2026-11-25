# rogue/talia_rusk: Talia Rusk's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-44, accepted by Art/UI 2026-11-22) for the hub cast (Fable second pass, AI-42 item
7): long dark hair, scarred cheek, rust scale-leather jerkin, grey scarf, a cipher scroll and a short cane for her crippled leg (the walk limps). Drawn from the portrait `portraits/npcs/talia_rusk.png`, keyed and cut to the game's layout. A placeholder
until the finals (decision "Placeholder-first assets"); character looks are not locked.

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (a work loop: unrolling and reading the cipher scroll), `hurt` 2 (a startled flinch), `death` 2 (kneeling to rest on the cane).
- `clips.json` marks it `painted: true`. The character row `talia_rusk` still draws its class rig until Build points its art
  here.
