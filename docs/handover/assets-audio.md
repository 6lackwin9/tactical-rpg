# Assets & Audio handover

Written 2026-10-18 by the fourth Assets & Audio session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`.

## Queue (in order)

1. MUSIC2 (proposed to the user, not yet approved): wire the remaining music contexts and make their tracks. Proposal: location `kind` town/fortress -> `town`; ruin/camp -> `danger`; shop panel open -> `shop`; roamer or ambush fights -> `encounter`; tracks `town`, `shop`, `danger`, `boss`, `travel` via `tools/gen_music.py`. See the thread "Menu music track" for his answer.

## Landed on `develop` (this session)

- MUSIC1-A + MUSIC1-L (a836119): five generated loops (`menu_theme`, `ashlands`, `iron_reaches`, `tidewrack`, `combat`) and `MusicDirector` (requests by scene context, data-only fallback chain, 1 s crossfade). Spec `docs/specs/MUSIC1.md` rules 1-7.

## Notes for the next session

- Tracks come from `python3 tools/gen_music.py [key ...]` (numpy + ffmpeg/libvorbis, ~45 s for all five). Each track is a function in that file (notes, chords, instruments); add a function and a `TRACKS` entry for a new one, then a manifest row through a seed like `data/seeds/music1_a.gd`, and set `loop=true` in the new `.ogg.import`.
- A new track needs no code: name it by the chain in MUSIC1 rule 7 (`<region>_<context>`, `<world>_<context>`, `<context>`, or a plain region/world key for field).
- Location kinds today: fortress (warden_keep), ruin, gate, camp, town (gull_harbour only), wild. Only kind `town` maps to the `town` context so far.
- UI/UX owns the volume-slider samples in `settings_view.gd`; they dropped their placeholder menu loop.
- Earlier notes still hold: seeds wrap catalog unit upserts in `begin_system_edit()`; run the gate with `SUITE_TIMEOUT=600 tools/fc.sh gate`; never `pkill -f godot`.
- Branches to delete at the next cleanup (landed): `claude/assets-audio-zspfs7`, `claude/TRAV2-A-supplies`, `claude/project-thread-5tn3pn`.
