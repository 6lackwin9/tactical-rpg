# assets

System: S14. Status: placeholder art stays until the base game is done. Spec: games/emberwake/docs/specs/S14_L1.md. See docs/STRUCTURE.md and games/emberwake/docs/ROADMAP.md.

Purpose: art, audio and font files. Since 2026-10-19 the Aegis engine reads the PNGs directly: `emberwake` loads `games/emberwake/assets/art/<namespace>/<key>_<variant>.png` by key (`games/emberwake/crates/emberwake_gui/src/art.rs`), downsamples it to pixel scale and draws a simple fallback shape when a file is missing. There is no import step and no `.import` file (those were Godot's and stay with the Godot build in `legacy/`).

Folders (file names follow the pattern):

- `art/unit/`, `art/item/`, `art/tile/`, `art/skill/`, `art/status/`, `art/ui/`: `<key>_<variant>.png` (the Godot build also read `.svg`)
- `audio/sfx/`: `<key>.ogg` or `.wav`; `audio/music/`: `<key>.ogg` (the engine plays no audio yet)
- `fonts/`: web fallback fonts of the Godot build; the engine's GUI font is `engine/assets/fonts/`

Manifest: `games/emberwake/data/assets/manifest.json` holds the Godot build's file rows (`res://` paths, aliases); the engine does not read or write it yet. Its replacement comes with the engine's sprite work (the SPRITE1 rig work moves to the engine), so a new art key today is a file at the pattern path, plus a planner question when it needs an alias or a manifest row.

Placeholder art (ART1-A): `python3 tools/gen_art.py [group ...]` (needs Pillow and numpy; deterministic) regenerates every placeholder PNG under `art/` (groups: characters classes items skills statuses terrain overlays region markers pins world_map backdrops menu; `--list` prints them). Swap a file by replacing the PNG at the same path (keep the size).

Pixel-art sprites (SPRITE1-A): the six party classes (warrior, ranger, brawler, priest, rogue, mage) and the named party characters that use them (Mara Voss and Bryn Hale warrior, Tev Quill and Sella Marsh ranger, Dagny Coal brawler, Alda Wren priest, Kit Vane rogue, Oren Dusk mage) now have pixel-art unit tokens and portraits taken from the sprite rig's idle frame (token: the 48 px figure at 2x; portrait: a head-and-shoulders crop at 10x); goblins, bosses and NPCs keep the generated placeholders. The rig and class animations (idle, walk, attack, hurt, death, stances) live in `tools/sprites/` (`rig.py`, `classes.py`); `python3 tools/sprites/export_units.py` writes the token and portrait files and `tools/gen_art.py` skips those keys; the Godot seed `sprite1_a.gd` (on the `sprite1-a-godot` branch with the rest of the unreleased SPRITE1-A work) updated the manifest notes. Animation playback in battle needs sprite strips and is a separate task (SPRITE1-A2 / -L / -U in `docs/TASKS.md`), to be built on the engine (decision "Engine transition (user 2026-10-19)").
