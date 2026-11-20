# Kit passive icons

Generic passive icons by archetype (STATUSVIS1 SV12, Inbox AI-39). One 64 px I2 icon per archetype: a painted object on
the shared slate tile (STYLE_BIBLE section 4), added in code when the sheet is cut, so every icon shares the same tile.
It must read at 16 px, the size under a plate's portrait. A game points each of its passives at one archetype; the kit
names no game.

Status: placeholder set landed 2026-11-19 from the image pipeline run `icons_kit_passives_1` (sheet 2, cut by `art_src/prompts/cut_queue_hub_icons.py`): `<archetype>.png`, 64 px. Generic kit art, so it may be approved as final after review in game.

| Archetype | Object | Covers |
| --- | --- | --- |
| `weapon_drill` | short sword crossed over a war hammer | weapon proficiency |
| `armour_drill` | steel breastplate | armour proficiency |
| `might` | clenched fist in a strap | STR, CONS, forced movement |
| `intellect` | open book with a rune | INT, KNOW, spell lore |
| `perception` | wide-open eye | PERC, reach, foresight range |
| `agility` | winged boot | DEX, evasion, haste on a trigger |
| `poise_up` | planted shield with a gold chevron | max poise, less stagger |
| `resist` | ward amulet with a pale ring | status resistance or immunity |
| `ap_gain` | hourglass with a spark | AP gained or returned |
| `crit_up` | blade tip with a star glint | critical chance or power |
| `heal_up` | cupped hands with a green drop | healing potency, earlier healing |
| `status_potency` | vial with falling drops | status potency, inflicted statuses |
| `plan_sight` | parchment with an eye | sees the enemy round plan |
| `immovable` | iron anchor | cannot be moved, steadfast while guarding |
| `cover_ally` | two overlapping kite shields | covers or guards an ally |
| `last_stand` | cracked heart with a flame | a once-per-battle trigger at low HP |
