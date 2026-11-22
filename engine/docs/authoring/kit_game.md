# Starting a game on the RPG kit

Status: canon (KIT-SAMPLE1, ENGINE-42, 2026-11-22). Game-agnostic. The working example is the kit's sample game,
`games/kit_sample/`: copy what it does, in this order, and a new kit game fights on the kit's battle stage on its first
day. `games/hello/` is the same for a game on the engine alone.

## 1. The folder

1. `aegis new game <id>` makes a game on the engine alone: a manifest, a crate with the game and its command line, a
   strings file and a scenario.
2. In `game.json` set `"kit": "rpg"`. The kit's asset library (`kits/rpg/assets/`) then joins the asset layers between
   the game's and the engine's, so its unit bodies, props and ground textures are found with no copy.
3. In the crate's `Cargo.toml` add the kit crates the game uses (the lints refuse an unused one): `rpgkit_battle` (the fight), `rpgkit_world` (the data
   loader), `rpgkit_content` (the content kinds) and `rpgkit_ui` (the battle stage and
   the kit's widgets).
4. In the command line (`src/bin/<id>_cli.rs`) set `type Content = rpgkit_content::store::RpgContent`. From then on
   `<id>_cli content upsert|remove|check|apply` writes `data/` through the kit's schema and checks.

## 2. The data

Write every row with the content commands; the writer refuses a row that adds a problem. The least a fight needs:

| Kind | Rows | Notes |
| --- | --- | --- |
| `skills` | `guard`, `move`, `rest`, `switch_target`, `offhand_jab`, plus an attack such as `strike` | The kit requires the first five; the check names any missing |
| `classes` | one per body kind | `base_shares` within 6-18 per stat, `growth_shares`, a `proficiency` dictionary, a valid `ai_role` (AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT) and an `art` that is a body folder |
| `units` | each side's units | `class_id` and `level` (a class-derived unit), `side`, `art` (a body key such as `warrior/base`) |
| `encounters` | one fight | `zone`, `party`, `enemies`, `max_rounds`, `rng_seed`; optional `rows`, `decor` and the rest |

- Prefix the game's ids (`ks_`, say): the layer check refuses a game's class id named in kit code, so a game's ids must
  not match words the kit uses.
- Terrain rows are optional: list only the types the game changes (each listed row is checked whole); every other
  type takes the kit's code defaults, with no warning.
- `progression/xp_curve.json` is optional; without it the kit's default curve applies.
- `<id>_cli content check` is the whole check. Run it once the rows are in.

## 3. The fight

Load the data with `rpgkit_world::Data::open(<game>/data)`, then start the encounter as the kit's session:
`Session::start(data.cat.clone(), EncounterDef::from_obj(row), &data.cat.units)`. One round of the auto driver is
`auto_draft(script::AUTO_PRESET)`, `confirm_plan()`, `resolve_round()`, `advance_round()`. The fight is over when
`combat_over` is set or the last round has resolved (`!is_planning() && !can_advance_round()`). A player-driven screen
replaces `auto_draft` with the player's picks (`set_round_intent`, `pick`).

## 4. The board

The kit's battle stage (`rpgkit_ui::stage`, [battle_stage.md](battle_stage.md)) draws the fight on the old 640 x 360
grid:

- `Geom::new(View::Stage, &sizes)` from a `Sizes` (or the game's tuning through `StageSizes`).
- `zone_look(art, set, zone)` reads the stage set's `stage.fx.json` once.
- `board_items`, `decor_items` and one `unit_item` per standing unit, sorted by `feet_order`.
- Then each of `passes()` in turn:
  - `Backdrop`: `draw_backdrop`.
  - `Ground`: `draw_ground`, then `draw_light` (from `light_spots` and `lamps_with`) and `draw_exit_gates`.
  - `Face`: `draw_face`.
  - `Feet`: `draw_block`, `draw_prop`, `draw_decor` or the unit's body, in feet order.

The sample's `draw_stage` is that loop in about 50 lines.

A stage set of the game's own is a folder `assets/tilesets/<set>/`. Two files are enough:

- `floor/sheet.png`, a whole number of 64 x 48 cells, so the floor composes into one picture.
- `stage.fx.json`, with one zone's `tint` and `spots`, and a `lights` entry for the decor prop that glows.

Everything else falls back to the kit's library. A world names its set with the manifest's `stage_set`. A catalog
encounter is drawn with whatever set the screen passes.

## 5. The scenario

One scenario: the menu, the board (a shot), a round, to the end (a shot). The screen's `state` reports the fight
(`over`, `outcome`, every unit's HP), so the steps wait on `state.fight.over`. The gate runs it with every other game's
scenarios (`scenarios_all`) and again for byte-identical shots (`shotsexact`).
