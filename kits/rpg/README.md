# RPG kit

Reusable RPG assets shared by games built on Aegis (ASSETORG2, `docs/specs/ASSETORG2.md`; decision "Asset layers and
reuse groups (user 2026-10-20)"). A game uses the kit by naming it in its `game.json` (`"kit": "rpg"`); the engine
then looks up every asset in the game's `assets/`, this kit's `assets/` and `engine/assets/`, in that order, so a game
overrides a kit file by putting its own at the same path.

Since KIT1-M and KIT1-N (2026-10-25) the kit also holds code and names no game: the genre rules in
`crates/rpgkit_{data,rules,battle,content,world,sim,host}` (the host crate is the battle command set on `aegis_host`) and
their field tables `schema.json`; the lookup chains that pick a body, portrait, icon or battle stage art (`battle_stage_dirs`, `battle_prop_dirs`,
`battle_backdrop`, STAGE1-L) are in `rpgkit_content`'s `art.rs`,
the writer hooks are `rpgkit_content::store::RpgContent`, and the data folder is found by `Data::find_data_dir` or
`find_game_data_dir` with the game's id. A game's crates re-export the kit and add their own code. The sim names no world: a game passes
`SimGame { default_world, story }` (KIT1-SIM). Base-class ids (`warrior`, ...) are the kit's contract. See `docs/specs/KIT1.md`.

`kit.json` is written through `aegis content manifest --kit rpg <json>` (id = this folder's name, a semver version,
the engine range it needs).

What is here (`assets/`):

- `units/<class>/<variant>/sheet.png` + `clips.json`: the six class bodies (`base`), the goblin variants
  (`brawler/goblin`, `rogue/goblin`, `ranger/goblin`, `priest/goblin`, `brawler/goblin_berserker`,
  `rogue/goblin_scout`) and the training dummy.
- `portraits/common/<class>_<variant>.png`: the portraits mobs and minor characters share.
- `items/<family>/base.png` and generic variants (`sword/greatsword`, `accessory/amulet`, `consumable/bomb`, ...);
  `items/any/`.
- `props/common/{tree,stones}/v<n>.png`: the battle stage's upright props (STAGE1-A2, 2026-10-27: generated locally with
  FLUX.2 Klein, sources and prompt records in `art_src/props/`, cut by `tools/sprites/props_cut.py`; STAGE1-T drew them
  by code before).
- `skills/kinds/<kind>.png`, `statuses/<id>.png`, `map/markers/<marker>.png`, `ui/{pins,intents,badges,chips}/`.
- `ui/hud/` (gold, supplies, party, journal, stash, menu), `ui/markers/` (npc, chest, chest_open, lore, exit,
  world_map, encounter, roamer, point) and `ui/services/` (shop, recruit): the PLAYUI1 icons (32x32, made by
  `tools/portraits/sheets.py ui` from `art_src/ui_1.png`).
- `audio/sfx/` (combat sounds) and `audio/music/` (mood music: combat, boss, town, shop, danger, travel).

Placeholder and baseline art comes from the repository's `tools/`, which write the layered tree (ASSETORG2-T):

- `python tools/sprites/export_units.py [body ...] [--clips idle|all]`: the six class bodies (`units/<class>/base/`)
  from the sprite rig, every clip by default (SPRITE1-A2), with the rig's base colours as `clips.json` palette slots;
  the goblin variants (`brawler/goblin`, `brawler/goblin_berserker`, `rogue/goblin`, `rogue/goblin_scout`,
  `ranger/goblin`, `priest/goblin`: the class clips on a shorter goblin body) and the dummy (`dummy/base`: idle,
  hurt, death) from `tools/sprites/variants.py` (SPRITE2).
- `python tools/gen_art.py [group ...] --layer kit [--overwrite]`: common
  portraits, item bases and variants, skill kind icons, statuses, map markers and the UI pins, intents, badges and
  chips. Existing files are kept without `--overwrite` (the portraits and skill and item icons are ART2 art from
  `art_src/`, made by `tools/portraits/`).
- `python tools/gen_music.py --layer kit`: the mood music (numpy and ffmpeg).

To check a regeneration, write it into an empty folder (`--root DIR`) and compare it with the committed tree
(`tools/gen_art.py --compare DIR`). `tools/sprites/export_units.py` always rewrites the bodies it owns (no keep rule).

HOOKS1-S: the combat hook schema (9 rule ids; skill, item, passive, unit, encounter and manifest fields; kinds `hazards`, `unit_meters`, `zones`). Behaviour lands per hook.
HOOKS1-Z: zones as data: the 8 code zones plus `zones` rows (encounter and road-fight `zone` options, `Grid::apply_board(zone, rows, &cat.zones)`, `grid::data_zone`, `art::zone_backdrop`).
HOOKS1-E: events and triggers (rule `triggers`): passive, item and running-skill triggers on 15 events with `when` conditions and `limit`, 12 effects (`meter` acts since HOOKS1-M), held guard, redirect and intercept (one per hit, the running skill first, the holder kept at 1 HP); `[Trigger]` log lines.
RULESET1-G: the Dojo under the timeline ruleset (`rpgkit_battle` `dojo.rs`): a ruleset toggle rebuilding heroes with `build_unit_in` (gear, pouch, points kept), `DojoSession::set_deck`, poise readouts, Rest AP at the round's end, a planner API on a scratch session (plans per intent with AP, estimates and reasons).
HOOKS1-E: events and triggers (rule `triggers`): passive, item and running-skill triggers on 15 events with `when` conditions and `limit`, 12 effects (`meter` waits for HOOKS1-M), held guard, redirect and intercept (one per hit, the running skill first, the holder kept at 1 HP); `[Trigger]` log lines.
HOOKS1-K: riders (rule `triggers`; per unit a hit damaged or the support target, `[Rider]` lines), HP cost (rule `hp_cost`; paid at the command's start, refused at or below the cost, an `HP Cost` cell), guard bypass (rule `ignore_guard`; cover kept, P6 guard factor 1.0); metrics key `triggers` {fired, riders, damage, taken, hp_paid}.
HOOKS1-X: execute (rule `execute`): an `exploit` entry `{hp_below, power_mult}` matches a target at or below that HP share before the hit (best entry wins, nothing consumed); in the hit, the estimate, the AI's combo order and the log cell.
HOOKS1-M: unit meters (rule `unit_meters`): a passive `meter` effect gives a battle-only gauge (start, floor); skill `meters` paid at the command's start (an uncovered cost is refused by the planners and fails unpaid), the trigger effect and condition `meter`, round-end decay, bands with effects and held statuses, crossing rules with `meter_set` (depth 1), slotless meter skills; `[Meter]` lines, `UnitSnap::meters`, `meter::shown`; the band and crossing helper `rpgkit_rules::bands` is shared with the campaign meters.

HOOKS1-F: forced movement (rule `forced_move`): push and pull after stagger, collisions with no random draw, the wall-slam break of a windup, immunity and bonus tiles from passives and gear.

HOOKS1-G: ground hazards (rule `ground_hazards`): placements, the tile overlay, enter and round-end effects, traps, fades, the AI path penalty.
