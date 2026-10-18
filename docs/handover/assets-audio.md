# Assets & Audio handover

Written 2026-10-18 by the fourth Assets & Audio session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue (in order)

Each starts when Gameplay pings that its `-L` row landed (Gameplay session cse_01JracEuqBiTxnrgMZMMFE5s messages this thread directly):

1. RECRUIT1-A after RECRUIT1-L.
2. MAPREAD1-A after MAPREAD1-L1.
3. ITEMUSE1-A after ITEMUSE1-L.
4. INTENT1-A after INTENT1-L and RECRUIT1-A.
5. BOSS1-A and OBJ1-A after their `-L` rows (specs approved 519eda1).

## Landed on `develop` (this session)

- MUSIC2-A + MUSIC2-L (5a1f677): `town`, `danger`, `shop`, `travel`, `boss` loops; location kind -> context (`MusicDirector.KIND_CONTEXT`), shop while a shop is open.
- MUSIC1-A + MUSIC1-L (a836119): five generated loops (`menu_theme`, `ashlands`, `iron_reaches`, `tidewrack`, `combat`) and `MusicDirector` (requests by scene context, data-only fallback chain, 1 s crossfade). Spec `docs/specs/MUSIC1.md` rules 1-7.

## Notes for the next session

- Tracks come from `python3 tools/gen_music.py [key ...]` (numpy + ffmpeg/libvorbis, ~45 s for all five). Each track is a function in that file (notes, chords, instruments); add a function and a `TRACKS` entry for a new one, then a manifest row through a seed like `data/seeds/music1_a.gd`, and set `loop=true` in the new `.ogg.import`.
- A new track needs no code: name it by the chain in MUSIC1 rule 7 (`<region>_<context>`, `<world>_<context>`, `<context>`, or a plain region/world key for field).
- Generic `town`/`danger` rows replace the world track at those locations (decision "MUSIC2 tracks"); offered the user per-world town versions (e.g. `ashlands_town`), no answer yet.
- Volume sliders bug (user 02:11): likely cause sent to UI/UX: `apply_volumes` only runs through `AudioDirector.active`, which is null on the Main Menu scene. UI/UX owns the fix and the mute button unless they hand the static change back.
- UI/UX owns the volume-slider samples in `settings_view.gd`; they dropped their placeholder menu loop.
- Earlier notes still hold: seeds wrap catalog unit upserts in `begin_system_edit()`; run the gate with `SUITE_TIMEOUT=600 tools/fc.sh gate`; never `pkill -f godot`.
- Branches to delete at the next cleanup (landed): `claude/assets-audio-zspfs7`, `claude/TRAV2-A-supplies`, `claude/project-thread-5tn3pn`, `claude/MUSIC2-tracks`.
