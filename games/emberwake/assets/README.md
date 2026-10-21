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

Generated art: do not run `tools/gen_art.py` or `tools/sprites/export_units.py` until ASSETORG2-T: they still write the old
`assets/art/` layout, which the layout test refuses. Source images of generated portraits and icons are in `art_src/` (Git LFS).
