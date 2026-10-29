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
RULESET1-G: the Dojo under the timeline ruleset (`rpgkit_battle` `dojo.rs`): a ruleset toggle rebuilding heroes with `build_unit_in` (gear, pouch, points kept), `DojoSession::set_deck`, poise readouts, Rest AP at the round's end, a planner API on a scratch session (plans per intent with AP, estimates and reasons). Slice 2: the sim's `feel` mode (`rpgkit_sim` `feel.rs`, `SimGame.feel_encounters`; machine-readable `FEEL_JSON` line) and the Dojo's review fixes (the planner shows its scratch battle's AP, Rest matched by kind).
HOOKS1-E: events and triggers (rule `triggers`): passive, item and running-skill triggers on 15 events with `when` conditions and `limit`, 12 effects (`meter` waits for HOOKS1-M), held guard, redirect and intercept (one per hit, the running skill first, the holder kept at 1 HP); `[Trigger]` log lines.
RULESET1-U slice 1 (kit helpers for the battle screen): `round_intent::would_drop` (the intent-bound buffs an intent change would drop; nothing changes) and `round_intent_plan::rest_preview` / `rest_of` / `RestGain` (a planned round's Rest gain: the AP at round end, the poise under `poise`; the Dojo's planner view uses it too).
RULESET1-U slice 2: the foresight `range` reveals are measured when the round's enemy plan is scheduled (`intent::range_reveals`, `Session::range_reveals`, `Session::measure_reveals`; `intent::intents` / `round_intents` / `intent_of` take them), so the playback never changes a reveal; dialogue `best_stat` builds the members with `progression::build_in` under the played world's deck rule.
RULESET1-U slice 3: `skill::attack_band` (a unit's plain attack band under a ruleset, K6: the battle screen's reach band and "Reach 1-2"; `reach_by` of a plain skill).
RULESET1-U slice 5: `poise::forecast` / `Verdict` / `Forecast` (one hit's stagger, P6-P14 without the P13 roll; `resolve_hits` resolves every hit through it), `round_intent::flank_bonus_from` (a flank from a planned tile), `round_intent::round_summary` / `RoundSummary` (breaks, windup breaks of windup hits, flanks, follow-ups, Rest AP), `command::walk_preview` (a planned walk's tiles as the walk would take them now) on `World::step_from` / `StepTo` (`move_unit`'s step rule).
HOOKS1-K: riders (rule `triggers`; per unit a hit damaged or the support target, `[Rider]` lines), HP cost (rule `hp_cost`; paid at the command's start, refused at or below the cost, an `HP Cost` cell), guard bypass (rule `ignore_guard`; cover kept, P6 guard factor 1.0); metrics key `triggers` {fired, riders, damage, taken, hp_paid}.
HOOKS1-X: execute (rule `execute`): an `exploit` entry `{hp_below, power_mult}` matches a target at or below that HP share before the hit (best entry wins, nothing consumed); in the hit, the estimate, the AI's combo order and the log cell.
HOOKS1-M: unit meters (rule `unit_meters`): a passive `meter` effect gives a battle-only gauge (start, floor); skill `meters` paid at the command's start (an uncovered cost is refused by the planners and fails unpaid), the trigger effect and condition `meter`, round-end decay, bands with effects and held statuses, crossing rules with `meter_set` (depth 1), slotless meter skills; `[Meter]` lines, `UnitSnap::meters`, `meter::shown`; the band and crossing helper `rpgkit_rules::bands` is shared with the campaign meters.

HOOKS1-F: forced movement (rule `forced_move`): push and pull after stagger, collisions with no random draw, the wall-slam break of a windup, immunity and bonus tiles from passives and gear.

HOOKS1-G: ground hazards (rule `ground_hazards`): placements, the tile overlay, enter and round-end effects, traps, fades, the AI path penalty.

HOOKS1-H: scheduled arena hazards (rule `arena_hazards`; `arena.rs`): an encounter's `arena_hazards` strike their cells at the end of their listed blocks and rounds, after the block's actions and before the verdict (damage through guard, not armour; the status roll; stagger through P6; a compass push with flat collision damage under `forced_move`); the round's cells and blocks are told from its start (`Session::arena_tells`, the host `intents` reply's `arena`), PL4 counts a due cell as a threat; `Arena` log cells and the metrics key `arena` {strikes, units: {struck, taken, statuses, pushed, collisions, broken}}. Since HOOKS1-U (user 2026-10-29) a block whose actions already wiped out a side skips its strikes (no damage, cells or draws).

HOOKS1-U: the encounter phase effect `deck` (rule `deck`; `encounter.rs` `Phases::apply_decks`): a fired phase adds its `add` families to each active unit of its ref's deck actives (an auto deck first takes its current slots; with the slots full an added family pushes out the last automatic one, never an authored one, and the content check warns naming it) and replaces the named intents' `order` lists, refilling the active slots, from that unit's next planning (a phase fired mid-round: the next round), once per unit; the phase cell reads "new skills (from the next round)"; nothing without the rule.

HOOKS1-AI: AI tactics (rule `ai_tactics`; `ai.rs`, `world.rs`, `round_intent.rs`): target preferences SOFTEST (lowest armour rating, then HP, then nearest; out of reach this round last) and TOUGHEST (melee foes by max HP first; ties nearest) in the shared target ranking of both AIs (a reachable taunter still first; without the rule they read as ROLE); unit and phase `tactics`: `brace_after_hit` (the AI row `brace` before row 4: Guard when the unit or a friend within 2 tiles lost HP to an attack hit last round, `World::hits`), `hold_line` (PL4 ranks tiles next to a friend, then fewer free neighbours, right after threat; the brace Guard goes to that tile), `flush_cover` (a foe hit in cover scores once more in the classic area aim; under round intent an area opener re-aims at a tile that still holds the first ranked target); PL4's threat adds 1 for a visible harmful ground hazard and for an arena hazard due on the tile. Without the rule every plan is as before.

ENCOUNTER2-HA: the party AI of a hold escape (`ai.rs`, ENCOUNTER2 H6; sims, autoplay and the Balanced draft, both AIs): the keeper (the holder; with it down and `fallback` any, the hero on a hold tile or nearest to one) walks to the nearest free hold tile and Defends there (under `round_intent` the Guard intent with the tile as its target, or the advance toward it; it never leaves the tile); the other heroes plan Withdraw once the gate is open or the keeper reaches a tile with this round's move, and draft by their preset before that. The sim's `enc=all` skips encounters with `stay_behind` or `rooms` and `kit_` ids (S4; `enc=<id>` runs them). Without a hold escape every plan is as before.
ENCOUNTER2-K: the encounter phase effect `kill` and phase groups (`encounter.rs`, `session.rs` `phase_kill`, `world.rs` `World::kill`, `log.rs` `kill_entry`; no rule flag, inert without the keys): `{"type": "kill", "units": [refs], "by": ref, "within": 0}` drops every active, placed, unreserved unit of the refs (never a hero) to 0 HP through the journaled damage path, after the phase's own cell; `within` > 0 keeps only units at most that many tiles from the `by` unit on the field (none without one); `by` resolves to the first placed, unreserved unit of its ref and `within` measures from it; one EVENT cell when the kill took HP (subtype `Kill`, actor the `by` unit else the encounter, "Brute and Guard fall."); fates read Killed (key loot pays as for any kill), morale counts the deaths, no unit gets kill credit (the metrics count the HP as the struck units' `scripted` loss, `BattleMetrics::scripted`, part of the HP balance) and no HOOKS1-E event is raised (`kill`, `ally_downed` come only from actions). A kill takes an `unkillable` unit down too (user-confirmed 2026-10-29; the switch is `encounter::KILL_OVERRIDES_UNKILLABLE`, copied to `Session::kill_past_unkillable`; off: the unit keeps 1 HP and its SPARED cell follows the kill's). Phase key `group`: when a phase fires, every unfired phase of its group drops (both due at once: the first in list order fires). An hp_below trigger may watch a phase-wave ref (P5). Checks (`check/encounter.rs` `kill_errors`): 1-6 refs, each an enemy, a phase-wave ref or a guest, never a hero or a protected guest; `by` "" or a unit of the encounter; `within` 0-11 and above 0 only with a `by`; `group` `[a-z0-9_]{1,24}`; one error per mistake.
