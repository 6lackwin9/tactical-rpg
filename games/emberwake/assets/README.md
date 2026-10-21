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
  default. A unit's token is its `idle` clip's first frame; the team rim is drawn by code.
- `portraits/{heroes,npcs,elites,bosses}/<id>.png` unique portraits; `portraits/common/<class>_<variant>.png` shared ones.
- `items/<family>/base.png` and `items/<family>/<icon>.png`; `items/any/{base,<item_type>}.png`.
- `skills/<family>.png`, `skills/kinds/<kind>.png`; `statuses/<id>.png`; `map/markers/<marker>.png`.
- `ui/{pins,intents,badges,chips}/<id>.png`; `backgrounds/{menu,title,world_map,location}/<id>.png`.
- `tilesets/<set>/<tile>/v<n>.png` (set = a world id, or `common`; variants are the files present).
- `audio/sfx/<key>.wav`, `audio/music/<key>.ogg` (played by `aegis_ui::audio`).

Which file an entry shows is decided by the lookup chains in `games/emberwake/crates/emberwake_content/src/art.rs` and
checked by the asset checks A1-A6 (`check/assets.rs`). The Godot-era asset manifest is retired (the folders are the
truth); Godot's copy stays in `legacy/data/assets/`.

## Regenerating

The generators write the layered tree directly (ASSETORG2-T): each file goes to its layer's root (this folder, the
kit's `kits/rpg/assets/` or `engine/assets/`). Python 3.12 with Pillow and numpy (on Windows the full path
`$LOCALAPPDATA/Programs/Python/Python312/python.exe`; the `python` alias is a stub).

- `python tools/gen_art.py [group ...] [--layer engine|kit|game ...] [--root DIR] [--overwrite]`: placeholder art
  (ART1-A). Groups (`--list`): bodies, portraits, items, skills, statuses, terrain, overlays, region, markers, pins,
  chips, world_map, backdrops, menu. Here: named bodies (one-frame `sheet.png` + `clips.json`), unique portraits,
  skill family icons, the named items, the Ashlands tiles and the backgrounds. An existing file is kept unless
  `--overwrite`: the committed portraits and skill and item icons are ART2 art from `art_src/` (Git LFS, made by
  `tools/portraits/`), which the placeholders must not replace.
- `python tools/sprites/export_units.py [class ...] [--root DIR] [--clips idle|all]`: the six class bodies of the kit
  from the sprite rig (`units/<class>/base/`; `idle` = the one-frame 104 px sheet, `all` = every clip at 48 px,
  SPRITE1-A2), with the rig's base colours as palette slots.
- `python tools/gen_music.py [track ...] [--layer kit|game] [--root DIR] [--list]`: the music (numpy and ffmpeg);
  `menu_theme` and the world tracks land here, the mood loops in the kit.
- Check a regeneration without touching the committed files: run `gen_art.py --root DIR` and
  `export_units.py --root DIR` into an empty folder `DIR`, then `gen_art.py --compare DIR [--spot OUT]`. It lists
  missing or extra paths and PNG size differences (pixels may drift with numpy and Pillow, so bytes are not compared
  as a rule) and `--spot` writes committed | regenerated body sheets side by side.
