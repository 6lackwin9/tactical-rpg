# AI-07, AI-11: keepsake, service and HUD icon sheets (runnable prompt file)

Art/UI batch 2026-11-12. Whole-sheet prompts with no shared style block, the same layout as `ART2_ICONS.md` and
`PLAYUI1_ICONS.md`. Run with `tools/portraits/generate.py run <model> <sheet_id> --file art_src/prompts/ai_icons.md --no-ref`.
Each placeholder is replaced **in place**: same path, same pixel size, so nothing needs rebuilding.

## Sheet keepsakes_1 (`keepsakes_1`)

Inbox AI-07 (2). The six Crucible keepsakes (`CANON.md` 5.2, `CAST1_GEAR.md` 7) in the ART2 item-icon look (a rounded
tile, like `items_1`), 3 by 3. Cut with `python3 art_src/prompts/cut_ai_sheets.py keepsakes_1 <sheet.png> [--pick
kaelen=7,...] [--out <review dir>]`: each cell is reduced to a 32 grid and scaled to 96 x 96 (the ART2 cut), written to
`games/emberwake/assets/items/accessory/keepsake_<founder>.png`. Cells, row by row: `kaelen`,
`vesper`, `maren`, `hollis`, `dunstan`, `elyse`, then spare takes of `kaelen`, `vesper`, `elyse` (keep the better of
each pair).

```text
A sheet of 9 pixel-art item icons for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG icons. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Each tile is one icon on its own rounded-square tile filled with dark slate grey, with a subtle darker vignette. Every icon is a single small personal keepsake drawn large and centred, worn and humble, made of scavenged everyday materials (no gems, no gold jewellery, no magic glow), with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour, a warm light from the upper left and a thin warm amber rim light. Each reads at a glance at small size. No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a palm-sized square of dented iron breastplate hung on a leather thong, a cross shape chiselled out of its centre leaving a rough gouged scar in the metal; a small dented tin whistle on a frayed soot-black cord; a goose-feather quill pen whose nib is stained black with gall ink, beside a tiny cracked ink pot.
Row 2: a brass monocle with a cracked round lens on a short braided cord collar strap; a single broken link of heavy black iron penal chain with a rivet, threaded on a twist of wire; a small round-bellied glass chemist's retort flask with a long bent neck and a crack down its side, a little pale-blue liquid inside.
Row 3: a second take of the gouged iron square on its leather thong, the gouge deeper and rougher; a second take of the dented tin whistle, its cord knotted; a second take of the cracked glass retort, the crack bound with a twist of copper wire.
```

## Sheet services_hud_1 (`services_hud_1`)

Inbox AI-07 (1) and AI-11. Interface glyphs on the PLAYUI1 magenta key, 3 by 3. Cut with
`python3 art_src/prompts/cut_ai_sheets.py services_hud_1 <sheet.png> [--pick alert=8,...] [--out <review dir>]` (the
PLAYUI1 keying, mode reduce and outline): `recipe` to `kits/rpg/assets/ui/services/recipe.png` (32 x 32, the
RecipePane badge, generic: the kit names no game); `scrap`, `tallow`, `alert`, `doom_clock` to
`games/emberwake/assets/ui/hud/<name>.png` at **14 x 14** (the `hud_bar` chip size AI-11 gives; check each still
reads in the contact sheet and in game). Cells, row by
row: `recipe`, `scrap`, `tallow`, `alert`, `doom_clock`, then spare takes of `scrap`, `tallow`, `alert`, `doom_clock`.

```text
A sheet of 9 pixel-art icons for the interface of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG icons. Square 1:1 image, an exact 3 by 3 grid of 9 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no shadow on it, so the background can be keyed out. Each icon is one single object drawn large and centred on its tile, filling about three quarters of it, with thick clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour, a warm light from the upper left and a thin warm rim light; no magenta, pink or purple anywhere in the icons. The icons are HUD chips and an NPC service badge, each a very simple bold silhouette that still reads at 14 pixels. No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:
Row 1: a stone mortar with a pestle standing in it, in front of a small iron anvil (a crafting service badge); a small heap of scrap iron: a bent nail, a cog and a broken blade end; a stub of a fat yellow-white tallow candle with a small orange flame.
Row 2: an iron watch bell with a short red alarm ring around it; a cracked brass clock face with one hand and a drop of red blood at its twelve; a second take of the scrap heap, simpler, mostly one cog and one bent bar.
Row 3: a second take of the tallow stub, a squat block of tallow with a short wick and flame; a second take of the watch bell, swinging, with two short motion lines; a second take of the clock, an hourglass with blood-red sand.
```
