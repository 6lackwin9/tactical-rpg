# assets/art/ui

Menu art files (S14): `<key>_icon.png` and `<key>_background.png` (no size check); each needs a row in `data/assets/manifest.json`.

ART1-A keys (ui, `<key>_icon.png`): region map tiles `region_<ash_floor|basalt_wall|water>_<0-2>`, `region_<road|lava|rubble|scrub>_<0-1>`, `region_door_0` (32x32, `.` = ash_floor, `#` = basalt_wall, `~` = water); world map pins `pin_<fortress|town|camp|ruin|gate|wild|here>` (24x24); badges `badge_quest`, `badge_ready` (32x32). Backgrounds (`<key>_background.png`): `ashlands_world_map` (1920x1080), `main_menu` (1920x1080), `ashlands_title` (1280x360), `warden_keep`, `ruined_courtyard`, `cinder_gate`, `ashmaw_camp` (1280x720).

ART2-A keys (ui, `<key>_icon.png`, 32x32): intent kind chips `intent_<attack|area|support|guard|focus|rest|retreat|move|flee|switch|stunned|act|arriving>` (hostile-red rim), `badge_foresight` (gold eye), `chip_slay` (gold crown), `chip_guest` (blue shield). UI wiring comes later.
