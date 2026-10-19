# assets/art/tile

Terrain tile files (S14): `<key>/v<n>.png` with n 0-3 (108x108), one folder per terrain (ASSETORG1); the engine finds it by path (`data/assets/manifest.json` is the Godot-era index, not read by the engine).

ART1-A: map marker overlays are tile keys with a v0 file at 32x32 (`overlay_<kind>/v0.png`): `overlay_<npc|chest|chest_open|lore|exit|exit_locked|encounter|roamer|poi>`.
