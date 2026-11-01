# assets

System: S14. Status: placeholder art stays until the base game is done. Spec: games/emberwake/docs/specs/S14_L1.md. See docs/STRUCTURE.md and games/emberwake/docs/ROADMAP.md.

Purpose: the game's art and audio. The Aegis engine reads them over three asset layers (ASSETORG2,
docs/specs/ASSETORG2.md): this folder (the game), the RPG kit's `kits/rpg/assets/` and the engine's `engine/assets/`;
the first layer holding a file wins, so a file here overrides the same path in the kit or engine. Since ASSETORG2-G the
kit-bound files moved to `kits/rpg/assets/` (ASSETORG2-K) and the engine baseline (the common tileset, the interface
sounds) to `engine/assets/` (ASSETORG2-B); what stays here is the game's own: named bodies and portraits, its skill
family icons, the named items' icons, the Ashlands tiles, backgrounds and the world music.

Tree (section 3 of the spec; a test keeps every file on these patterns):

- `units/<class>/<variant>/sheet.png` + `clips.json` (+ `palettes/<id>.json`): bodies; `<variant>` `base` is the class
  default. A unit's token is its `idle` clip's first frame; the team rim is drawn by code. A body with front and back
  views (DIR4) also lists `<clip>_down` / `<clip>_up` clips in `clips.json`, each with `of` (its side clip) and `view`.
  An attack clip has `impact` (FX1-L1) and `strike` (WINDUP1: the first frame after its own wind-up); every
  humanoid body (the warrior's, WINDUP1-R1; the ranger's, brawler's, priest's, rogue's and mage's with their goblin and
  named bodies, R2) adds six rows after today's: `windup`, `windup_heavy`, `windup_buff`, `windup_debuff` (looping
  from `loop_from` 3 after a three-frame intro), `stagger` and `dazed`; every creature body (the crawler and the
  quadruped, kit base and named, R3) adds three: `windup`, `stagger` and `dazed`.
- `portraits/{heroes,npcs,elites,bosses}/<id>.png` unique portraits; `portraits/common/<class>_<variant>.png` shared ones
  (the game layer's `common/` holds the creature bodies' portraits, `beast_render_beast.png` and `crawler_ash_crawler.png`, ART3).
- `items/<family>/base.png` and `items/<family>/<icon>.png`; `items/any/{base,<item_type>}.png`.
- `skills/<family>.png`, `skills/kinds/<kind>.png`; `statuses/<id>.png`; `map/markers/<marker>.png`.
- `ui/{pins,intents,badges,chips}/<id>.png`; `backgrounds/{menu,title,world_map,location,battle,story}/<id>.png` (`story`: vignette card pictures, STORYART1; `tools/gen_art.py` groups `story`, `world_card`, `menu_sunless`, `sunk_backdrops`, `family_icons` draw the Sunk Ward slice's story scenes at 1120 x 472 under a starless Shroud sky (`shroud_sky`, ash in `ASH_COLS`, never white), the `sunk_ward` world card, `menu/main_menu.png` (the sunless main menu, MENU2; the old sunny menu drawing is retired, so the `menu` group now draws only the Ashlands title card), the seven zone backdrops and the scourge and sprayer family icons in the ART2 icon look).
- `tilesets/<set>/<tile>/v<n>.png` (set = a world id, or `common`; variants are the files present).
- `audio/sfx/<key>.wav`, `audio/music/<key>.ogg` (played by `aegis_ui::audio`).
- `ui/theme.json` and `ui/frames/<role>.{png,json}`: the Tallow and Iron theme (LOOK1-G1, spec
  `games/emberwake/docs/specs/LOOK1.md` sections 2-3, read by `aegis_ui::theme`): the colour roles, the faces, the
  layout values and the nine-slice frames (`tools/gen_art.py ui_frames`, exact pixels at scale 2 in the art bible's
  hues; the `ornate` corners are code-drawn until the A6 corner sheet exists, then `--ornate SHEET`). Not content: no
  `aegis content` row. Test `emberwake_gui/tests/theme.rs`.
- `fonts/<file>.ttf` with `<Family>-OFL.txt`: the theme's faces beyond the engine's body face (OpenSans SemiBold in
  `engine/assets/fonts/`). Shipped unmodified under the SIL Open Font License 1.1 (decision "LOOK1 fonts"):

| File | Face | Version | Copyright | Reserved Font Name | Source |
| --- | --- | --- | --- | --- | --- |
| `AlegreyaSC-Bold.ttf` | `display` (headers; old-style digits, so numbers use the body or mono face) | Version 2.003 | Copyright 2011 The Alegreya Project Authors (`https://github.com/huertatipografica/Alegreya`) | none | `https://github.com/google/fonts/tree/main/ofl/alegreyasc` (upstream commit d6aedcbb5a82a09dedb07ab99bb91803db8e02af) |
| `JetBrainsMono-wght.ttf` (upstream `JetBrainsMono[wght].ttf`, variable wght 100-800, default 400) | `mono` (badge numbers) | Version 2.211 | Copyright 2020 The JetBrains Mono Project Authors (`https://github.com/JetBrains/JetBrainsMono`) | none | `https://github.com/google/fonts/tree/main/ofl/jetbrainsmono` (upstream commit 19371302b95d218af43299bce79ddbddd0bc364d) |

The engine's font loader (fontdue) draws a variable font at its default instance, so `JetBrainsMono-wght.ttf` draws
at weight 400 (Regular), not the picked Bold (700), until a static Bold file replaces it.

Which file an entry shows is decided by the lookup chains in `kits/rpg/crates/rpgkit_content/src/art.rs` and
checked by the asset checks A1-A6 (`check/assets.rs`). The Godot-era asset manifest is retired (the folders are the
truth); Godot's copy stays in `legacy/data/assets/`.

## Regenerating

The generators write the layered tree directly (ASSETORG2-T): each file goes to its layer's root (this folder, the
kit's `kits/rpg/assets/` or `engine/assets/`). Python 3.12 with Pillow and numpy (on Windows the full path
`$LOCALAPPDATA/Programs/Python/Python312/python.exe`; the `python` alias is a stub).

