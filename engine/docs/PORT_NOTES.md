# Port Notes (Godot -> Aegis)

Working log for the combat port. One row per Godot file: where it went in Rust and how it is checked.
Pass bar: statistical parity (ENGINE_SPEC section 6). Dev tool: seeded battles replay step for step,
because the RNG is bit-exact.

## Godot behaviours to reproduce exactly

| Godot behaviour | Rust |
| :--- | :--- |
| `RandomNumberGenerator` (PCG32, two-draw `randf`, bounded `randi_range`) | `aegis_core::rng::GodotRng` (tested against exported draws) |
| `roundf` / `roundi` round half away from zero | `f64::round` (`stats::roundi`) |
| `Array.sort_custom` is Godot's `SortArray` introsort (insertion sort at 16 or fewer, so stable there; unstable above) | `aegis_core::sort::godot_sort` (port of `SortArray`) |
| Dictionary iteration is insertion order | `IndexMap` everywhere order can matter |
| JSON numbers come back as floats; `x is int` checks fail on parsed data | lenient `aegis_core::value` reads; checked by parity |
| Integer division truncates toward zero, `floori` floors | match per call site |
| A float total starts at 0.0 (Rust's `Iterator::sum` of no floats is -0.0, which prints "-0.0") | `fold(0.0, ...)` where a total is printed |

## Files

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `core/specs/stat_rules.gd`, `character_stats.gd`, `proficiency.gd`, `stat_modifier.gd` | `emberwake_rules::stats` | unit snapshots |
| `core/inventory/equipment.gd`, `inventory.gd` | `emberwake_rules::gear` | unit snapshots |
| `core/combat/combatant.gd` (build, skills, unlocks) | `emberwake_rules::unit` | unit snapshots |
| `content/content_factory.gd` (`unit`, `loadout_of`, `_equip`) | `emberwake_rules::unit::build_unit` | unit snapshots |
| `core/specs/passives.gd` (`apply_effects`) | `Unit::apply_effects` | unit snapshots (gear effects) |
| `core/combat/damage_calculator.gd` | `emberwake_rules::damage` | damage cases |
| `combat/status/status_effects.gd`, `status_registry.gd` | `emberwake_rules::status` | battle traces |
| `combat/arena/combat_grid.gd`, `terrain.gd`, `combat_zones.gd` | `emberwake_battle::grid`, `world` | battle traces |
| `combat/arena/grid_path.gd` | `emberwake_battle::path` | battle traces |
| `combat/arena/cover_rules.gd`, `hazard_rules.gd`, `status/riposte_rules.gd`, `status/support_rules.gd` | `emberwake_battle::command` | battle traces |
| `core/combat/battle_command.gd`, `combat/actions/*` (with `item_command.gd`) | `emberwake_battle::command` | battle traces, item traces |
| `combat/arena/combat_resolver.gd` | `emberwake_battle::resolver` | battle traces |
| `combat/planning/combat_plan.gd`, `plan_step.gd`, `plan_entry.gd` (no options / UI probes yet) | `emberwake_battle::plan` | battle traces |
| `combat/planning/skill_def.gd`, `skill_catalog.gd`, `content/skill_kinds.gd` | `emberwake_battle::skill` | battle traces |
| `combat/ai/tactical_ai.gd` (with `_try_item`), `ai_profile.gd`, `plan_presets.gd` | `emberwake_battle::ai`, `world` | battle traces, item traces |
| `combat/session/combat_session.gd`, `catalog_encounter.gd`, `combat_encounter.gd` | `emberwake_battle::session` | battle traces, world traces |
| `combat/session/encounter_objectives.gd`, `encounter_phases.gd`, `encounter_morale.gd`, `world_encounter.gd` (guests, start buffs) | `emberwake_battle::encounter`, `session`; `emberwake_world::battle::WorldRoster` | world traces |
| `tools/sim/sim_runner.gd` (`fight`, `apply_pouch`), `sim_variants.gd` (jitter) | `emberwake_battle::sim` | battle traces |

## Parity status (2026-10-19)

- Unit snapshots: 102 rows exact. Damage: 18,216 cases exact. RNG: 180 draws exact.
- Battle traces: 488 balance-sim fights replay step for step: 360 duels (6 classes, levels 1 / 5 / 10, 2 seeds,
  both sides, jitter on), 72 group fights (hero vs 3 minion / standard units) and 56 team fights (mixed trios).
  Compared after every step: HP, AP, tile, stance, active and pending statuses; and every round's schedule.
  About 80 distinct skills appear (dashes, disengages, areas, heals, cleanses, taunts, buffs, Focus, riposte);
  statuses seen: stagger, stun, root, bleed, weaken, haste, taunt, riposte.
- Action messages (PORT-BREPORT): chase, dash (and its blocked lines), walk, disengage, area attack, support and
  terrain hazard texts now use Godot's wording, since the log entries carry them as detail text.
- Enemy tells (PORT-INTENT): every round of every battle and world trace also compares the enemy tells read while
  planning (`parity/godot/trace.gd` `intents_state`, recorded at `on_start` / `on_round`): 3088 rounds, 5352 tells
  (kind, target, cell, blocks, skill, revealed, readers, plan, morale state and group), 15 of them Holding. The
  traces hold no foresight holder, no enrage at planning and no arriving unit, so those rules are covered by
  `emberwake_world/tests/intents.rs` instead. Re-recording added only the tells: every fight is otherwise identical.
- Battle metrics, report and action log (PORT-BREPORT): every battle and world trace also carries what a BattleMetrics
  following the fight in sim mode counted (per-unit records, report detail, fight record, self-checks), the
  BattleReport built from it and the session's action log (`parity/godot/trace.gd`, recorded at `on_end`). All
  677 battle fights and 240 world fights compare equal: 32475 log entries (category, actor, target, skill,
  subtype, value, damage, outcome, detail and status text), every record field, every detail, the fight record, the
  report and no METRIC FAIL. One tolerance (decision "Exact number parsing"): an attack line's "Skill x" multiplier
  may print one cent apart, because Godot's JSON reader parses some decimals one ulp off. Re-recording added only
  these keys: every fight is otherwise identical.
- Battle items (ITEMUSE1): 180 more sim fights with `pouch=` (`trace.sh ... pouch=<ids>`, written as
  `traces_<mode>_l<level>_items.json`): 120 duels at level 5 (potion, bomb, focus elixir), 24 team fights and 36
  group fights at level 7; 250 Healing Draught and 39 Throwing Fire Bomb uses, all step for step. Mutating the
  potion threshold or the bomb damage makes 8 to 12 fights diverge, so both item paths are checked.
- World battles (`trace.sh mode=world preset=<p>`, `emberwake_world/tests/world_traces.rs`): 240 fights over every
  world encounter plus synthetic variants (escape, centre deployment, morale, custom hold tiles, start buffs) with
  the balanced, aggressive, withdraw and defensive presets: hold, slay, escape, fled, lost and round-limit verdicts,
  boss phases (hp_below, round, waves, status, ai, enrage), morale breaks and surrenders, ambush and centre
  deployments, a protected guest and start buffs, all step for step.
- Not covered yet: statuses expose, fortify, regen, poison, burn, empower, slow (the shipped classes rarely use
  them in these fights), terrain zones other than the stone arena and the two catalog ones.
- Suite: `cargo test -p emberwake_battle --test trace_parity` replays every `parity/out/traces*.json` except the world
  ones (677 fights, about 8 s).
- Encounters: the three shipped catalog encounters (Ruined Courtyard and Forest Clearing terrain, exits, the
  goblin_ambush reinforcement wave) replay step for step too (`trace.sh mode=encounters`).
- Until 2026-10-19 Windows Smart App Control blocked built binaries at random (`os error 4551` from cargo test,
  "Permission denied" from Git Bash), also ones that ran before; relinking helped only sometimes, PowerShell
  `Start-Process` started a GUI build cmd could not. The user turned it off on 2026-10-19 (decision "Smart App Control
  off"); the gates still relink a blocked test exe in case it is turned on again (skill `known-issues`).

## Game loop (emberwake_world, 2026-10-19)

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `game_state/game_state.gd`, `progression/party_state.gd`, `unit_state.gd` | `emberwake_world::state` | journeys (state after every action) |
| `game_state/new_game.gd`, `game_rolls.gd`, `json_util.gd` | `new_game`, `rolls`, `json` | journeys (start state, every roll) |
| `game_state/save_game.gd`, `save_migrations.gd` (all steps 1 -> 10; the engine adds 10 -> 11, SKILLUI1 rank choice, and 11 -> 12, REWARD1 `reward_paid`, with no Godot counterpart) | `save` (the steps, format and header) on the engine save framework `aegis_save` (slots, envelope, atomic write, migration runner) | `tests/saves.rs`: every Godot save fixture v1..v10 (the loaded state without REWARD1's `reward_paid`) |
| `progression/xp_curve.gd`, `level_up.gd`, `party_builder.gd`, `heal_rules.gd` | `progression` | journeys (HP views, level-ups, camp) |
| `world/world_catalog.gd` (lineage, merged, resolve, party rules), `world_factory.gd` (unit entry, passives) | `aegis_content::Catalogs` (lineage, merged, resolve), `data` (party rules, unit entry, passives) | journeys |
| `exploration/map_rules.gd` (runtime), `explore_runner.gd`, `purse_rules.gd`, `battle_bridge.gd` (apply) | `explore`, `purse`, `battle` (REWARD1 replaced the win-only apply and `PurseRules.roll` / `pay` with rewards per defeated enemy: `battle::apply` with a tally, `purse::share`) | journeys (marker views too; they pass no tally, so the default one gives Godot's win-only numbers, and the extra keys `rewards` and `reward_paid` are skipped) |
| `quests/quest_rules.gd` (runtime), `quest_runner.gd`, `reward_grant.gd` | `quest` | journeys (statuses, objectives, offers) |
| `dialogue/rep_rules.gd`, `dialogue_rules.gd` (runtime), `dialogue_runner.gd`, `shop_rules.gd` (runtime) | `dialogue`, `shop` | journeys (dialogue and shop views) |
| none (Godot's shop bought and sold one line at a time; `ui/shop_panel.gd` had Buy and Sell tabs) | `emberwake_world::shop::trade` (a whole trade at once: counts added up per id and side, today's buy and sell checks, net gold, all or nothing; PLAYUI1-P) and `emberwake_gui::shop` (the three-column trade) | engine-only additions; `shop::buy` and `shop::sell` stay the parity ports (journeys) |
| `travel/travel_rules.gd` (runtime), `travel_runner.gd` | `travel` | journeys (world map view) |
| `exploration/game_encounter.gd` (saved party in battle, HP and pouch write-back, start-status flags) | `emberwake_gui::battle` (`BattleScreen::world`) | manual (GUI) |
| `audio/audio_director.gd` (buses, voice pools), `music_director.gd` (player: crossfade, loop), `volume_preview.gd` (throttle) | `aegis_ui::audio` (the master slider's sample plays on the interface bus and the music sample without the -6 dB, as in Godot) | unit tests (levels, wanted track); manual (GUI) |
| `combat/session/intent_rules.gd`, `combat_session.gd` (`enemy_intents`, `_enemies_on_grid_at_schedule`), `walk_command.gd` (`approach_target`) | `emberwake_battle::intent`, `Session::enemy_intents`, `CmdKind::Walk.approach` | battle and world traces (every round's tells, `parity/godot/trace.gd` `intents_state`) |
| `core/events/action_log_entry.gd`, `combat/session/combat_log_builder.gd` (`from_action`, `riposte_entry`, `chase_entry`, `dash_entry`), the encounters' event entries (phase, morale break, wave, start buff) | `emberwake_battle::log` (`Entry`, `from_action`, `riposte_entry`, `move_note_entry`, `Entry::event`), `Session::log` | battle and world traces (every entry). Not ported: `format_resolver_message` (the BBCode text feed; the battle screen keeps its own log lines), `ref_tag` only as a method, no breakdown text on attack entries (the dojo has the breakdown) |
| `combat/session/battle_metrics.gd` (sim mode and live mode), `combatant.gd` (`health_changed`), `combat_resolver.gd` (`combo_resolved`, `last_round_end_steps`) | `emberwake_battle::metrics` (`BattleMetrics`), `World::damage` / `World::heal` and `World::hp_journal`, `Resolver::combos` / `round_end_steps`, `Session::attach_metrics` / `metrics_step` / `metrics_round` / `metrics_end` | battle and world traces; `emberwake_world/tests/report.rs` (live equals sim, no self-check fails, a fight is unchanged by a collector; a scripted round for combos, the best finish and regen credit, which no trace holds). Not yet wired: the battle screen attaches no collector (BREPORT1-U). Intended: no round checkpoints (the engine has no round reset yet; port them with it); signals become session feed points in Godot's order |
| `combat/session/battle_report.gd`, `combat_encounter.gd` (`hero_key`), `game_encounter.gd` (`hero_key`) | `emberwake_battle::report` (`Report::build`, `columns_for`, `best_moment`, tips, `problems`), `Session::hero_key`, `Roster::lore_ref`, `Prebuilt::key` | battle and world traces; `emberwake_world/tests/report.rs` (the Godot suite's column, tip, best-moment and validation cases) |
| `ui/combat/intent_text.gd`, the tells in `combat_grid_panel.gd`, `combat_enemy_card_panel.gd` and the hover card | `emberwake_gui::intent_text`, `battle.rs` | manual (GUI, agent link). Not ported: the timeline's masked enemy banner (the engine's strip shows the selected hero only; the UI Gallery's battle timeline masks enemy tracks instead). Intended: a revealed enemy with no reader (enraged) is headed "Enraged: its whole plan shows" (Godot printed an empty "Read by") |
| `scripts/dojo/dojo_session.gd`, `dojo_breakdown.gd`, `dojo_loadouts.gd`, `dojo_log_entry.gd`; `core/inventory/party_carry.gd`; `inventory.gd` (`needs_weapon_switch`, `toggle_active_weapon`); DamageCalculator's explanation keys | `emberwake_battle::dojo`, `emberwake_rules::carry`, `gear::Inventory` (`off_active`), `damage::AttackPower` / `Hit` explanation fields | `emberwake_world/tests/dojo.rs` (rules-derived; the battle traces check the damage math is unchanged). Intended differences: the trained-fists breakdown names its weapon power (Godot printed 0), the stash is listed by item id (Godot: catalog order), arrows and multiplication signs are plain ASCII |
| `combat/arena/block_snapshot.gd`; `combat_session.gd` (`round_actions`, `round_snapshots`, `_refresh_archive`) | `Session::history` (`RoundRecord`, `UnitSnap`), `snapshot`, `round_record` (`recorded_actions`, `snapshot_at`) | `emberwake_world/tests/history.rs` (rule test; the traces do not record the archive). Intended: while planning the archive holds the enemies' drafts only (Godot added the party's unsent plan and refreshed on every plan edit; the UI Gallery draws the live plan instead); the archived actions keep their cancelled flags and are refreshed at every block end (Godot's copies dropped them); `UnitSnap` is the display part of BlockSnapshot (no pouches, targets, status rounds, magnitudes or sources, boost rounds, objective, phase or morale state), so it cannot restore a round |
| `ui/timeline_modal.gd` (TimelineModal), `ui/combat/combat_timeline_panel.gd` (the round timeline) | `aegis_ui::timeline` (engine widget) and the UI Gallery's battle timeline (`emberwake_gui::gallery`) | manual (GUI, agent link); not yet on the battle screen |
| `tools/balance_sim.gd`, `tools/sim/*.gd` (SimArgs, SimRunner, SimVariants, SimMetrics, SimTargets, SimScenarios, SimWorld, SimParty, SimRoles, SimRoleMode, SimCampaign) | `emberwake_sim` (`aegis sim <mode>`) | `emberwake_sim/tests/sim_parity.rs`: 20 recorded command lines over every mode, 469 printed lines equal (`games/emberwake/parity/sim.sh`). Kept as recorded: the striker's "prio down rnd" KPI reads the test unit's own down round (Godot's match hit that arm first). Not ported: the sandbox catalog fork (the units are built in memory) and the upsert's refusal lines (the sim's units are always valid) |
| Godot Control focus (`ui_focus_next` / `ui_focus_prev`, `ui_up` .. `ui_right`, `ui_accept`, `grab_focus` on menus) | `aegis_ui::focus`, `Screen::focus_first` (the opening focus) and `Screen::pointer_to` (engine-only: the OS pointer onto a widget, ASKNO2), the shell's `focus_key` / `settle_focus`, `Screen::arrow_focus` / `auto_focus` | `aegis_ui` unit and shell tests (Tab order, arrows, an opted-out screen, a choice list and a slider from the keys, a new screen); manual (agent link). Intended: focus comes from the widget list each frame (no per-control focus neighbours); a click hides the ring |
| `ui/dojo/*.gd` panels (party, inspector, derived, gear, stash, stash filter, stage, console, log, carry bar, allocation) | `emberwake_gui::dojo` on `aegis_ui::window`, scroll areas and `Painter::rich` | manual (GUI, agent link) |
| `ui/framework/ui_window.gd` (`_clamp_to_parent`, `commit_fraction`, title drag, grip resize, collapse), `ui/dojo/dojo_workspace.gd` (`LAYOUTS`, `apply_layout`, `save_layout`, `_load_layout`, window toggles) | `aegis_ui::window` (`Desk`, `Win`, `Layout`), the shell's desk handling (`desk_press`, `save_desk`) | unit and shell tests (window.rs, shell.rs) |
| `audio/music_director.gd` (choice: contexts, candidates, scopes, kind and combat context), `sound_events.gd` | `emberwake_gui::music`, `battle` (`sound_of`, from the resolver log; a hit for 0 plays mitigated, the engine log has no mitigation tag) | unit tests (chains, every shipped file decodes, log lines) |
| none (Godot showed static tokens; its FxSettings speed and instant flags) | `aegis_core::assets::ClipPlayer`, `aegis_ui::art` `played` / `body_outline` (with `flip` for mirrored frames, `body_clip_flipped`, SPRITE2), `Image::outline`, `Painter::ellipse` (SPRITE1-L, 2026-10-22); the battle honours the settings `anim_speed` and `skip_anim`; `aegis_core::assets::{View, Facing}`, `Clips::resolve_view` / `frame_at_view`, `ClipPlayer::frame_facing`, `Art::played_facing` (DIR4-E, 2026-10-23: front and back rows in the clip sheet) | new in the engine, not a port |
| none (Godot had no map props) | `aegis_ui::depth::DepthList`, `Art::varied` / `pick` / `size_of` / `image_mirrored`, `Image::mirrored` (PROPS1-E, 2026-10-23); `emberwake_content` `check::map::prop_errors` (P1-P12), `check::assets::prop_warnings` (A7-A10) (PROPS1-L1) | new in the engine, not a port; parity retired in part: the writer's `"props": []` line and the loader's `props` key on map templates (HISTORY 2026-10-23) |

Godot behaviours found on the way:

- `hash([...])` is murmur3 over the array (Variant ARRAY = 28), Thomas Wang 64 -> 32 for ints, djb2 for strings
  (`aegis_core::ghash`); `String.hash()` is djb2.
- `JSON.stringify` sorts keys by default, so a Godot save round trip re-orders every dictionary (stash, flags,
  quests): engine saves sort keys too, and the journey replay's reload does the same.
- `JSON.parse` makes every number a float and `stringify` writes it back as `3.0`; the 2 -> 3 save step hashes that
  text (rebuilt by `save::godot_text`).

Journeys (`parity/journey.sh`, `tests/journey.rs`): seeded random play through the shipped rules, goal-directed
walking, dialogue biased to quest services, shops, chests, travel, camp, level-ups, abandons, journal turn-ins, save /
load round trips and battle outcomes drawn at random (won / fled / lost). Each action's result, the state after it and
the screen views (quest statuses, objectives, turn-in offers, HP, camp, exits menu, markers, dialogue, shop, world
map) must equal Godot's. Last run: 8 journeys, 5608 actions, all equal; includes quest completion, recruits and a
`connects` world crossing.

Not ported yet: content validation (`*_errors`, warnings), save import / export.

## Data loader (emberwake_content::source, 2026-10-19)

The engine reads the project's `games/emberwake/data/` folder itself; the Godot exports are only needed for parity checks.

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `content/content_catalog.gd` (`reload_from_disk`, `_read_kind`, `_read_ranked_skills`, `_read_items`) | `source::load_catalog` | `tests/data_source.rs` |
| `content/content_schema.gd` (`normalize`, `coerce`, `_copy`), `world/world_schema.gd` (`normalize`) | `aegis_content::schema::normalize`, `coerce`, field tables in `kits/rpg/schema.json` | `tests/data_source.rs` |
| `content/skill_ranks.gd` (`expand`, `effective`, `resolve`), `content/skill_text.gd` (`render`, `token_value`) | `source::expand`, `render` | `tests/data_source.rs` (151 rank rows, every description) |
| `core/specs/item_families.gd` (`derive`, hands, base), `item_modifiers.gd` (`resolve`, `legacy_tiers`), `item_text.gd` (`tier_desc`) | `source::derive_family`, `resolve_item` | `tests/data_source.rs` |
| `core/specs/item_text.gd` (`render`, `token_value`) | `emberwake_content::source::render_item`, `emberwake_world::Data::item_description` | `emberwake_world/tests/item_text.rs` |
| `progression/unit_state.gd` (`_gear_errors`), `core/specs/proficiency.gd` (`can_use`), `ui/menus/gear_model.gd` (`equip`, `unequip`, `_pouch_problem`) | `UnitState::can_wear` and `validate`, `state::item_fits_slot`, `Proficiency::can_use`, `emberwake_gui::party` gear moves (plus slot to slot and pouch reorder, new in GEARUI1) | `emberwake_gui` party tests |
| `world/world_catalog.gd` (`reload_from_disk`, `lineage`, `own_entries`) | `aegis_content::world::load` (over `source::WORLD_KINDS`) | `tests/data_source.rs` |
| `exploration/map_catalog.gd` + `map_rules.gd` (`normalize_template`), `quests/quest_catalog.gd`, `dialogue/service_catalog.gd`, `progression/xp_curve.gd` | `source::load_list`, `xp_curve` | `tests/data_source.rs` |

- `tests/data_source.rs` compares the loader's output with `catalog.json`, `worlds.json` and `game_data.json`: keys, key
- PHASEFOLD (2026-10-22) retired part of `data_source` and `store`: the loader diff folds Godot's catalog and world encounter rows with `source::fold_legacy_waves`, drops Godot's four `wave_*` schema rows per encounters section and skips the two `phases` help rows (`HELP_CHANGED`); the resave check reprints Godot's encounters files folded through the document writer (their data and key order still compared, Godot's byte formatting of those two files no longer). The reinforcement wave is an arrival phase in the engine; Godot's legacy rows replay through the fold (every battle trace, world trace, sim, journey and save unchanged).
  order and int / float types exactly, floats to 1e-9 (Godot's `JSON.stringify` writes fewer digits than it computes
  with: `0.1 + 0.2` is written `0.3`), and the playable worlds. 239 catalog rows and 3 worlds match.
- Run on loader-built data instead of the exports, the journeys (5608 actions), save fixtures and world battles (240)
  still match Godot, and the CLI balance sim prints the same table.
- `kits/rpg/schema.json` (moved from `games/emberwake/schema.json` by KIT1-M) is the committed copy of Godot's field tables (`parity/export.sh only=schema` writes
  `parity/out/schema.json`; the test fails when they drift). It carries labels, help, options and groups too, for
  content editing in the engine later.
- Not read: version 1 (flat) skills and items files, pre-CLASS1 class `skill_path` lists (the shipped data has none;
  the loader reports them), and the load-time validation messages that only block saving in Godot.

## Data writer (emberwake_content::store, 2026-10-19)

Content is edited without Godot through `aegis content check | upsert | remove | apply` (shell `aegis_cli`, binary in `emberwake_host`, also
`tools/fc.sh content`), which replaced `ContentCatalog.upsert` seeds when the Godot build moved to `legacy/`.

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `content/content_catalog.gd` (`save`, `upsert`), `content_schema.gd` (`to_storage`, `is_visible`, `field_errors`) | `store::catalog_storage`, `check` on `aegis_content::store::upsert`, `remove` (hooks) and `aegis_content::schema::to_storage`, `visible`, `field_errors` (`ref_problems`, the writer's reference check until PORT-CHECKS, is unused since) | `tests/store.rs` (the writer parity leaves out the empty defaults of the fields added since the freeze, REWARD1; `tests/data_source.rs` AFTER_GODOT does the same for the loader) |
| `content/skill_ranks.gd` (`collapse`, `fold`), `skill_kinds.gd` (`has_field`) | `store::skill_storage` | `tests/store.rs` |
| `core/specs/item_modifiers.gd` (`stored`) | `store::catalog_storage` (items) | `tests/store.rs` |
| `world/world_catalog.gd` (`save_world`, `upsert`), `world_schema.gd` (`to_storage`) | `aegis_content::schema::world_storage`, `store::manifest_text` | `tests/store.rs` |
| `exploration/map_catalog.gd`, `quests/quest_catalog.gd`, `dialogue/service_catalog.gd` (`to_storage`, `save`) | `store::game_storage` | `tests/store.rs` |
| `JSON.stringify(doc, "\t", false)` | `aegis_content::file::stringify` (14 significant digits for floats, Godot's escapes) | `tests/store.rs` |

- `parity/export.sh only=resave` copies `legacy/data` (catalog, worlds, maps, quests, dialogue) to `out/resave/before`
  and lets Godot's own saves rewrite the copy into `out/resave/after`; the writer, given `before`, writes all 53 files
  of `after` byte for byte. Nine shipped files are older than their schema (field order), so Godot's save reorders
  them too; the first engine save of such a file does the same.
- Validation: id (snake_case), name, enum options and catalog references by the schema; skill ranks and item families
  by construction; then the whole data folder must not check with new problems (the content checks below, since
  PORT-CHECKS), else the file is put back.
- Not covered (retired by ASSETORG2-G on 2026-10-22: the folders are the truth): the asset manifest (formerly `games/emberwake/data/assets/manifest.json`, rows of `res://` paths: redefined with the engine's
  sprite work; the checks read its unit and item keys). The Forge's richer checks (SkillValidator, item budgets, world
  reference rules) are ported since PORT-CHECKS (next section).

## Content checks (emberwake_content::check, PORT-CHECKS, 2026-10-20)

`aegis content check` reports what the Forge's validators reported: soft warnings (printed as `warning:` lines, never
failing), then hard errors; every `upsert` / `remove` refuses a write that adds a blocking one (see Writes below).

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `content/content_catalog.gd` (`validate_entry`, `_prepare`, `warnings_for`, `validate_all`, `warnings_all`), `content_schema.gd` (`field_errors`, `options_of`, `passes_filter`) | `check` (`View`, `validate_entry`, `warnings_for`, `validate_all`, `warnings_all`) | checks parity |
| `content/validators/skill_validator.gd`, `skill_kinds.gd`, `skill_ranks.gd` (`fold`, `collapse`), `skill_text.gd` (`validate`, warnings) | `check::skill` | checks parity |
| `content/validators/class_unit_validator.gd`, `core/specs/stat_rules.gd` (`validate_class`, `validate_spend`) | `check::unit` | checks parity |
| `content/validators/item_validator.gd`, `core/specs/item_modifiers.gd` (`collapse`, `stored`, `stable_order`), `item_families.gd`, `item_text.gd` | `check::item` | checks parity |
| `core/specs/gear_budget.gd` | `emberwake_rules::budget` | checks parity |
| `content/validators/terrain_validator.gd`, `core/specs/passives.gd` (`validate_effects`) | `check::simple` | checks parity |
| `combat/session/encounter_phases.gd`, `encounter_objectives.gd`, `encounter_morale.gd` (`problems`, pace, start buffs), `exploration/purse_rules.gd` | `check::encounter` (REWARD1: `purse.loot` refused; the loot tables, key loot and objective rewards in `check::reward`, new) | checks parity (the purse loot messages left out on both sides) |
| `world/world_catalog.gd` (`validate_entry`, `manifest_errors`, `validate_world`, `validate_all`, `world_warnings`, `warnings_all`) | `check::world` | world cases, lint |
| `quests/quest_rules.gd`, `dialogue/rep_rules.gd` (value rule, key lint) | `check::quest` | world cases, lint |
| `dialogue/dialogue_rules.gd` (trees, conditions, effects, service lines, tree and gone-NPC lints) | `check::dialogue` | world cases, lint |
| `dialogue/shop_rules.gd`, `travel/travel_rules.gd` (with the route graph), `exploration/map_rules.gd` (location rules, lints) | `check::shop`, `travel`, `map` | world cases, lint |
| `assets/asset_registry.gd` (`keys` of unit and item: template keys plus the asset manifest) | none since ASSETORG2-G (2026-10-22): the asset checks A1-A6 read the asset layers (`check::assets`) | retired (HISTORY 2026-10-22) |

- Recording: `games/emberwake/parity/checks.sh` runs `parity/godot/checks.gd` on `legacy/` and writes
  `parity/out/checks.json`; `cargo test -p emberwake_content --test checks_parity` replays it on `legacy/data`.
- Catalog: 5166 cases (three entries of every kind and sub-kind, each field set to a few wrong values in the shape
  it holds, plus hand-made cases for the rules across fields: skill families, combos, item modifiers, tiers and
  text, family changes, encounter phases, objectives, morale and purse) and the shipped catalog's reports (0 errors,
  18 warning keys) match message for message, errors and warnings.
- Worlds: 1555 cases (own entries of every kind in every world, fields set wrong) and the shipped worlds' reports
  match; a lint set (14 entries upserted into ashlands in memory: a dialogue tree, two quests, a location with
  markers and bindings, a shop, a route, a character, regions, a faction, an item, loot and an encounter with start
  buffs) matches with its 196 errors and 26 warning lines over every world.
- Godot divergence: a family band with `min` but no `max` stops Godot's `ItemValidator.family_warnings` with a script
  error; the engine prints the warning with an empty max (the recording leaves that shape out).
- Writes: a write is refused when it adds a blocking problem (`Hooks::blocking`): every catalog error
  (ContentCatalog.upsert validates the entry) and the world entries' `WorldCatalog.blocking_errors` (id, name, enums,
  overrides, json shapes; also recorded and compared per world case). Unresolved world references do not block, as in
  Godot (worlds are full of mutual references); `content check` reports them.
- Not ported: the Forge's per-field problem records ({field, index}); the checks return the message texts.

## Playtests (aegis_host::playtest, PORT-PLAYTEST, 2026-10-20)

The engine records and replays what Godot's PLAYTEST1 did, on the command API instead of plans and RNG states:
every battle runs through the command host, so its request log replays it exactly (the engine has no live round
redo, the reason Godot stored `rng_state` per round). Recordings are version 2 of the `playtest` format; version 1
files (the Godot build's) are refused with that reason.

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `scripts/playtest/playtest_record.gd`, `playtest_store.gd` (ids, list, atomic save) | `aegis_host::playtest` (`Recording`, `new_id`, `list`, `save`) | unit tests |
| `playtest_recorder.gd` (rounds, outcomes) | `Host` checkpoints (`Commands::checkpoint`; Emberwake: the round outcome after a concluded round) and `Session::of` | `emberwake_host/tests/replay.rs` |
| `playtest_replayer.gd` (match, diverge, unverified, take-over) | `playtest::replay` (with `until`) | unit tests, `replay.rs` |
| `tools/replay.gd` (`--battle`, `--json`, `--variant`, `--then`, `--write`, `--seeds`, `--fallback`) | `aegis replay` (`aegis_cli`) | the tests above, by hand |
| `playtest_sweep.gd` | `playtest::sweep_one`, `parse_seeds` | unit tests |
| `content_fingerprint.gd` | `aegis_content::fingerprint` (on `aegis_core::sha256`) | unit tests |
| `scripts/ui/playtest/playtest_hub.gd` (one recording per run) | `aegis_ui::playtest::Playtests` on `Cx`, the screens' `leave` hook | unit test |
| `scripts/ui/playtest/ui_trail.gd` (screen, panel / modal, button, key, hover, click) | the shell (`click`, `back`, `key`, `settle`, `track_hover`), the battle screen (plan_edit, confirm, battle_end) | shell test |
| `scripts/playtest/playtest_logger.gd` (errors), `AssetRegistry.resolve_observer` (asset fallbacks) | `aegis_ui::playtest::report_error` with a panic hook, `report_asset` from the art loader | hub test |
| `ui/playtest/note_box.gd` (the F8 note text box), Godot's `TextEdit` | `aegis_ui::edit` (PLAYUI1-T: the kit text editor), the taller note box, `recordings::NoteView` (a note's full text with Copy) | engine-only additions past Godot (no parity suite) |
| none | `aegis_core::formula` (DERIVE1-E: data formula evaluator) | unit tests; engine-only addition (no parity suite) |
| none | `rpgkit_content` `check/derived.rs`, the `derived_stats` kind (DERIVE1-S) | `rpgkit_content/tests/derived_stats.rs`; no Godot original; the schema parity strips `content.derived_stats` (`data_source.rs` `KINDS_AFTER_GODOT`) |
| none | `rpgkit_rules::derived` (DERIVE1-R: value, breakdown, resist chance) | `rpgkit_rules/tests/derived.rs`; no Godot original; no classic rule reads it, parity unchanged |
| none | `rpgkit_battle` `poise.rs` (RULESET1-P, engine-only: RULESET1 sections 5-7); `rpgkit_rules::derived` gains the `gear_mitigation` input and the `_for(&Ruleset)` wrappers; `rpgkit_rules/src/damage.rs` (DamageCalculator) gains an armour-skip flag for piercing hits under timeline only (classic never sets it) | `rpgkit_battle/tests/poise.rs`; no Godot original; classic byte-identical (trace_parity) |
| none | `rpgkit_rules` `DeckRule`, `build_unit_in`; `rpgkit_world` `progression::build_in` and the member `deck` (RULESET1-D, engine-only: RULESET1 9 D1-D5) | `rpgkit_battle/tests/deck.rs`, `rpgkit_world/tests/deck.rs`; no Godot original; `build_unit` = `build_unit_in(.., None)` (ContentFactory.unit unchanged); classic parity unchanged |
| none | `rpgkit_battle` `skill.rs` `reach_by`, `melee_reach`, `threat_reach`; `rpgkit_rules` `Equipment::has_reach`, `Unit::has_reach_weapon`; the resolved item key `reach_rule` (loader only, the writer drops it) (RULESET1-K, engine-only); `rpgkit_battle::portrait` and `rpgkit_content::check::portrait`, the `portrait_cues` kind, face frame checks A11-A12 (PORTRAITFX1-S, engine-only) | `rpgkit_battle/tests/reach.rs`, `rpgkit_content/tests/reach_rule.rs`; no Godot original; `reach_by` = `reach_for` without the rule; classic parity unchanged (677 fights), data_source unchanged (no reach family in legacy data) |; `rpgkit_content/tests/portrait_cues.rs`, `rpgkit_battle/tests/portrait.rs`; no Godot original (Godot had no portrait cues); parity narrowed per HISTORY 2026-10-26 |
| none | `rpgkit_rules` `ruleset.rs`, `rpgkit_content` `check/ruleset.rs`, the `rulesets` kind (RULESET1-R) | `rpgkit_content/tests/rulesets.rs`, `rpgkit_world/tests/world_ruleset.rs`, `rpgkit_sim/tests/ruleset_args.rs`; no Godot original; the schema parity strips `world.world` `ruleset` / `plan_reveal_flag` and `content.rulesets` (`data_source.rs`, `store.rs`) |
| none | `rpgkit_rules` `status.rs` rows 16-17 (`steadfast`, `stealth`) and the `intent_bound` column; the RULESET1-S checks in `rpgkit_content` `check/{skill,simple,unit,item,encounter}.rs` (SkillKinds gains the sections cost, timing, intent and impact) | `rpgkit_content/tests/timeline_fields.rs`, `rpgkit_rules/tests/timeline_statuses.rs`; no Godot original; the schema, writer and checks parity strip the new fields, options and list items (`data_source.rs`, `store.rs`, `checks_parity.rs`); `rpgkit_content/tests/weapon_types.rs` (CAST1-S: `censer`, `bone_saw`, `sprayer` in `check/{skill,unit}.rs` gear and weapon lists) |
| none | `rpgkit_battle` `grid.rs` `apply_board` / `rows_valid`, `rpgkit_content` `check/encounter.rs` `rows_problems` (ENCOUNTER2-T, R0 authored rows; `apply_zone` unchanged) | `rpgkit_content/tests/encounter_rows.rs`; no Godot original; the schema and writer parity strip `rows` (`data_source.rs`, `store.rs`) |
| none | `rpgkit_content` `check/campaign.rs`, the `rpgkit_data` campaign vocabulary and manifest readers (CAMPAIGN1-S, engine-only) | `rpgkit_content/tests/campaign_schema.rs`, `emberwake_content/tests/campaign_checks.rs`; no Godot original; the schema, writer and checks parity strip the campaign fields, kinds and effect names (HISTORY 2026-10-25) |
| none | `rpgkit_world` `campaign/{mod,meter}.rs`, `GameState.campaign`, `read_flag`, save v13 (CAMPAIGN1-M, engine-only) | `rpgkit_world/tests/campaign.rs`; no Godot original; inert without a `campaign` block (journeys, saves and world traces unchanged) |
| none | `rpgkit_world` `story.rs`, `board.rs`, `GameState.story`, `dialogue::apply_effect`, the `story` block in save v13 (STORYUI1-W, engine-only) | `rpgkit_world/tests/{story,board}.rs`; no Godot original; inert without vignettes, sequences or boards (journeys, saves and world traces unchanged) |
| none | `rpgkit_content` `check/story.rs` and `check/board.rs`, the `rpgkit_data` story vocabulary and step reader (STORYUI1-S, engine-only) | `rpgkit_content/tests/story_checks.rs`, `emberwake_content/tests/story_checks.rs`; no Godot original; the schema, writer and checks parity strip the story fields, kinds and effect names (HISTORY 2026-10-26) |
| `CombatResolver` (resolver.rs), additions only | `rpgkit_battle` `resolver.rs` `ScheduledAction.impact`, `world.rs` `pays_ap` / `recovery_of`, `skill.rs` `ap_cost_by` (RULESET1-T, engine-only) | `rpgkit_battle/tests/timeline_timing.rs`; actions land at `impact`, recovery follows it for enemy-side units under `enemy_moves`; classic `impact == end`, `trace_parity` unchanged (677 fights) |
| none | `rpgkit_battle` `round_intent.rs` (RULESET1-IN, engine-only); `command.rs`, `plan.rs`, `resolver.rs` gain branches behind the rule `round_intent` (`Carry`, `after_move`, the intent filter, the lost-target Rest, the Rest gain at round start) | `rpgkit_battle/tests/round_intent.rs`; classic parity unchanged (677 fights) |
| none | `rpgkit_battle` `round_intent_plan.rs` (RULESET1-PL, engine-only) and the host requests `round_intent`, `manual`, `fill`, `round_plan` (party only) | `rpgkit_battle/tests/round_intent_plan.rs`, `rpgkit_host/tests/round_intent_requests.rs`; no Godot original; classic parity unchanged (677 fights) |
| `IntentRules` (intent.rs), additions only | `rpgkit_battle` `intent.rs` round fields, impact badges, range and plan reveal, party records, the timeline trace keys (RULESET1-V) | `rpgkit_battle/tests/plan_badges.rs`, `rpgkit_world/tests/plan_reveal.rs`, `rpgkit_host/tests/plan_badges.rs`; the classic tell and trace stay the IntentRules port (677 fights) |
| none | `aegis_ui` `Painter::ellipse_ring`, `Image::opaque_bounds`, `Art::body_bounds` (STAGE1-E) | `aegis_ui/tests/stage_pieces.rs`; engine-only addition (no parity suite) |
| none | `emberwake_gui/src/stage.rs` (STAGE1-L: the tilted battle stage) | `stage.rs` and `battle.rs` inline tests; new in the port, no Godot counterpart; no parity suite draws the GUI |
| none | `aegis_save` keyed saves (CAMPAIGN1-E: `write_keyed`, `read_keyed`, `exists_keyed`, `peek_keyed`) | `aegis_save/tests/keyed.rs`; engine-only addition (no parity suite) |
| none | `aegis_ui::vignette` (STORYUI1-E: the vignette player and `VignetteScreen`), settings `text_speed`, `reduce_motion` | `aegis_ui/tests/vignette.rs` (TE1-TE10); engine-only addition (no parity suite) |
| none | `aegis_ui::portrait_fx`, `aegis_core::assets` face sidecars (PORTRAITFX1-E) | `aegis_ui/tests/portrait_fx.rs` (E2-E15), `assets.rs` unit tests (E1); Godot had no portrait effects, sidecars or cue rows; no parity suite |
| `ui/combat/fx_presets.gd` (token tweens only) | `aegis_ui::fx` (FX1-E: effect clock and hit-stop, primitives, live list, camera shake), `Painter::shift`, `Image::silhouette`, `Art::body_silhouette`, `Settings::reduce_motion` | `aegis_ui/tests/fx.rs` (E1-E13); engine-only addition (no parity suite); the miss sidestep (smaller) and the token-shake fix land in FX1-L1/L2; the melee bump is not ported |
| `scripts/playtest/trail_report.gd` (`--trail-report`) | `aegis_host::trail` (`build`, `lines`), `aegis replay <file> trail-report` | unit test (acceptance 12 and 13) |
| `scripts/ui/playtest/note_dialog.gd` (F8: screenshot, text, tags) | `aegis_ui::notes::NoteBox`, the shell's `note` (stamp + `Screen::note_context`), `Playtests::add_note`, `aegis_ui::jpeg` (baseline encoder, std only), the kit's text field (`Ui::text_field`, `Kit.edit`) | shell test |
| `scripts/ui/runner` Playtests panel, `playtest_store.gd` (`copy_to_repo`, inbox folders) | `aegis_ui::recordings::Recordings` (kit screen; the game gives the launcher), `playtest::copy_to`, `image_dir`, `list` of `<id>/<id>.json`; `BattleScreen::playback` | shell test, by hand on the agent link |

- A variant is a request patch ({at, remove, requests, then}), not a plan patch: any request can change.
- Ids use UTC (std has no local time zone).
- The trail: controls are the engine kit's widget infos (path = widget id, cls = widget kind); overlays open and close
  as modals, other screens are screen changes (leaked is always 0: the engine keeps no node tree). Keys typed into a
  text field are not logged.
- The report differs from `trail_report.gd` only in names and one count: `scene` events and fields are `screen`, the
  leak line says "left" (no node tree), `asset_templates` is not counted (the engine draws no template placeholders
  through the loader). A right click (back) always counts as handled. A panic that ends the process is reported but
  not written (no frame follows it); panics the process survives are.
- Notes: the screenshot is a JPEG (quality 85) from the engine's own encoder; the stamp has the screen, mouse,
  viewport, the control under the mouse and the battle's slot, round, block and phase.
- Playback differs from Godot's: Play back replays the battle to its end and shows the final state (the battle's
  own replay of the last round and its log), not a round-by-round viewer with a notes drop-down (PLAYBACK2). Take
  over picks Round 1 or After round N; the new session's `end.parent` names {recording, battle, at}. Scenarios (H)
  are not listed (none exist).
- Recording can be turned on and off while the app runs (the browser's toggle; the setting is now the engine's).
- Not ported yet: party battles of the world map (PLAYTEST2).
