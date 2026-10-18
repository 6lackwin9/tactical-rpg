# Assets & Audio handover

Written 2026-10-18 04:20 UTC by the fifth Assets & Audio session ("Fresh sessions" in `docs/workflow/cloud.md`). Read this first, then `docs/workflow/cloud.md`. Dispatch mode is economy (asset-builder low/sonnet, change-reviewer low).

## Queue (in order)

1. INTENT1-A: passive `wardens_eye` on Bryn (`{"type": "foresight"}`, no params). Unblocked (INTENT1-L a56dda7, RECRUIT1-A 40cd863).
2. BOSS1-A: Grakka's phases. Unblocked (BOSS1-L schema 1cb608e, behaviour 5c85e7e).
3. OBJ1-A: spec `docs/specs/OBJ1.md` section 4, after BOSS1-A (same encounter rows). Schema on develop (2c9bc80); runtime lands next. A world guest must be a party character or an npc whose base unit is side party (Wenn as npc + ranger is fine).
4. ITEMUSE1-A after ITEMUSE1-L (Gameplay pings).

Gameplay's session is now session_01HQvENsjr5dFTBZp6L6GkPU; UI/UX is session_018UFG3nVhoGx8TP3mRwKgBz. Tell both the new session id.

## Landed on `develop` (this session)

- ART1-A (98d6da5): generated placeholder art for all of Ashlands, `python3 tools/gen_art.py [group ...]` (Pillow + numpy, ~18 s), seed `data/seeds/art1_a.gd` (scans `assets/art/*`, one manifest row per PNG). Every Ashlands character has its own unit art key = character id (decision "ART1"). Keys and sizes: `docs/specs/ART1.md`. New PNGs need `godot --headless --import` before the seed.
- MAPREAD1-A (092c1d6): marker labels/notes, `map_pos` pins on the world-map art, Ashlands locations on the 48x30 `ashlands_*` map templates. Seed `data/seeds/mapread1_a.gd`.
- RECRUIT1-A (40cd863): Bryn alone at start, `party_max` 3, Toller muster with five recruits, priest/brawler units, early fights resized. Seed `data/seeds/recruit1.gd`.

## Notes for the next session

- UI/UX hookups still pending: marker overlays, quest badges and pins come with their MAPREAD1-L2; `main_menu`, `ashlands_title` and the four location backdrops are unused until UI/UX wires them. Region tiles and the world-map backdrop are already drawn (MAPVIEW1).
- Leftover story text: some Ashlands encounters still list `party` Mara/Tev and the `pass_ambush` intro mentions Tev (RECRUIT1-A review nit); fix in a text pass with the next encounter task (OBJ1-A touches the same rows).
- Seeds that save the world use `WorldCatalog.save_world("ashlands")`, not `save_all` (it normalises other worlds).
- Per-world music: Tushar 02:23: shared tracks are fine until one world has a full campaign.
- Earlier notes hold: music via `tools/gen_music.py`; run the gate with `SUITE_TIMEOUT=600 tools/fc.sh gate`; never `pkill -f godot`.
- Branches to delete at the next cleanup (landed): `claude/project-thread-prl0uj`, `claude/MAPREAD1-A-ashlands-maps`, `claude/RECRUIT1-A-recruits`.
