# brawler/guild_master_orlov: Guild-Master Orlov's painted frame sheet (placeholder)

Gemini Style 2 frame sheet (item SW-33, accepted by Art/UI 2026-11-22): Act 1 boss: huge, bald and black-bearded, butcher's apron with harness straps, back tank and two flensing hooks on chains. Matches the boss portrait. Keyed,
mirrored to face right (the stage mirrors it for enemies) and cut to the game's layout. A placeholder until the
finals (decision "Placeholder-first assets").

- `sheet.png`: 672 x 480, one clip per row, frames left to right, each frame 112 x 96 (`clips.json` `frame`), feet on
  pixel row 93 (`foot`), facing 3/4 right.
- Rows: `idle` 4, `walk` 6, `attack` 6 (hook throws on chains, impact on frame 2), `hurt` 2, `death` 2.
- `clips.json` marks it `painted: true` (SCREEN_CONTRACT_BATTLE 8.1 D5). The unit row `guild_master_orlov` still names `brawler/base` until
  Build points it here.
