# PLAYUI1: UI icon sheet

Status: approved for one API image by the user (2026-10-22: "Generate via API (1 image)"). Task PLAYUI1-I, spec `PLAYUI1.md` section 6: 22 icons (32 x 32 PNG, transparent, at most 16 colours, readable at 12 to 16 px) plus 3 spare variants, as one 5 x 5 sheet on a flat magenta key. Generate with `tools/portraits/generate.py run <model> ui_1 --file games/emberwake/docs/specs/PLAYUI1_ICONS.md --no-ref`; cut with `tools/portraits/sheets.py ui art_src/ui_1.png ui_1`, which writes each icon to its layer path.

## Sheet ui_1 (`ui_1`)

Icons, row by row (cell: layer path):

1. `lock`: `engine/assets/ui/lock.png`
2. `unlock`: `engine/assets/ui/unlock.png`
3. `delete`: `engine/assets/ui/delete.png`
4. `help`: `engine/assets/ui/help.png`
5. `chevron`: `engine/assets/ui/chevron.png`
6. `gold`: `kits/rpg/assets/ui/hud/gold.png`
7. `supplies`: `kits/rpg/assets/ui/hud/supplies.png`
8. `party`: `kits/rpg/assets/ui/hud/party.png`
9. `journal`: `kits/rpg/assets/ui/hud/journal.png`
10. `stash`: `kits/rpg/assets/ui/hud/stash.png`
11. `menu`: `kits/rpg/assets/ui/hud/menu.png`
12. `npc`: `kits/rpg/assets/ui/markers/npc.png`
13. `chest`: `kits/rpg/assets/ui/markers/chest.png`
14. `chest_open`: `kits/rpg/assets/ui/markers/chest_open.png`
15. `lore`: `kits/rpg/assets/ui/markers/lore.png`
16. `exit`: `kits/rpg/assets/ui/markers/exit.png`
17. `world_map`: `kits/rpg/assets/ui/markers/world_map.png`
18. `encounter`: `kits/rpg/assets/ui/markers/encounter.png`
19. `roamer`: `kits/rpg/assets/ui/markers/roamer.png`
20. `point`: `kits/rpg/assets/ui/markers/point.png`
21. `shop`: `kits/rpg/assets/ui/services/shop.png`
22. `recruit`: `kits/rpg/assets/ui/services/recruit.png`
23. `spare_lock_gold`: spare (scratch folder)
24. `spare_coin`: spare (scratch folder)
25. `spare_scroll`: spare (scratch folder)

```text
A sheet of 25 pixel-art icons for the interface of a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG icons. Square 1:1 image, an exact 5 by 5 grid of 25 equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no shadow on it, so the background can be keyed out. Each icon is one single object drawn large and centred on its tile, filling about three quarters of it, with clear dark outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour, a warm light from the upper left and a thin warm rim light; no magenta, pink or purple anywhere in the icons. The icons are interface glyphs, HUD buttons, map markers and NPC service badges, each a simple bold silhouette that reads at a glance at 16 pixels. No text, no letters, no numbers (the question mark is the only glyph), no labels, no frame, no watermark. Row by row, left to right:
Row 1: a closed iron padlock; an open iron padlock, the shackle swung up and open; a metal trash bin with a lid; a bold question mark inside a round ring; a thick chevron arrow pointing down.
Row 2: a stack of gold coins; a tied supply sack with a loaf of bread and a coil of rope; three helmeted heads side by side, the middle one in front; a closed leather journal with a quill; a wooden chest with iron bands, seen from the front.
Row 3: three thick horizontal bars stacked, light stone colour; the bust of a hooded figure, face in shadow; a closed treasure chest with a gold lock plate; an open treasure chest with a gold glint inside; an open book with a glowing rune on its pages.
Row 4: a stone doorway with a bold arrow pointing out through it; a rolled parchment world map, partly unrolled; two crossed swords; a snarling grey wolf head in profile, prowling; a glowing golden waypoint pin.
Row 5: brass merchant scales over a gold coin; a gauntleted hand clasping a sword hilt; a closed padlock with a gold rim; a single large gold coin; a rolled parchment scroll tied with a red ribbon.
```
