# assets/art/tile

Terrain tile files (S14): `<key>_<variant>.png|svg` with variant v0-v3 (108x108; a v0 file serves every variant without its own file); each needs a row in `data/assets/manifest.json`.

ART1-A: map marker overlays are tile keys with a v0 file at 32x32: `overlay_<npc|chest|chest_open|lore|exit|exit_locked|encounter|roamer|poi>`.
