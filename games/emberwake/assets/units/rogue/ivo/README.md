# rogue/ivo: Ivo's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-45, accepted by Art/UI 2026-11-22) for the hub cast (Fable second pass, AI-42 item
7): a child, about four fifths of a founder's height: grey knitted cap, blue patched coat, a string of bone-cipher tokens. Drawn from the portrait `portraits/npcs/ivo.png`, keyed and cut to the game's layout. A placeholder
until the finals (decision "Placeholder-first assets"); character looks are not locked.

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (a work loop: flipping and catching a bone token), `hurt` 2 (a startled flinch), `death` 2 (sitting cross-legged).
- `clips.json` marks it `painted: true`. The character row `ivo` still draws its class rig until Build points its art
  here.