- `python tools/gen_art.py [group ...] [--layer engine|kit|game ...] [--root DIR] [--overwrite]`: placeholder art
  (ART1-A). Groups (`--list`): bodies, portraits, items, skills, statuses, terrain, overlays, region, markers, pins,
  chips, world_map, backdrops, menu. Here: unique portraits (no bodies: every body is the rig's since SPRITE2),
  skill family icons, the named items, the Ashlands tiles and the backgrounds. An existing file is kept unless
  `--overwrite`: the committed portraits and skill and item icons are ART2 art from `art_src/` (Git LFS, made by
  `tools/portraits/`), which the placeholders must not replace.
- `python tools/sprites/export_units.py [body ...] [--root DIR] [--clips idle|all] [--review OUT]`: every body on the
  sprite rig (`all`, the default = every clip at 48 px, SPRITE1-A2; `idle` = the old one-frame 104 px sheet): the six
  class bodies of the kit, and from `tools/sprites/variants.py` (SPRITE2) the kit's goblin variants and dummy and,
  here, the named bodies (`warrior/grakka_ashmaw`, `brawler/ash_warchief`, `warrior/brask_toller`,
  `warrior/hest_the_quartermaster`, `warrior/orsa_tallow`, `ranger/wenn_tallow`) and the palette files on the kit's
  goblin bodies (`units/<class>/<goblin variant>/palettes/ash.json` for the Horde mobs, `<character>.json` for the
  named goblins). `--review OUT` writes contact sheets and GIFs for a review page. DIR4: a body whose parts and grip all
  have view drawers (`tools/sprites/views.py`; since DIR4-R3 every body on the humanoid rig: the six class bases, the
  warrior's looks, the goblin and named variants; the dummy and the creatures keep side rows) gets its view rows after
  the side rows;
  the exporter exits when a view clip's frames or ms differ from its side clip (R3), a view adds a colour or a palette
  slot (R4), front and back differ in under 10% of the pixels at 28 px (6.4), or the front and back idle frames leave
  out a palette slot colour the side idle frame shows (DIR4-R2, decision "Front and back views follow the side
  model"); other bodies keep side rows and the exporter prints what they lack; `--review` adds
  `<body>_turnaround.png` (idle, walk, the attack and the wind-up holds), `<body>_walk_4dir.gif`,
  `<body>_<attack>_views.gif` and, once per run, `models.png` (side, front and back of each body). WINDUP1 (spec `games/emberwake/docs/specs/WINDUP1.md` 7.3): every attack clip marks
  its `strike` frame (`rig.strike`, else exit); a body of a class in `classes.WINDUP_CLASSES` (all six classes) has the
  six wind-up rows and a creature body (WINDUP1-R3) `rig.CREATURE_ROWS` only (`windup`, `stagger`, `dazed`), with
  section 4's frames, ms, loop and `loop_from` (X1), its `windup` hold within 15 % of the attack's frame `strike - 1` at 48 px (X2), the four holds, `dazed` and `idle` 6 % apart at
  28 px (X3, side and views), no new colour but the gold motes and no new palette slot (X5); `--review` adds
  `<body>_windup_board.png`, `<body>_windup_strike.gif`, `<body>_windups.gif` and `<body>_stagger_dazed.gif` (X6).
- `python tools/gen_music.py [track ...] [--layer kit|game] [--root DIR] [--list]`: the music (numpy and ffmpeg);
  `menu_theme` and the world tracks land here, the mood loops in the kit.
- Check a regeneration without touching the committed files: run `gen_art.py --root DIR` and
  `export_units.py --root DIR` into an empty folder `DIR`, then `gen_art.py --compare DIR [--spot OUT]`. It lists
  missing or extra paths and PNG size differences (pixels may drift with numpy and Pillow, so bytes are not compared
  as a rule) and `--spot` writes committed | regenerated body sheets side by side.
