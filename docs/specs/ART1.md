# ART1: Ashlands placeholder art (audit 2026-10-18)

Status: approved (user 2026-10-18, decision "ART1 Ashlands placeholder art" in `docs/decisions/content-forge.md`)

Everything Ashlands shows on screen today that is a generated glyph, a coloured square or nothing. All new art is placeholder quality, made by a script (`tools/gen_art.py`) so it can be regenerated or swapped file by file later.

## A. Shows in game as soon as it lands (files behind existing keys, no code)

| # | What | Count | Size |
| --- | --- | --- | --- |
| 1 | Named characters: token + portrait (Mara Voss, Tev Quill, Hest, Skarn, Vex, Nok, Gruk, Bolg, six Ash goblin types, Grakka), and each character's `art` key set | 15 x 2 | 104 / 240 px |
| 2 | Class and unit tokens + portraits (6 classes, goblin units, dummy) replacing the glyph placeholders | ~12 x 2 | 104 / 240 px |
| 3 | Item icons: every catalog item + Ember Brigandine | 32 | 96 px |
| 4 | Skill icons (one per skill family) | up to 57 | 64 px |
| 5 | Status icons | all statuses | 64 px |
| 6 | Battle terrain tiles, 4 variants each | 17 x 4 | 108 px |

## B. Files ready, need a small hookup by UI/UX or Gameplay

| # | What | Notes |
| --- | --- | --- |
| 7 | Region map tileset: ash ground, basalt wall, road, water, lava, rubble, scrub, door | location maps are flat colour squares today |
| 8 | Map markers: NPC, chest closed/open, lore, point, exit (open/locked), encounter, roamer, quest ! and ? badges | MAPREAD1 overlay keys |
| 9 | World map: painted Ashlands backdrop, pins per location kind, "you are here" ring | world map is plain circles today |
| 10 | Location backdrops (Warden Keep, Ruined Courtyard, Cinder Gate, Ashmaw Camp) for dialogue, shop and arrival panels | |
| 11 | Main menu background and an Ashlands title card | |
| 12 | Bigger location map layouts (full-screen maps) | size agreed with UI/UX first |

## Keys (as built)

- Region tiles (ui icon, 32x32): `.` `region_ash_floor_0..2`, `#` `region_basalt_wall_0..2`, `~` `region_water_0..2`; extras `region_road_*`, `region_lava_*`, `region_rubble_*`, `region_scrub_*`, `region_door_0`.
- Markers (tile v0, 32x32): the MAPREAD1 2.1 `overlay_*` keys plus `overlay_chest_open`, `overlay_exit_locked`; badges `badge_quest`, `badge_ready` (ui icon).
- Pins (ui icon, 24x24): `pin_fortress`, `pin_town`, `pin_camp`, `pin_ruin`, `pin_gate`, `pin_wild`, `pin_here`.
- Backgrounds (ui): `ashlands_world_map`, `main_menu` (1920x1080), `ashlands_title` (1280x360), `warden_keep`, `ruined_courtyard`, `cinder_gate`, `ashmaw_camp` (1280x720).
- Map templates (48x30): `ashlands_warden_keep`, `ashlands_ruined_courtyard`, `ashlands_cinder_gate`, `ashlands_ashmaw_camp`; locations switch onto them after UI/UX's full-screen map view lands.
- Hookup owners: UI/UX (region map tiles, markers, pins, world map backdrop, backdrops, menu art); Gameplay adds location `map_pos` with MAPREAD1-L1.
