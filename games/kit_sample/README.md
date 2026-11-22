# Kit Sample: the RPG kit's sample game

ENGINE-42 (`KIT-SAMPLE1`). The smallest whole game on Aegis and the RPG kit, with no other game's code, data or lore.
It proves the kit stands alone, as `games/hello/` proves the engine does, and it is the place to start a new kit game
(`engine/docs/authoring/kit_game.md`).

| Part | Where | What it shows |
| --- | --- | --- |
| Manifest | `game.json` | `"kit": "rpg"`: the kit's asset library joins the layers (game, then kit, then engine) |
| Data | `data/catalog/` | the 5 skills the kit requires and a basic attack, 2 classes, 4 units and 1 encounter, all written by `kit_sample_cli content upsert` (below) |
| Fight | `crates/kit_sample_game/src/battle.rs` | The encounter as the kit's battle session (`rpgkit_battle`), each round drafted by the kit's auto plan |
| Board | `crates/kit_sample_game/src/screens.rs` | The kit's battle stage (`rpgkit_ui::stage`): its passes, the composed floor, the decor and the light |
| Stage set | `assets/tilesets/ks_stage/` | `floor/sheet.png` (a crop of the kit library's floorboards) and `stage.fx.json` (one zone's tint and light spot, the fire pit's light) |
| Art | the kit library (`kits/rpg/assets/`) | Unit bodies (`warrior/base`, `ranger/base`, `brawler/goblin`, `brawler/goblin_berserker`) and the `fire_pit` prop; all placeholders |
| Command line | `crates/kit_sample_game/src/bin/kit_sample_cli.rs` | `aegis_cli::Game` with the kit's content kinds (`rpgkit_content::store::RpgContent`) |
| Scenario | `scenarios/kit_sample_fight.json` | Menu, Start, the board (shot), a round, to the end (shot), back; in the gate |

## Running it

| Command | Does |
| --- | --- |
| `cargo run -p kit_sample_game --bin kit_sample` | The game in a window |
| `target/debug/kit_sample_cli scenario run all` (from this folder) | Its scenario (the gate runs it, step `scenarios_all`) |
| `target/debug/kit_sample_cli content check` | The kit's whole content check on `data/` |

## How the data was written

Every row went through the kit's writer, so it passed the kit's schema and checks:

```sh
C=target/debug/kit_sample_cli   # run from games/kit_sample
$C content upsert skills '{"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}'
# and guard, move, rest, switch_target, offhand_jab: the skills the kit requires
$C content upsert classes '{"id": "ks_fighter", "name": "Fighter", "art": "warrior", "ai_role": "AGGRESSOR", "base_shares": {...}, "growth_shares": {...}, "proficiency": {...}}'
$C content upsert units '{"id": "ks_guard", "name": "Guard", "side": "party", "class_id": "ks_fighter", "level": 3, "art": "warrior/base"}'
$C content upsert encounters '{"id": "ks_skirmish", "name": "Skirmish by the Fire", "zone": "stone_arena", "party": [...], "enemies": [...], "decor": [{"art": "fire_pit", "x": 6, "y": 0}]}'
```

The ids carry a `ks_` prefix so they never match an id the kit's code might name (the layer check refuses a game's
class id in kit code).
