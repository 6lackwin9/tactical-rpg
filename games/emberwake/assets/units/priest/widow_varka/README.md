# priest/widow_varka: Widow Varka's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-46, accepted by Art/UI 2026-11-22) for the hub cast (Fable second pass, AI-42 item
7): grey hair under a small black hat and waxed mourning veil, black mourning dress, leather apron, brass salt-scales. Drawn from the portrait `portraits/npcs/widow_varka.png`, keyed and cut to the game's layout. A placeholder
until the finals (decision "Placeholder-first assets"); character looks are not locked.

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (a work loop: weighing salt on the scales), `hurt` 2 (a startled flinch), `death` 2 (sitting down on a stool).
- `clips.json` marks it `painted: true`. The character row `widow_varka` still draws its class rig until Build points its art
  here.
