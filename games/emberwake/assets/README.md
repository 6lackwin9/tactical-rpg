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
  named goblins). `--review OUT` writes contact sheets and GIFs for a review page.
- `python tools/gen_music.py [track ...] [--layer kit|game] [--root DIR] [--list]`: the music (numpy and ffmpeg);
  `menu_theme` and the world tracks land here, the mood loops in the kit.
- Check a regeneration without touching the committed files: run `gen_art.py --root DIR` and
  `export_units.py --root DIR` into an empty folder `DIR`, then `gen_art.py --compare DIR [--spot OUT]`. It lists
  missing or extra paths and PNG size differences (pixels may drift with numpy and Pillow, so bytes are not compared
  as a rule) and `--spot` writes committed | regenerated body sheets side by side.
