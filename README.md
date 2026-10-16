# Tactical RPG (Godot 4)

A modular, simultaneous-execution tactical RPG engine built in **Godot 4** (GDScript). Every unit plans a **6-block round** (AP-costed skills, one move, AI-planned enemies) that resolves simultaneously on a **9x9 spatial grid**, featuring rock-paper-scissors **stance dynamics**, tactical targeting, perimeter escape/withdrawal, and deterministic state scrubbing.

---

## Table of Contents

1. [Core Design Philosophy](#core-design-philosophy)
2. [Codebase Architecture & Directory Breakdown](#codebase-architecture--directory-breakdown)
3. [Currently Implemented Features & Functionality](#currently-implemented-features--functionality)
4. [Headless Testing & Automated Quality Assurance](#headless-testing--automated-quality-assurance)
5. [How to Run](#how-to-run)
6. [Roadmap & Features Yet to be Implemented](#roadmap--features-yet-to-be-implemented)

---

## Core Design Philosophy

- **Decoupled Rules Engines**: All core game mechanics inherit from `RefCounted` rather than scenes or rendering nodes, allowing 100% headless simulation and rapid automated testing.
- **Simultaneous 6-Block Rounds**: Combat does not use traditional turn-based initiative. Every unit plans its round as a sequence of skills on a 6-block timeline (AP pool of 12, +4 per round); all plans then resolve together, block by block, with multi-block durations and stance interactions.
- **Dynamic Stance Matchups**: three stances in a rock-paper-scissors loop (`Offensive` > `Tactical` > `Defensive` > `Offensive`) plus `Neutral` govern damage dealt and received through asymmetric multipliers.
- **Spatial Positioning & Withdrawal**: 9x9 grid maneuvering with center-ring deployment, reach bands, and safe boundary withdrawal.
- **Content as data**: skills, items, classes, units and encounters live in a JSON catalog edited with the Content Forge; the engine, the scenes and the tests all load from it.

---

## Codebase Architecture & Directory Breakdown

```text
tactical-rpg/
├── assets/                       # Static assets (icon.svg only; all art is generated in code, see scripts/art/)
├── data/
│   └── catalog/                  # Content catalog (JSON, edited in the Content Forge): the single source of game content
│       ├── skills.json               # Every skill tier (attacks, defends, focus, rest + engine skills marked "system")
│       ├── items.json                # Weapons, shields, armor, accessories, consumables ("starter_stash" flag)
│       ├── classes.json              # The six base classes: stats, growth, gear proficiency dict, skill path, loadout, art
│       ├── units.json                # Party members & enemies: vitals or class/level-derived, loadout, skills, AI profile
│       └── encounters.json           # Battles: zone, exits, round rules, party & enemy rosters
│   └── worlds/                   # Lore worlds (JSON, edited in the World Forge): modular, inheritable; reference catalog assets
│       ├── ashlands/                 # Root world: Ember Wardens vs the Ashen Horde (regions, locations, factions, characters, named item, disciplines, quest, lore, 2 encounters)
│       ├── tidewrack/                # extends ashlands: the same war re-skinned (overrides) plus its own coast, characters and encounter; a tuned boss
│       └── iron_reaches/             # separate world that connects to ashlands (a road into Warden Keep) without inheriting
│           └── world.json + regions / locations / factions / characters / items / disciplines / quests / lore / encounters .json per world
├── scenes/
│   ├── combat_arena.tscn         # Main scene: combat UI foundation (CombatSession + CombatDockHost)
│   ├── combat_arena.gd           # Builds the session for an encounter and mounts the dock host
│   ├── scene_runner.tscn         # Main scene: Scene Runner menu (launch scenes, hot-reload scripts, run suites)
│   ├── test_arena.tscn           # Test harness scene: runs SuiteRegistry.run_all (headless exit code); windowed opens the runner
│   ├── test_arena.gd             # Harness: headless exit code for check_project.ps1 and the runner's suite buttons
│   ├── gear_dojo.tscn            # Dedicated Gear, Skill & Damage Formula Benchmark scene
│   ├── gear_dojo.gd              # Thin bootstrap: builds DojoSession, mounts the window workspace or runs assertions
│   ├── content_forge.tscn        # Content Forge: author skills, items, classes, units & encounters; playtest in the arena
│   ├── content_forge.gd          # Mounts ContentForge; swaps in combat_arena for playtests ("Back to Forge")
│   ├── world_forge.tscn          # World Forge: author lore worlds; play their battles
│   ├── world_forge.gd            # Mounts WorldForge; swaps in combat_arena for "Play this encounter"
│   └── dojo/
│       └── dojo_assertions.gd        # 42 headless dojo assertions (rules, session actions, filters, carry, catalog skills, dummy attacks, UI)
├── scripts/
│   ├── content/                  # Content catalog engine (headless)
│   │   ├── content_schema.gd         # Field lists per kind (tables in skill_fields.gd, item_fields.gd, class_unit_fields.gd): types, defaults, ranges, enums, refs, conditional fields; normalising
│   │   ├── content_catalog.gd        # Load/save JSON, CRUD, duplicate-as-template, reference tracking & renames, validation, sandbox fork
│   │   └── content_factory.gd        # Entries <-> engine objects (SkillDef, Item, ClassArchetype, Combatant + AiProfile)
│   ├── world/                    # World engine (headless): the lore layer over the content catalog
│   │   ├── world_schema.gd           # Element kinds (regions, locations, factions, characters, items, disciplines, quests, lore, encounters), fields, tuning keys
│   │   ├── world_catalog.gd          # Worlds on disk, inheritance (lineage), namespaced ids, overrides, connections, references & renames, validation, sandbox fork
│   │   └── world_factory.gd          # World character / named item -> playable unit / item (base asset + lore + tuning, disciplines)
│   ├── core/                     # Foundational rules engine primitives & base models
│   │   ├── combat/               # Core unit, stance & command models
│   │   │   ├── battle_command.gd         # Base command: blocks, AP + stance switch at start, initiative, effect at end
│   │   │   ├── combatant.gd              # Core unit model (HP, AP, speed, range, stance, statuses, buffs, target, known skills)
│   │   │   ├── combat_types.gd           # Stances, enum helpers, matchup multipliers & formatting
│   │   │   └── damage_calculator.gd      # Authoritative damage formula (shared by arena, dojo & future AI)
│   │   ├── utils/                # Shared helpers
│   │   │   └── dict_util.gd              # Typed reads & lenient conversions for Variant/Dictionary data
│   │   ├── specs/                # Character specs, 6-stat framework & progression models
│   │   │   ├── character_stats.gd        # 6 stats (STR, DEX, INT | CONS, PERC, KNOW), leveling (1-10), derived stats
│   │   │   ├── class_archetype.gd        # Base stats, growths & proficiencies; presets come from the catalog (create(id))
│   │   │   ├── stat_rules.gd             # Enforced rules: class budget & growth, spend pool, tiers, SK2 unlock constants
│   │   │   ├── proficiency.gd            # Gear proficiency dict {type: {level, scaling}} and what training does
│   │   │   ├── passives.gd               # Passive effects granted by sources (stat, proficiency, resist, AP regen, skill)
│   │   │   └── stat_modifier.gd          # Composable flat/percent modifiers with contextual gating
│   │   ├── inventory/            # Gear, equipment requirements & inventory containers
│   │   │   ├── item.gd                   # Base item model (type, rarity, weight, value)
│   │   │   ├── equipment.gd              # Gear model with stat requirements, threshold tiers & timeline penalties
│   │   │   ├── consumable.gd             # Consumable items used in combat (ItemCommand) or exploration
│   │   │   ├── inventory.gd              # Party stash, equipped gear loadout & 3-slot combat battle pouches
│   │   │   ├── item_catalog.gd           # create(id) from the content catalog, named shorthands, starter stash
│   │   │   ├── party_carry.gd            # Party-wide carry capacity, load tiers & overload speed penalty
│   │   │   └── stash_filter.gd           # Stash filter model: gear-slot filter + custom dimensions & sorting
│   │   └── events/               # Headless simulation event data transfer objects
│   │       └── action_log_entry.gd       # Standardized action event model (ticks, actors, outcomes)
│   ├── combat/                   # Combat simulation, spatial arena rules & battle actions
│   │   ├── actions/              # Discrete battle command implementations
│   │   │   ├── attack_command.gd         # Melee & ranged attacks: reach band, one-tile adjust at start (step in / ranged back off)
│   │   │   ├── defend_command.gd         # Defensive stance transition & incoming damage mitigation
│   │   │   ├── focus_command.gd          # Tactical buff command (+50% ATK for 2 rounds)
│   │   │   ├── walk_command.gd           # One block of a planned move: any path, re-paths after a cut-short half, refunds unwalked tiles
│   │   │   ├── rest_command.gd           # Idle block: recovers AP, returns to Neutral
│   │   │   ├── area_attack_command.gd    # Tile-targeted attack hitting every unit in an area (friendly fire)
│   │   │   ├── status_block_command.gd   # A block lost to Stagger / Stun
│   │   │   └── target_command.gd         # Switch Target: lock onto a different opponent
│   │   ├── session/              # Headless combat session & encounter definitions
│   │   │   ├── combat_session.gd         # Battle state: rounds, resolution, snapshots, replay, selection, log
│   │   │   ├── combat_encounter.gd       # Encounter plug-in base (combatants, deployment, plans, triggers, art)
│   │   │   ├── catalog_encounter.gd      # Any catalog encounter in the real engine: spawns units, AI profiles, deployment, reinforcement wave (virtual _unit_entry / _customize)
│   │   │   ├── world_encounter.gd        # A world's battle played in that world: rosters are world characters (lore names, art, named items, tuning)
│   │   │   ├── encounter_source.gd       # Lists / creates every battle: catalog ids ("goblin_ambush") and world ids ("tidewrack:strand_clash")
│   │   │   └── combat_log_builder.gd     # Resolved actions -> log cells; resolver messages -> BBCode
│   │   ├── planning/             # Round command sequences (headless)
│   │   │   ├── skill_def.gd              # One skill tier: level 1-5, AP cost = level (Move per tile, Rest free), stance pool, auto switch cost
│   │   │   ├── skill_catalog.gd          # The catalog's skills as SkillDefs; per-unit filtering (cap + known skills)
│   │   │   ├── plan_step.gd              # One input: Engage (target + attack, auto-moves), Move, Withdraw, Self
│   │   │   ├── plan_entry.gd             # A compiled timeline command (lowest free block, AP charged)
│   │   │   ├── plan_context.gd           # Grid, sides and AI profiles a plan works against
│   │   │   ├── plan_option.gd            # A probed next input: valid or why not, plus AP/block cost
│   │   │   └── combat_plan.gd            # Per-unit sequences: simulate, validate (stance flow, AP, blocks, paths), options, commit
│   │   ├── status/               # Round-long statuses
│   │   │   └── status_effects.gd         # Stagger / Stun / Root, seeded resist rolls, AP-debt penalties, round roll-over
│   │   ├── ai/                   # Computer-controlled planning
│   │   │   ├── ai_profile.gd             # Role, power, skill cap, flee/guard thresholds, target & skill preferences
│   │   │   ├── plan_presets.gd           # One-click AI draft presets (balanced, all-out, focus weakest, guard, kite, economy, fall back)
│   │   │   └── tactical_ai.gd            # Drafts a validated step sequence for a unit
│   │   └── arena/                # Arena spatial rules & resolution
│   │       ├── combat_grid.gd            # 9x9 grid, Manhattan distance, open/closed arenas, escape tiles, terrain walkability
│   │       ├── terrain.gd                # Terrain types (14) with walkability, cover, move cost & hazard metadata
│   │       ├── combat_zones.gd           # 8 preset 9x9 combat zones (forest, river, lava cavern, ...)
│   │       ├── grid_path.gd              # Breadth-first routes (centre tie-break); shared by planner, AI and commands
│   │       ├── combat_resolver.gd        # Round resolution: AP/stance at start, effects at end, initiative, casualty cleanup
│   │       └── block_snapshot.gd         # Historical state capture for deterministic scrubbing/replay
│   ├── art/                      # Procedural SVG art (rasterized + cached, no image assets)
│   │   ├── svg_raster.gd             # SVG string -> cached ImageTexture
│   │   ├── unit_art.gd               # Class-glyph tokens (blue friendly / red hostile / yellow neutral) & portraits
│   │   ├── tile_art.gd               # Terrain tiles (seeded variants) plus exit/center overlays
│   │   └── item_art.gd               # Gear icons by type with rarity frame; unique icons via Item.icon_id
│   ├── dojo/                     # Gear Dojo headless logic (no UI dependencies)
│   │   ├── dojo_session.gd           # State + actions: heroes, dummy, stash, quick setup, strikes, benchmarks, stat allocation, test log
│   │   ├── dojo_loadouts.gd          # Declarative quick-setup gear presets (class-proficiency aware)
│   │   ├── dojo_breakdown.gd         # DamageCalculator result -> step-by-step BBCode explanation
│   │   ├── dojo_log_entry.gd         # ActionLogEntry subclass carrying a math breakdown
│   │   └── dojo_stat_defs.gd         # Display metadata (names, roles, colors) for the 6 core stats
│   ├── tests/                    # Headless test suites & validation harnesses
│   │   ├── core_rules_test_suite.gd  # Core combat rules via the planner: guard & matchup, stances, Focus, casualties, withdrawal, feed
│   │   ├── class_rules_test_suite.gd # Six classes: budget/growth, spend variance, tiers, proficiency, passives, skill paths
│   │   ├── content_test_suite.gd     # Catalog parity, round trips, validation, authoring, full-engine battle, Forge UI
│   │   ├── runner_test_suite.gd      # Scene Runner: scene list, launch/restart/menu, unsaved-work guard, hot reload
│   │   └── world_test_suite.gd       # Worlds: shipped data, inheritance, same-base/different-lore parity, tuning, renames, connections, Forge UI, runner
│   └── ui/                       # View layer, docks, components & full-screen modals
│       ├── framework/                # Neutral UI framework shared by the dojo and the combat UI
│       │   ├── ui_panel.gd               # Panel base: metadata, bind(session), refresh(), overlay requests
│       │   ├── ui_window.gd              # Draggable / resizable / collapsible floating window
│       │   ├── ui_overlay_layer.gd       # Hosts overlay windows (pickers, filters, dialogs) above a host
│       │   ├── ui_theme.gd               # Shared palette, font sizes, widget & summary helpers
│       │   └── ui_tip_panel.gd           # Container with BBCode formula tooltips
│       ├── dojo/                     # Gear Dojo & Party/Stash components (extend DojoPanel)
│       │   ├── dojo_panel.gd             # UiPanel adapter with a typed DojoSession
│       │   ├── dojo_workspace.gd         # Toolbar, desk, window toggles, layout presets, persistence, overlays
│       │   ├── dojo_unit_inspector_panel.gd  # Component 1: identity, 6 stats, gear proficiency & passives (static layout)
│       │   ├── dojo_gear_panel.gd        # Component 2: equipped gear, pouch & Quick Setup
│       │   ├── dojo_stage_panel.gd       # Component 3: combat region (hero vs dummy, vitals, FX)
│       │   ├── dojo_console_panel.gd     # Component 4: test console (attacks, skills, modifiers, benchmarks)
│       │   ├── dojo_derived_stats_panel.gd # Component 5: derived stats with hover formulas
│       │   ├── dojo_log_panel.gd         # Component 6: test log using the battle-log cell UI
│       │   ├── dojo_party_panel.gd       # Party & Stash column A: 4 party slots, carry bar, inspector/derived tabs
│       │   ├── dojo_stash_panel.gd       # Party & Stash column C: slot filters, custom filter, one-click equip
│       │   ├── dojo_carry_bar.gd         # Party-wide carry meter with formula tooltip
│       │   ├── dojo_allocation_panel.gd  # Overlay: allocate attribute points
│       │   ├── dojo_picker_panel.gd      # Overlay: equip picker for one slot / battle pouch
│       │   └── dojo_stash_filter_panel.gd # Overlay: custom stash filter (stat, rarity, type, damage, hands, sort)
│       ├── combat/                   # Combat panels (extend CombatPanel) & the dock host
│       │   ├── combat_panel.gd           # UiPanel adapter with a typed CombatSession
│       │   ├── combat_dock_host.gd       # Static dock regions, layout presets, undock/edit mode, modals, shortcuts
│       │   ├── combat_roster_panel.gd    # Party / enemy roster (enemies shown only while on the field)
│       │   ├── combat_grid_panel.gd      # 9x9 grid tokens, hover cards & action FX
│       │   ├── combat_action_log_panel.gd # Compact resolution log dock
│       │   ├── combat_inspector_panel.gd # Unit inspector (snapshot-aware)
│       │   ├── combat_forecast_panel.gd  # Tactical forecast
│       │   ├── combat_playback_panel.gd  # Step / play / reset / round advance
│       │   └── combat_timeline_panel.gd  # Block ruler & focused unit track (planning bar)
│       ├── runner/                   # Scene Runner (main menu)
│       │   ├── scene_runner.gd           # Scene list, launch / restart / back to menu, unsaved-work guard, suite runs, F5/F6/F10
│       │   └── script_reloader.gd        # Hot reload: recompiles scripts edited on disk (GDScript.reload)
│       ├── forge/                    # Content Forge authoring UI
│       │   ├── forge_source.gd           # What a ForgeForm edits (fields + reference choices); ContentForgeSource is the catalog's
│       │   ├── content_forge.gd          # Kind tabs, entry list (search/side filter), form, preview, problems, save, playtest
│       │   ├── forge_form.gd             # Schema-driven editor: one widget per field, conditional fields, ref pickers
│       │   └── forge_preview.gd          # What an entry becomes in the engine (stats, reach, damage, deployment), icons/portraits
│       ├── world/                    # World Forge authoring UI
│       │   ├── world_forge.gd            # World picker / new world, element tabs, merged list (inherited marked), form, lore preview, Override, play
│       │   ├── world_forge_source.gd     # Plugs WorldSchema / WorldCatalog into the shared ForgeForm (refs resolve from the edited world)
│       │   └── world_preview.gd          # What an entry means: lore, what it plays as, where it differs from its base, who uses it
│       ├── combat_grid_view.gd       # 9x9 grid rendering (54px tiles), tokens, tile input, tweens & floating text
│       ├── plan_overlay.gd           # Planning ghosts on the grid: route, destination ghost, reach, target ring, cost hint
│       ├── plan_action_popup.gd      # Small per-target popup of valid actions (AP/blocks, hover preview, 1-9/Enter)
│       ├── roster_panel_ui.gd        # Party status (2x2) and hostile status (5x2) docks
│       ├── unit_inspector_ui.gd      # Left dock (avatar, vitals, stats, action decks)
│       ├── tactical_forecast_ui.gd   # Right dock (range checks, damage forecast, arena intel)
│       ├── playback_controls_ui.gd   # Step prev/next, Auto-Play, Reset, Round navigation
│       ├── timeline_dock_ui.gd       # Top-row dock (block ruler sized to fit, unit track, planning bar: AP status, AI Draft, Clear, Confirm; drag/remove steps)
│       ├── action_log_dock_ui.gd     # Top action dock displaying active block resolution
│       ├── combat_log_modal.gd       # Fullscreen expanded event stream modal with category filters
│       ├── timeline_modal.gd         # Fullscreen synchronized multi-unit timeline modal
│       └── character_equipment_modal.gd # Party & Stash screen: 3 columns (party viewer, gear loadout, party stash) sharing one DojoSession
├── check_project.ps1             # 4-step automated validation script (cache, syntax, arena suite, dojo suite)
├── check_project.bat             # Batch launcher for check_project.ps1
└── run.bat                       # Opens the Scene Runner (every scene, hot reload, test suites)
```

---

## Currently Implemented Features & Functionality

### 1. Simultaneous Block Resolution Engine

- **Synchronous Execution**: Block-by-block progression through the round (6 blocks; per encounter). Every action is a planned command with a start block and a length; it pays AP and enters its stance when it starts, and its effect lands at the end of its last block.
- **Initiative Sorting**: Multiple actions completing on the same block resolve deterministically via speed ordering `(priority * 1000) + speed`.
- **Casualty Action Cancellation**: If an active combatant is eliminated mid-round, all future scheduled commands for that unit are cancelled immediately.
- **Round Rollover**: Buff durations decrement cleanly, lingering stances carry over deterministically, and fallen/withdrawn units are purged from subsequent rounds.

### 2. Stance System & Matchup Multipliers

- **3 stances + Neutral**: `OFFENSIVE`, `DEFENSIVE`, `TACTICAL` form a rock-paper-scissors loop; `NEUTRAL` sits outside it (no advantage, no exposure).
- **Matchups** (attacker stance vs defender stance): Offensive beats Tactical (+25%), Tactical beats Defensive (+25%), Defensive beats Offensive (+25%); the losing side of each pair takes -25%. Same stance or Neutral = 1.0x.
- **Stance pools**: Attack skills are Offensive, Defend skills Defensive, Focus/buff skills Tactical. Move keeps your current stance; Rest returns you to Neutral.
- **Automatic switching**: using a skill from another stance switches stance for **+1 AP** and no block cost; the new stance applies as the skill starts.

### 2b. Damage Formula (`DamageCalculator`, one formula everywhere)

Every hit goes through `DamageCalculator.compute`: arena attacks (single-target and each victim of an area attack), the planner's `~N dmg` estimates, the tactical forecast and the Gear Dojo.

- **Attack Power = Weapon Power + Unit Power**
  - *Weapon Power* = (base + threshold-tier bonus + proficiency training) × (1 - stat deficit). A weapon that doesn't suit the attack (a sword for a ranged shot) adds 0.
  - *Unit Power* = the weapon's scaling stat × the class's **proficiency scaling** for that gear (1.0-2.0, default 1.5; `DamageCalculator.unit_power_scaling`), then × the unit's **tier power** (hero 1.0 ... minion 0.6, boss 1.2). A weapon that doesn't suit the attack uses DEX (ranged) or STR (melee).
  - *Unarmed*: Weapon Power 0, Unit Power **STR + DEX**. Off-hand jabs use the off-hand weapon, or bare hands.
  - *Authored strike power* (a unit's `power` > 0 in the catalog, e.g. the goblins) is absolute (tiers don't scale it); used by fixed units such as the training dummy.
- **Hit = Attack Power × Focus × Skill × Stance Matchup × Crit × (1 - Guard) × (1 - Armor)**, rounded once, minimum 1.
  - *Stance Matchup*: 1.25 / 0.75 / 1.0 (section 2).
  - *Crit*: **5% + 1% per DEX above 10 + 1% per weapon training level above 1**, damage **×1.5**. Rolled with the battle's seeded RNG (the encounter's `rng_seed`; the Dojo's own seeded RNG), so replays repeat exactly.
  - *Guard*: Defend skills (Guard -30%; stronger tiers come from class skill families such as shield wall, not basic variants).
  - *Armor*: rating = equipped gear mitigation + STR / 4 (or the unit's authored `armor`, the training dummy has 0); damage × (1 - rating / (rating + 50)): 10 → -17%, 20 → -29%, 40 → -44%.
- **Estimates** (`DamageCalculator.expected`): the same formula with no crit and no guard (unknown ahead), armor included. Log cells show the hit actually dealt and, in debug builds, the full breakdown on hover (`DamageCalculator.describe`).
- **Future**: weapon-focused skills may scale only Weapon Power; class / weapon preferences may change the 1.5 scaling.

### 3. 9x9 Spatial Grid & Spatial Rules

- **Arena Geometry**: Fixed 9x9 grid (`0..8` on both axes) with landmark center at `(4, 4)`.
- **Center-Ring Deployment**: Initial spawn radius around `(4, 4)` offset by attack range so units can engage center on Block 1.
- **Reach Bands**: attacks check the target's distance against the skill's reach band (melee 1, ranged 2-3, off-hand jab 1, area skills their own); a targeted attack adjusts one tile when it starts, otherwise an out-of-reach attack fails cleanly.
- **Perimeter Escape & Partial Withdrawal**: Stepping onto designated exit tiles (e.g. `(8, 4)`) triggers safe partial withdrawal, removing the unit from active combat without penalty.
- **Reinforcement Ingress**: an encounter's reinforcement wave enters before a given block of a given round (e.g. the Goblin Ambush at Round 1 Block 4), deploying on free tiles of the enemy side; the AI plans it from the next round.

### 4. Deterministic Scrubbing & Replay System

- **State Snapshots (`BlockSnapshot`)**: Full state (HP, AP, stances, targets, buffs, guard, active flags, coordinates) captured per block and per log entry.
- **Two-Way Stepping by log point**: After a round resolves, `Step Prev` / `Step Next` walk the round's log entries one at a time (each entry keeps the board state right after it), so a 2-block move replays from where it started. Selecting a round jumps to its start; selecting a block jumps to the state just before that block. Playback is a review/testing aid only and never changes the battle.
- **Log export**: in debug builds every resolved round is written to `user://combat_last.log` (board cells plus the plain-text feed) for sharing and bug reports.
- **Multi-Round Historical Traversal**: Review previous completed rounds (Round 1, Round 2) without losing active simulation state.

### 5. Modular Combat UI (`combat_arena.tscn`)

- **Headless `CombatSession`**: Owns one battle: the encounter's combatants, grid and resolver, round flow, block-by-block resolution, per-block snapshots, multi-round traversal, selection/hover and the action log. Panels only read it and call its actions; every change ends in `changed`, visual one-shots use `fx_requested` / `hover_changed`. The whole battle flow is headless-testable.
- **Pluggable `CombatEncounter`**: Defines *what* a battle is (combatants, deployment, per-round plans, mid-round triggers, narration, art). Every battle is a `CatalogEncounter` built from the content catalog (Goblin Skirmish is the arena default; Goblin Ambush adds a mid-round wave); region-generated encounters will plug in the same way. Its combatants are the single source of truth for simulation and rendering.
- **Panels (`CombatPanel`)**: Party roster, enemy roster, resolution log, unit inspector, combat grid, tactical forecast, playback controls and timeline are independent panels bound to the session, each wrapping an existing view component. The inspector, roster and grid all render the replayed snapshot when reviewing a past block.
- **Static Dock Host (`CombatDockHost`)**: Fixed regions (top row, left/center/right, bottom bar) filled from data-driven layout presets (`Standard`, `Mirrored`). The game UI stays static; **layout edit mode** (`⚙` / `F9`) overlays an `⇱ Undock` tag on each panel without moving anything, floats panels into `UiWindow`s (their dock keeps a same-size placeholder) and docks them back when closed. The host also owns the full-screen modals and shortcuts (`Space` play, `Q`/`E` blocks, `[`/`]` log, `L` log modal, `F`/`Tab` timeline, `G` party & stash, `Esc` close).
- **Custom SVG Visual Assets**: Custom vector graphics for characters, unit tokens, and grid tiles.
- **Allied & Enemy Status Roster Docks**: Real-time HP bars, plan status (`○ no plan` / `✔ n` / `⚠ n`), AP, stance and position.
- **Planning Phase (on the combat grid)**: Each round starts in PLANNING and you plan right on the main screen. Select a hero (click the token or roster card; cards show `○ no plan` / `✔ n` / `⚠ n` and AP), then click a target: an **enemy** opens a small popup of attack skills, **your own token** lists Guard/Brace/Focus/Rest, an **empty tile** offers *Move here* (the exit offers *Withdraw*). Each row shows `AP · blocks`; hovering a row (or just hovering a target) previews the route, destination ghost, weapon reach and cost on the grid, and hovering inspects the unit in the side panels. Pick with the mouse, `1`-`9`, or `Enter` / clicking the same target again for the first option; **right-click the grid or press Backspace to undo the hero's last step**; the step lands on the lowest free block and the hero's ghost trail and the timeline update. The timeline bar shows the hero's AP, next block and warnings, with **🤖 Draft** (AI presets for the hero or party), **Clear** and **Confirm Round** (`Enter`); right-click a timeline command to remove its step and drag one onto another to reorder. **Engage** combines target + move: the moves to get in range are inserted automatically and walk the planned path (see the one-tile adjustment below). Hard-invalid sequences (no path, bad target, too many blocks) block Confirm; running out of AP only warns. Once confirmed the timeline is a read-only visualisation.
- **Round rules**: rounds are **6 blocks** (roughly three actions). Each unit has an **AP pool of 12** that carries across the battle and regains **+4 each round**; **Rest** (1 free block) recovers 2 AP and returns to Neutral. Leveled skills cost AP equal to their level (a level 2 Attack or Defend always costs 2 AP unless an effect shifts it via `Combatant.ap_cost_modifier`). **Moving must come first** and happens once: a manual move, a move-to-tile or the approach of an Engage, up to **2 blocks** at the unit's speed-based tiles per block, 1 AP per tile. A 2-block move is split per block: its first stretch lands at the end of block 1, the rest at the end of block 2. A move blocked at resolution (e.g. an enemy stepped into the path) stops on the last free tile, refunds the tiles it didn't walk and logs **CUT SHORT** with what blocked it; a second half re-paths toward the destination if the first half was cut short. Allies plan around each other's planned moves: tokens stay on their real tiles and each planned move shows as a **clickable shadow** at its destination (click your own shadow for self skills; a shadow on a tile that is still occupied shrinks into the corner). A unit may take a tile an ally leaves in block 1 if that ally acts first: click that ally's token to get *Move here* or *Plan for `<ally>`*. Attacks need the target in the skill's reach: melee 1, **ranged single-target attacks 2-3** (or the unit's longer range; an **Off-hand Jab** covers distance 1, with the off-hand weapon or bare hands); area attacks have their own reach (Volley 2-3, Firebomb 2, Shockwave adjacent). Attacking a different target than the current lock inserts a free **Switch Target** block. A target already expected to fall can still be attacked (insurance, flagged in amber). `AiProfile.skill_cap` (and an optional known-skills list per unit) stands in for skill progression. Rounds, AP pool and regen are per encounter (`total_blocks`, `unit_ap`, `ap_regen`).
- **AP debt instead of mid-round stops**: overspending is allowed; the unit finishes its sequence in AP debt and pays next round: the debt comes out of its +4 recovery, **3+ debt Staggers** it (blocks 1 and 4 lost) and **5+ Stuns** it (blocks 1-2 lost). The planner shows the projected debt and penalty. Engage approaches **walk the planned path** (no live re-chasing); when a **targeted attack** starts it adjusts by one tile if the target shifted out of its reach band (1 AP, may go into debt, shown as its own *Chase* log cell): melee steps in, a ranged unit whose target stepped point-blank **backs off** to shoot. Only damage-focused attacks adjust: after a hand-picked tile move (Move here / Withdraw) the unit holds its tile, and area attacks never move. Each approach destination is fixed when first planned, and units claim tiles in planning order, so allies (and the AI pack, which drafts into one shared plan) never plan onto the same tile. Confirming the round starts resolving it automatically; Play afterwards replays it.
- **Statuses (`StatusEffects`)**: Stagger, Stun and Root (no moving) last one round and always apply to the **next** round, so they reshape the next plan instead of breaking the current one; lost blocks show as locked `✖` blocks on the timeline. Skills roll to inflict them with a **seeded roll** (replays stay identical): chance = potency - the target's Debuff Resistance (INT) - any passive resistance (`Combatant.status_resist`; 100 = immune). New skills: **Shield Bash** (Defensive-stance strike, may Stagger), **Hamstring** (may Root).
- **Area attacks (tile-targeted)**: **Volley** (plus shape), **Firebomb** (3x3) at a tile in weapon reach, **Shockwave** (3-tile line from an adjacent tile). They hit everyone in the area, **allies included** (friendly fire is previewed in orange and warned). Click an empty tile or an enemy to aim; the preview shows the affected tiles.
- **Damage visibility**: attack rows in the popup and attack commands on the timeline show the expected hit (`~25 dmg`, `DamageCalculator.expected`: see 2b). Log cells show the damage actually dealt (`[HIT -28]`); in debug builds (`ActionLogEntry.show_formulas`) hovering the tag explains the formula.
- **Timing**: an action's effect lands at the **end of its last block**; its stance and AP apply when it **starts**; Defend protects from its first block. Within a block, higher priority (Switch Target, Defend) resolves first, then speed.
- **Layout**: the action timeline sits in the top row between the party and enemy rosters (its blocks shrink to fit the slot); the resolution log sits in the bottom bar next to the playback controls.
- **Unit Inspector**: portrait, HP and **AP** gauges (AP debt shown), speed, basic-attack reach band, stance, target, buffs and statuses, gear, and AI threat intel for hostiles.
- **Tactical Forecast Dock**: distance, reach-band check (in reach / too close / out of reach), stance matchup, the expected Strike hit and crit chance (section 2b), and the encounter's rules (blocks per round, AP pool and regen, exits).
- **Action Feed & Modals**:
  - Compact active block log dock.
  - Filterable fullscreen action stream modal (`ATTACK`, `DEFEND`, `MOVE`, `BUFF`, `EVENT` quick filters, plus a **custom filter** row to show a single unit's actions and the actions aimed at it). In debug builds hovering an attack cell shows its damage formula.
  - Synchronized multi-unit timeline modal.
- **Visual Combat FX**: Tweened attacker lunges, directional impacts, damage flash, stance pulse glows, and floating combat text (`-25 [MIT]`, `⚑ WITHDRAWN!`, `💀 DEFEATED!`).

### 6. Classes, Units, Proficiency & Passives (System 5)

- **6-Stat Architecture**:
  - **3 Active Stats**: `STR` (Physical Power, armor), `DEX` (Initiative Speed, Ranged Power, Crit), `INT` (Tactical & Skill Power, Debuff Resistance).
  - **3 Passive Stats**: `CONS` (Max HP), `PERC` (spotting, intent unmasking), `KNOW` (lore, forecast intel).
- **Derived Combat Metrics**: Max HP = 60 + CONS x 8 + STR x 2 (x tier HP scale); Initiative Speed = max(1, 8 + DEX - armor burden); Crit = 5% + 1% per DEX above 10 (+ training); Attack Power per section 2b.
- **Base rules (`StatRules`, enforced by the catalog)**:
  - Exactly **six base classes**: Warrior (tank), Ranger, Priest (support), Brawler, Rogue, Mage.
  - Every class has the **same level-1 stat budget (62)** (6-18 per stat) and the **same growth (5 points per level, max 3 per stat)**: classes differ only in distribution.
  - **Units of one class differ by their spend pool**: `4 + (level - 1)` free points, spent exactly (authored for NPCs / enemies; the player chooses for party members; empty = follow the class weights).
  - **Levels 1-10**; skill tiers unlock by level (SK2 unlock rule; AP is only the per-round budget).
- **Classes vs units**: a *class* is stats + gear + skills (stat distribution, growth, gear proficiency, skill path, default loadout, art, AI role), lore-free. A *unit* is a class + level + side (party / enemy / neutral) + **tier** (hero, minion, standard, elite, boss: scale HP and power) + spend + optional gear / extra-skill overrides. Enemies use the same class curve at a tier, so a goblin is *lore* on top of a minion Brawler; worlds set levels and explicit tuning deltas. Fixed-stat units (e.g. the training dummy) remain possible.
- **Gear proficiency** (`Proficiency`): an editable dictionary `{gear type: {level 1-3, scaling 1.0-2.0}}` per class. Unlisted gear can't be equipped. Levels give +1 Weapon Power each, a 2-point stat-requirement discount each, +1% crit per level above 1, -1 armor burden per level above 1, level 3 weapons cost one block less, and gate skills that need training (`SkillDef.weapon_types` / `min_proficiency`). `scaling` is how easy the gear is to wield: the multiplier on the scaling stat in Unit Power.
- **Skills**: each class has a **skill path** (`[{level, skill}]`); a unit knows the path up to its level plus `extra_skills` and skills learned in play (`Combatant.learn_skill`, passives of type `skill`). The planner and AI only offer skills the unit can use (`Combatant.can_use`).
- **Passives (replace mastery)**: lore-free templates in `data/catalog/passives.json` with effects `stat` (optionally only in a context, e.g. ruins), `proficiency`, `resist`, `ap_regen`, `skill`. Classes and base units grant none: **sources** (a world character's own list, faction `grants`, discipline `passives`, named item `grants`; later quests and lore) do, via `WorldFactory.character_passives`.
- **Progression**: guaranteed class growth each level plus 1 free point per level; bonus points from events (`add_free_attribute_points`).

### 7. Equipment, Requirements & Inventory Engine (System 7)

- **Item Taxonomy & Wield Rules**:
  - **Main-Hand Weapons**: Swords, Greatswords, Hammers, Bows, Staves. Plain 1H main-hand weapons may also be dual wielded in the off-hand.
  - **Off-Hand Weapons & Auxiliaries** (off-hand slot **only**, enforced by `Equipment.can_equip_in_slot`):
    - *Shield*: Pure damage mitigation (`mitigation_value`), non-offensive.
    - *Shortsword / Knuckles*: STR-based off-hand offensive strike.
    - *Knife / Dagger*: DEX-based off-hand offensive strike.
    - *Orb / Tome*: INT-based off-hand offensive strike / casting.
  - **1-Handed vs. 2-Handed Weapons**:
    - *1H Weapons*: Main and off-hand weapons are both ready at all times: `Main Strike`, `Off-Hand Strike`, or the `Dual Strike` combo that hits with both weapons in one action. No switching needed.
    - *2H Weapons*: Cannot be paired with a shield, but can carry an off-hand weapon as a backup. Only one of the two is active at a time; `Switch Weapon` (1 block) readies the other (`Inventory.active_weapon_slot`).
- **Stat-Gated Gear & Threshold Convention**: Equipment requirements follow `[Req <STAT level0>, <STAT level1>]`, where `level0` enables wielding without penalty and `level1` unlocks bonus effects (`+Wep power`, `+<STAT>`, or special combat effects).
- **Dynamic Deficit Scaling**: Wielding weapons with a stat deficit applies a dynamic penalty ratio (`minf(0.60, deficit * 0.15)`) to effective weapon power and delays timeline action blocks.
- **Class Archetype Gear Restrictions**: Class archetypes enforce allowable gear proficiencies (`can_equip_gear`); accessories have no restrictions and can be equipped by any class.
- **Accessory Stat Propagation**: Equipped accessories (rings, amulets) propagate stat bonuses directly into character stats, dynamically updating effective attributes, max HP, and attack power.
- **Unit Burden (per unit)**: The sum of a unit's equipped gear `burden`. It only slows that unit: initiative speed `8 + DEX - burden`.
- **Party Carry (party-wide)** (`PartyCarry`): One shared capacity `30 + Σ (STR × 3 + CONS)` over all members, against the party's total load (stash items + every member's equipped gear + battle pouches).
  - *Light* (< 75%): no effect. *Heavy* (75–100%): warning. *Overloaded* (> 100%): **every** unit loses 1 initiative speed per 10 weight over capacity (`Combatant.carry_speed_penalty`).
- **Battle Pouches**: 3 combat quick-slots per combatant for consumables used in combat (`ItemCommand` in Utility Stance).
- **Damage Types**: Items carry a `damage_type` (slash, pierce, blunt, magic, fire), used by stash filtering now and reserved for resistances later.
- **Party & Stash Screen**: Three reusable columns bound to one `DojoSession`:
  - (A) **Party viewer**: always 4 member slots (empty slots shown as placeholders so the layout never shifts), click to select the active unit, the shared party carry bar, and Unit Inspector / Derived Stats tabs.
  - (B) **Gear loadout** for the selected unit with the battle pouch; `Equip` / `Swap` / `Load` open an overlay picker.
  - (C) **Party stash**: one-slot-at-a-time filter (`All`, `Main Hand`, `Off Hand`, `Body`, `Accessory`, `Consumable`; shields live under Off Hand), plus a **Custom Filter** overlay layered on top (stat it scales with / grants / requires, rarity, weapon or armor type, damage type, 1H/2H, "usable by this unit", name search, sort by name/rarity/power/weight). Cards show class restrictions, tier/deficit badges and weight, with one-click `Main Hand` / `Off Hand` / `Equip` / `+ Pouch` actions that follow the slot rules.
  - Opened from the arena unit inspector or the dojo `Stash` button; `Esc` closes it.
- **Static Layout + Overlays**: Panels never side-scroll or reflow to show more content. Anything needing more space (equip pickers, stat allocation, the custom stash filter) opens as an overlay window (`UiOverlayLayer`) above the host, cascaded and draggable, and closes itself when finished.

### 8. Dedicated Gear & Skill Dojo (`gear_dojo.tscn`)

- **Separated Scene Architecture**: Distinct from the combat arena (`combat_arena.tscn`, the production combat UI), `scenes/gear_dojo.tscn` is a dedicated workbench for gear equipping, stat inspection, and damage formula validation.
- **Headless Session + Modular Windows**: All dojo state and rules live in `DojoSession` (`RefCounted`, no UI). Six independent panel components render from it, each hosted in a floating `UiWindow` inside `DojoWorkspace`. Panels never talk to each other, only to the session, so any of them can be closed, moved, or reused elsewhere.
- **Window System**: Drag by the title bar, resize from the bottom-right grip, collapse with `–` (or double-click the title), close with `✕`, and reopen from the toolbar `Windows:` toggles. Clicking a window raises it. Three layout presets (`Workbench`, `Combat Focus`, `Gear Focus`) plus `⟲ Re-tile`; window geometry is stored as fractions of the desk and persisted to `user://gear_dojo_layout.cfg`, so layouts survive restarts and window resizes.
- **The Six Components**:
  1. **Unit Inspector**: portrait, level/HP, the 6 base attributes with BBCode hover breakdowns (base, passives, gear, buffs) and gear proficiency. `Unspent Points` opens the **Allocate** overlay (the `-` button only undoes newly allocated points; `Reset`/`Confirm`/`Cancel`), so the inspector itself never changes layout.
  2. **Unit Gear**: Main Hand, Off Hand, Body Armor, Accessories and the 3-slot Battle Pouch as cards. `Equip`/`Swap`/`Load` open an overlay picker filtered to class-eligible stash items valid for that slot; `Unequip` returns gear to the stash. **Quick Setup** applies a one-click loadout preset (Sword & Board, Claymore Bruiser, Claymore + Backup Blade, Titan Hammer deficit test, Dual Wield, Archer with dagger backup, Blade & Dagger, Arcanist), spawning missing pieces and reporting items the class cannot wield. `Stash Manager` opens the Party & Stash screen. Cards for a 2H main + off-hand weapon show which weapon is `ACTIVE` or `sheathed`.
  3. **Combat Region**: hero and dummy tokens on a target mat with live HP bars, stance, guard, Focus, AP and statuses (the dummy's strike power and queued statuses too), lunge, shake and glow animations, floating combat text, plus `Toggle Dummy Defend`, `Full Heal Dummy` and `Dummy Attacks` controls.
  4. **Test Console**: a **catalog skill picker** (every Attack / Defend / Focus skill plus Rest from the content catalog, so skills made in the Content Forge can be tested here) with its description, AP cost (incl. the automatic +1 stance switch), resulting stance and expected hit on the dummy; **Use Skill** applies it with the battle rules (AP spent, debt allowed; stance switch; attacks use the skill multiplier and roll its status on the dummy; Defend skills guard until the next round; Focus boosts; Rest recovers AP); a **Round / AP** line with **Next Round** (Focus ticks, guards drop, statuses roll over, +4 AP); **weapon tests** (Main / Off-Hand / Dual Strike / Ranged Shot, auto-gated by the equipped weapons, `Switch Weapon (1b)`, `Stance Shift`); **Dummy Attacks** (the dummy's attack skill and strike power, attack on demand, **Counterattack** after every hero skill, `Heal Hero`); `Force Crit`; the battle pouch; and **Benchmark ×10 / ×100** of the selected attack skill (min/avg/max, crit count, expected damage per swing, swings to defeat the dummy).
  5. **Derived Stats**: melee/ranged/off-hand power (weapon + unit power), initiative speed (burden and party overload shown separately), max health, armor rating and its damage cut, unit burden, party carry (with load bar), attack range, crit rate, and debuff resistance. Hover any row for the formula with the live numbers substituted.
  6. **Test Log**: renders every dojo event with the same action-cell UI as the combat log (`CombatLogModal.format_action_cell`), with category filters and a `Clear` button. Click an entry to see its step-by-step damage breakdown in the detail pane.
- **Dojo Combat Math**: every dojo strike goes through the game's one damage formula (`DamageCalculator.compute`, section 2b) with the Dojo's own seeded RNG for crits; the Test Log shows its step-by-step breakdown (weapon power, unit power, Focus, skill, stance vs the dummy's stance, crit, guard, armor). **Stance Shift** cycles Neutral > Offensive > Tactical > Defensive; the dummy guards in Defensive stance and has no armor. The dummy's own attacks (authored strike power 20 by default, adjustable in the console) go through the same formula against the hero's guard, armor and stance.

### 9. Content Catalog & Content Forge (`content_forge.tscn`)

- **One catalog for all content** (`data/catalog/*.json`): skills, items, classes, units (party members and enemies) and encounters are data, not code. Every object that existed in code was seeded into it unchanged (28 skills, 22 items, 9 passives, 6 classes, 13 units incl. the Skirmish and Ambush goblins, the Gear Dojo heroes and the training dummy) plus 3 encounters (Goblin Skirmish, Goblin Ambush, Veterans' Drill), and the engine reads it: `SkillCatalog`, `ItemCatalog.create(id)` (the old `create_*` helpers are shorthands), `ClassArchetype.create(id)`, the party starter stash (`starter_stash` flag), every battle (`CatalogEncounter`) and the Gear Dojo heroes/dummy.
- **Schema-driven** (`ContentSchema`): each kind is a field list (type, default, range, enum options, references, conditional fields such as attack-only or consumable-only fields, help text). It normalises JSON (types, defaults, key order), drives validation and builds the editor form. Enums are stored by name (`"ATTACK"`, `"MAIN_HAND"`), so the files read and diff well.
- **Validation** (`ContentCatalog.validate_entry / validate_all`): snake_case unique ids, enum values, references that resolve and fit (a party roster only takes party units, an off-hand slot only off-hand gear), skill rules (engine-only kinds, reach bands, LINE areas, status potency), item slot rules, two-handed weapon + shield, derived units needing a class, encounter rosters and exit tiles. In-use and engine (`system`) entries can't be deleted; renaming an id rewrites every reference to it.
- **Units**: `fixed` vitals (HP / speed / range, like the goblins) or `derived` from class + level + gear (like the dojo heroes); loadout per slot plus battle pouch; AP pool; strike power (0 = weapon + unit power) and armor rating (-1 = gear + STR / 4); skill cap and an optional **known-skills list** (only those leveled skills are offered to the planner and the AI; Move, Rest and other engine skills are always known); AI role, targeting, flee / guard thresholds; per-status resistances.
- **Encounters in the real engine** (`CatalogEncounter`): any encounter entry runs through the full combat stack (planning, AI, resolver, replay, UI). Repeated enemies are numbered (`Goblin Archer 2`); units the centre-ring deployment can't fit go to the nearest free tile on their side. One **reinforcement wave** per encounter (`wave_units`, `wave_round`, `wave_block`, `wave_label`) enters mid-round. `combat_arena` loads one from the command line: `godot --path . res://scenes/combat_arena.tscn -- --encounter=veterans_drill`.
- **Content Forge** (Scene Runner > Content Forge): kind tabs; searchable entry list (side filter for units, ⚠ marks invalid entries, 🔒 engine skills); a form generated from the schema with dropdowns for enums and references, add/remove/reorder lists, stat grids and JSON fields; an **engine preview** of what the entry becomes (skill cost / reach / damage, item stats and which classes can use it, class stats at levels 1/5/10, unit vitals / power / skills / gear, encounter deployment) with its icon or portrait and "used by" references; live problem list. **New**, **Duplicate** (use any existing entry as a template), **Delete** (click twice), **Apply** / **Revert**, **Save** (`Ctrl+S`, writes the JSON), **Reload**, **Validate All**. **▶ Playtest** runs the selected encounter (or a unit in a sandbox fight on its side) in the real combat arena, with *Back to Forge* to return.
- **Reuse in tests**: tests build units and encounters straight from the catalog (`ContentFactory.unit(ContentCatalog.get_entry("units", id))`, `CatalogEncounter.new(id)`). Tests that write call `ContentCatalog.fork_to("user://...")`, so the shipped catalog is never modified.

### 10. Scene Runner (`scene_runner.tscn`, main scene)

- **One launcher** (`run.bat`, or Play in the editor): a menu listing every runnable scene: Combat Arena (with an encounter picker over the content catalog, remembered between runs), Content Forge, Gear & Skill Dojo, and any other `.tscn` dropped into `scenes/` (shown under its file name). Scenes run inside the runner, so returning to the menu is instant.
- **In-scene controls**: a small **☰ Menu / ↻ / ⟳** pill (bottom-right) and keys **F10** (menu), **F6** (restart the scene), **F5** (reload edited scripts, then restart). Leaving the Content Forge with unsaved catalog edits asks for a second press and then discards them.
- **Hot reload** (`ScriptReloader`): remembers each script's modification time and recompiles the edited ones in place (`GDScript.reload`), clearing static caches built by the old code and re-reading the catalog JSON. New files or new `class_name`s can't join a running game's class registry: the menu lists them and **⏻ Restart runner** starts a fresh process.
- **Test suites**: **▶ Arena suite / ▶ Dojo suite** run `test_arena.tscn` / `gear_dojo.tscn` headless in a separate process (the menu stays usable) and show the PASS / FAIL lines with a summary.

### 11. World Engine & World Forge (`world_forge.tscn`)

The Content Forge catalog is the **lore-free base library** (skills, items, classes, units, encounters). Worlds are a second layer: they give those assets names, backgrounds, factions and places, and never carry gameplay of their own.

- **Worlds are folders** (`data/worlds/<id>/`): `world.json` (name, tagline, description, `extends`, `connects`) plus one JSON file per element kind. Kinds: **regions**, **locations** (exits, encounters), **factions** (relations), **characters** (party, NPCs, enemies), **named items**, **disciplines**, **quests** (steps, requirements, rewards), **lore entries** and **encounters**. Entries have local snake_case ids; the **qualified id** is `world:id`.
- **Base class + lore** (`WorldFactory`): a character names a **base unit** (`base_unit`), a named item a **base item** (`base_item`), a discipline a list of **base passives** (`passives.json`: gear training, stat edges, resistances, AP regen, skills). The base asset decides how it plays; the entry adds name, title, background, personality, faction, home, art. Two characters (in one world or two) can point at the same base unit with different lore and behave identically: same vitals, class, skills, AI, damage.
- **Explicit divergence only**: a character may add `tuning` deltas (`hp_pct`, `power`, `armor`, `speed`, `ap`), named items (worn on top of the base loadout, one per slot) and inherited **passives** (the character's own, its faction's `grants`, its disciplines' and its named items' `grants`). Item tuning (`power`, `mitigation`, `burden`) and extra stat bonuses likewise. The World Forge shows **"Differs from base"** and the base unit's numbers side by side, so you can always see where a world deviates (Captain Dredge: hp +20%, power +2, armor +1).
- **Inheritance**: `extends` lists parent worlds; `lineage()` is the load order (ancestors first, the world last). A world sees every ancestor entry. Unqualified references resolve in the entry's own world, then up the lineage (nearest wins); `world:id` is explicit. Everything is namespaced, so two worlds never collide.
- **Overrides**: an entry with `override: "ashlands:mara_voss"` replaces that ancestor entry **for its own world and every world below it** (the parent is untouched), and every reference to the old id (even the old id used in an inherited encounter) now resolves to the override. This is how one world re-skins another: `tidewrack` overrides Mara, Tev and the five goblins, so **playing the inherited `tidewrack:courtyard_skirmish` is the same battle with different people**.
- **Connecting worlds**: `connects` links worlds without inheriting: a location may exit into `ashlands:warden_keep` from `iron_reaches`; qualified references to a world that is neither an ancestor nor connected are validation errors. Connections are inherited by child worlds.
- **Safe editing** (`WorldCatalog`): validation reports unknown base assets, role mismatches (a party roster only takes party characters), duplicate references, slot clashes between named items, unknown tuning keys, override targets, circular regions / inheritance. **Blocking errors** (ids, enums, json shape, override validity) stop an entry from being stored; **unresolved references** don't (worlds are full of mutual references: two locations exiting into each other), they are listed as problems, shown with a warning mark, and checked by *Validate All* and before playing. Renaming an id rewrites every reference to it (plain refs, json refs such as exits / quest steps / faction relations, override pointers) across the world, its descendants and connected worlds; in-use entries and extended worlds can't be deleted.
- **Playing worlds**: `WorldEncounter` runs a world's encounter in the real combat engine with the world's characters (lore names in the roster, log and inspector, their named items and tuning); `EncounterSource` lists catalog and world battles together (`tidewrack:strand_clash`), so the **Scene Runner's arena picker** and `combat_arena -- --encounter=ashlands:pass_ambush` run them. Quests, dialogue and map travel are authored as data now and become playable with those systems.
- **World Forge** (Scene Runner > World Forge): world picker, **+ New World** (id, name, parent), element tabs plus an Overview (manifest, inheritance, counts; the left list shows every world with its parents, click to switch); a merged entry list where inherited entries are marked with their origin world and are read-only, with **New / Duplicate / Override / Delete**; the shared schema-driven form (reference pickers show `Name [id]` plus the origin world); a lore preview with *Plays as*, *Differs from base* and *Used by*; live problems; **Apply / Revert / Save (Ctrl+S) / Reload / Validate All**; **Play this encounter** in the arena with a *Back to World Forge* button; unsaved edits are guarded by the Scene Runner.
- **Shipped worlds**: `ashlands` (root: Ember Wardens vs the Ashen Horde, the Courtyard skirmish and a wave ambush), `tidewrack` (extends it: Harbour Guard vs Reef Raiders, overrides, own coast and battle, one tuned boss) and `iron_reaches` (connected: a guild road into Warden Keep).

---

## Headless Testing & Automated Quality Assurance

All features are tested headlessly without requiring a graphical window or GPU.

The suites registered in [suite_registry.gd](scripts/tests/suite_registry.gd) (32 suites, about 300 assertions with the dojo; `check_project.ps1 -Fast -Quiet` prints the current count) run headlessly (each suite prints one line per assertion; the old hand-scripted 10-block battles were retired in favour of the Core Combat Rules suite):

1. **Combat UI Framework & Behaviour Test** (8 assertions): all 8 panels mounted in their dock regions, tokens & roster bound to the simulated combatants, selection → inspector, tactical forecast, log & timeline modal lifecycle, timeline block inspection, undock/dock + layout presets + action FX dispatch.
2. **Core Combat Rules Test** (7 assertions, [core_rules_test_suite.gd](scripts/tests/core_rules_test_suite.gd)): planned exactly like the game and resolved by the real resolver: the one damage formula (weapon + unit power, unarmed STR + DEX, unsuited weapons, authored power, armor curve and gear rating, x1.5 crits with their chance, seeded crits repeating, the planner's estimate = the formula), Brace mitigation with the Defensive-vs-Offensive matchup (exact damage, reading the seeded crit roll), stances persisting after resolving and into the next round (Shield Bash stays Defensive), Focus lasting its rounds then expiring, a defeated unit losing the rest of its round, Withdraw leaving the grid and ending the battle, and the full Goblin Ambush feed (moves, attacks with target HP, buffs/guards, the R1-B04 wave, withdrawal, defeats).
3. **Character Specs & Inventory Test** (10 assertions): Archetype 6-stat baselines (equal budget), derived attributes, level-up guaranteed growths, free point allocation, event point rewards, proficiency training and contextual passive bonuses, equipment stat requirement check, threshold tiers boost, deficit wield penalties, and battle pouch consumable usage in combat.
4. **Gear Equip UI & Test Dummy Suite** (10 assertions): Equipment modal mounting, hero switching, stash equipping, Claymore Threshold Tier 1 (+4 Power) boost, Titan Warhammer deficit penalty, armor burden speed penalty, unequipped item return to stash, training dummy 500 HP baseline, base strike and 50% defense mitigation formula, battle pouch consumable damage & removal.
5. **Combat Session Flow Test** (8 assertions): the catalog Goblin Ambush through `CombatSession` headlessly: only deployed enemies in view before the wave, the wave event at `R1-B04`, Round 1 resolving into replay with round advance offered, replay by log points, state carried into Round 2 with past rounds reviewable, later rounds (AI presets) playing on to a conclusion, timer-free auto-play (including Play pressed while planning), and round reset.
6. **Art, Terrain & Zones Test** (9 assertions): team-colored class tokens and portraits, encounter art follows sides, terrain glyph round-trip and walkability, all 8 zones valid, grid blocks blocked tiles, the Goblin Ambush (wave included) deploying and concluding on every zone, tile and item icon rasterization.
7. **Planning, Stances, AP & Enemy AI Test** (16 assertions): reach bands and the off-hand jab, move-first, Root, seeded status rolls and immunity, area friendly fire, allies moving into vacated tiles, AP debt penalties (Stagger/Stun blocks), on-grid input options (per target / tile), round rules (6 blocks, AP pool + regen, one move per round, centre tie-break for equal routes, target switching, insurance attacks, attack follow, damage in the log, auto-play on confirm), skill AP = level (Move per tile, Rest free) with effect modifiers and speed-based tiles per block, the rock-paper-scissors stance matrix and automatic +1 AP switch costs, sequences compiling to the lowest free blocks with Engage inserting 1-block moves, mid-sequence stance switching, projected "runs dry" instead of refusal, flagged invalid edits blocking Confirm, Confirm scheduling party and AI plans with AP spent at runtime, units stopping when they can't pay, Rest recovering AP, AI presets, role patterns (walls, sniper retreat, support Focus, flee, Engage chasing a moved target), deterministic AI-vs-AI skirmishes on all 8 zones, the one-tile chase only for targeted attacks (not after a tile move; ranged units back off), and round reset refilling AP.
8. **Content Catalog & Forge Test** (9 assertions, [content_test_suite.gd](scripts/tests/content_test_suite.gd)): the shipped catalog loads and validates; the engine's skills, items, stash, classes and Goblin Skirmish come from it; skills, items and classes round-trip through engine objects unchanged; validation catches bad ids, kinds, reach, slots, references and rosters and refuses unsafe deletes; on a sandbox copy: authoring a skill, item, class, units and an encounter, saving, reloading, renaming an id (references follow) and duplicating as a template; authored units bring class-derived vitals, gear, numbered duplicates, whitelisted skills and resistances into planning; the authored encounter (and the shipped Veterans' Drill) plays out in the full engine using the new skill; and the Forge UI (template, conditional fields, refused invalid draft, delete, new unit, playtest in the arena and back, save).
9. **Scene Runner Test** (5 assertions, [runner_test_suite.gd](scripts/tests/runner_test_suite.gd)): every runnable scene listed (runner and test harness hidden), suite output parsing, launching the arena with a chosen catalog encounter plus restart and back to the menu, the unsaved-Forge-work guard (second press discards), and hot-reloading a script in place.
10. **World Engine & World Forge Test** (9 assertions, [world_test_suite.gd](scripts/tests/world_test_suite.gd)): the shipped worlds load and validate (every element kind, lineage, descendants); inheritance (namespaced ids, overrides replace for the child only and old ids redirect, nearest-wins resolution, cycles and unknown parents rejected); same base / different lore (names differ, every stat, class, skill, AI plan and damage matches between `ashlands` and `tidewrack`, except the deliberately tuned boss); tuning deltas, named items (armor + mitigation, fixed vitals untouched) and disciplines apply exactly; world encounters (wave ambush, own battle, an inherited battle re-skinned) play to the end in the real engine with lore names, and `EncounterSource` lists both layers; on a sandbox copy: connections without inheriting, unconnected worlds refused, role / base / override / slot checks and safe deletes; authoring (new world, dangling references while drafting, overrides, renames rewriting plain / json / cross-world / override references, templates, save & reload); and the World Forge UI (create world, inherited read-only, override, dangling refs, tuning, play in the arena, save, delete) plus the Scene Runner launching a world battle.

Additionally, `scenes/gear_dojo.tscn` runs **42 dedicated headless assertions** ([dojo_assertions.gd](scenes/dojo/dojo_assertions.gd)) when launched with `--headless`:

- **Rules & formulas (Dojo-1 to 18)**: dummy vitals, melee formula with sword training, 50% mitigation, focus buff scaling, threshold tiers, deficit penalties, melee/ranged power gating, accessory stat propagation, class archetype equip restrictions, 1H/2H and off-hand scaling, crit rate and damage factor, stat allocation bounds, and reset to baseline.
- **Session behavior (Dojo-19 to 25)**: Quick Setup presets, class-proficiency skipping, 2H shield refusal / off-hand weapon acceptance, 1-block active-weapon switching, 1H dual-wield Dual Strike, stash duplicate integrity, strike log entries with breakdowns, rejected illegal actions logged as `FAILED`, and benchmark simulation leaving dummy HP untouched.
- **UI smoke tests (Dojo-26, 27)**: all six panels mount in the workspace and the Test Log mirrors the session log; the Party & Stash screen shares the session (party list, selection, stash equip).
- **Slots, filters & carry (Dojo-28 to 32)**: off-hand-only weapons and shields, per-slot stash filters (shield under Off Hand) plus stat/rarity/damage/hand/usable/search/sort custom filters, a party viewer that always shows 4 slots, per-unit burden vs party-wide carry overload, and overlay windows opening/closing on request.
- **Catalog skills & the dummy fighting back (Dojo-33 to 38)**: the console's skills are the catalog's; a catalog attack skill (Hamstring) follows the battle rules (2 AP + 1 switch, Offensive, the exact seeded hit); Brace guards and the dummy's attack goes through the same formula (its power and stance vs the hero's guard and armor) and is logged as the dummy's; Counterattack answers every hero skill; a stunned dummy loses its action and Next Round ticks Focus, drops the guard and regains +4 AP (debt included); Rest recovers AP and a downed hero is refused until healed.

### Combat extensions (M1)

Support skills (heal/buff on allies, `SupportRules`), gap-close attacks (`dash_tiles`), haste/bleed/taunt statuses, `learned_only` skills, a `reckless` AI profile and AI fixes (kiting, no idle turns, support healing); tested in `combat_extensions_test_suite.gd`. Example skills `mend`, `rally`, `taunt_shout`, `lunge`, `bleeding_cut` are in the catalog but in no class path yet (M2).

### Riposte (M2a)

A defensive self-buff skill (`riposte`, mitigation 0.15) grants the `riposte` status for 2 rounds: adjacent melee attackers are countered for 50% attack power (through `DamageCalculator.compute`), once per attacker per round; any stance change ends it. Tested in `riposte_test_suite.gd`; snapshots now store statuses so undo rewinds them.

### Gear budget (M2d)

`GearBudget` gives every item one weighted value (power, mitigation, stat bonuses minus burden and extra block cost) and checks each class's default loadout against a target (23, band +-20%). Warnings only (never errors); `special` items with a `special_note` may exceed the budget. Shown in the Forge class and item previews; tested in `gear_budget_test_suite.gd`.

### New gear and placeholder art (M2b)

Eight new items for the reworked roster (cloth robes, cleric vestment, mage robe, tower shield, wand, mage staff, cleric mace, and the special `wpn_glass_wand`), none in the starter stash; placeholder `priest`, `mage` and `rogue` token/portrait keys and a wand icon. Not yet in any class loadout (M2c).

### Reworked roster (M2c-1)

Six classes: Warrior (tank, special tower shield), Ranger, Priest (support), Brawler, Rogue, Mage. Loadouts sit in the gear budget band (the tower shield is the documented special). Goblin units now use the Priest, Brawler (reckless) and Rogue classes. Skill paths use existing skills for now; the new skill set is M2c-2.

### Skill families and loadouts (M2c-2a1)

Skills are grouped into families with tiers chosen at planning time (higher tier = more AP and power). A unit has an active loadout of at most 5 class families plus 3 extra families (basics free); it is set outside combat and locked in battle. Existing units and numbers are unchanged. Tested in `skill_family_test_suite.gd`.

### Engine pieces (M2c-2a2)

`cleanse` (support skills can remove debuffs from an ally, combinable with heal and buff), the `DISENGAGE` jump (move away from the nearest foe), the L-shaped `SWEEP` area for the Brawler, and a gear-budget value for unarmed loadouts. Example skills `purify`, `backflip`, `sweep_kick` are in the catalog (learned_only). Tested in `engine_pieces_test_suite.gd`.

### Foundation cleanup (slice A)

Basic skills are now six single-tier skills (strike, guard, move, rest, switch_target, offhand_jab); the old variants (power strike, heavy blow, brace, fortress, deep focus) are gone. New: a `StatusRegistry` for all statuses and stacking rules, the `unarmed` skill token, derived weapon hands (`main_1h`, `main_2h`, `off`), and a `skill_potency` passive hook. Tested in `status_registry_test_suite.gd` and `foundation_test_suite.gd`.

### Rest and stance (B1)

Rest now puts the unit in Tactical stance (free entry). The skill after Rest pays the usual `CombatTypes.STANCE_SWITCH_COST` (1) only if its stance differs: Rest then Focus has no switch cost, Rest then an attack or Defend costs +1.

### Orb and tome, item effects (C1-L)

Orb and tome are off-hand-only focus weapons; items carry an `effects` list (spell/heal potency, status potency, spell reach, resist) that applies only while equipped; a two-handed main weapon cannot be paired with them. The Dojo Arcanist preset is now wand plus tome. Tested in `focus_gear_test_suite.gd`.

### Weaken and data-driven riposte (D1-L)

New `weaken` status (outgoing skill power -15 to -25%, cap 25%, refresh to the longest duration) with a `status_magnitude` skill field; zero-power attacks apply a status without dealing damage; riposte AP, mitigation and duration now come entirely from skill data. Tested in `weaken_test_suite.gd`.

### Area statuses and taunt from defend skills (M2C2B-L)

Area attacks now roll their status once per unit hit (allies included); defend skills can grant riposte (immediate) or a support buff like taunt (queued). Tested in `skill_engine_test_suite.gd`.

### Progression and saves (M3)

Party state, XP curve (starting values in `data/progression/xp_curve.json`), level-up spend, a builder that turns a saved unit into a combatant, and a versioned single-slot JSON save with migrations (`user://saves`). No UI yet. Tested in `progression_test_suite.gd` and `save_test_suite.gd`.

### Class skill content (M2C2B)

All six classes now have their real kits: three-tier skill families, eight advanced families that replace two starters in one slot, intimidate for the Warrior, riposte at AP 3/2/1, orb and tome focus items with effects, and rogue and mage party units. Default loadout = the first five path families. Authored by idempotent seeds in `data/seeds/`.

### AI skill use (AI1)

The tactical AI (enemies and preset drafts) now uses disengage, area attacks (never hitting its own side), cleanse-only skills, taunt, ally buffs, debuff attacks and riposte. `AiProfile.smart_skills` switches it off for A/B balance runs. Tested in `ai_skills_test_suite.gd`.

### Skill unlock rule (SK2)

Skills open by unit level: a tier is castable at the family unlock level plus a per-tier offset (0, 2, 4). AP is only the per-round budget. Advanced families need level 5 or higher; KNOW and the new `unlock` passive effect can lower unlock levels (max 2). Unlock levels freeze during battle. Tested in `skill_unlock_test_suite.gd`.

### Atomic family edits (SK2-A)

`ContentCatalog.upsert_many` applies a set of entries all-or-none (a skill family whose ranks must change together). The SK2 data step flags the 11 advanced families, moves the warrior's riposte to level 3 and adds the passive templates `field_medic`, `arcane_study` and `riposte_drill`.

### Forge skill tab (SKILL1)

A skill is one **family template** plus up to two **rank modifiers** (absolute values resolved against the base, no cascade); rank names derive from the family ("Mend II"). `data/catalog/skills.json` is now version 2 (49 ranked entries on disk, still 127 flat rows in memory; `SkillRanks` folds and expands). Descriptions are templates with tokens (`{power}`, `{heal}`, `{rounds}`, ...; `SkillText`), caps 32 name / 240 description. New fields: `grade` (novice, advanced, expert reserved) replaces `advanced`, `requires` (all listed families, strictly lower grade) replaces `evolves_from` (seven pairings such as greater_mend requires mend), `weapon_hand` (main, off; `both` disabled) replaces `offhand`, and `origin` (system/user): system assets have key fields locked, can be cloned, and only user assets can be deleted (`promote` / undoable `demote`). Basic skills are granted to every unit from level 1 and are not valid in class paths. Saving never drops data: a kind that loaded with errors is save-blocked until fixed. `tools/fc.sh safe_delete <kind> <id>` scans references before a delete. The Forge skill form is driven by `SkillFormModel` (sections per kind, dynamic limits, Save-disabled reasons), `SkillFilter` and `SkillRanksPanel`. Migration seed `data/seeds/skill1_ranks.gd`. Tested in `skill_ranks`, `skill_text`, `skill_model`, `origin` and `skill_form_model` suites.

### Balance rules and sim v2 (BAL2-L)

- **Range**: the +1 attack range at PERC 18 (`CharacterStats.RANGE_PERC_THRESHOLD`) now applies only to ranged weapons (base range above 1); melee units stay at range 1.
- **Unit gear `none`**: a unit slot set to `none` (`ContentSchema.NO_ITEM`, pouch `["none"]`) opts out of the class default loadout; a blank slot still means "use the class default". Units only.
- **Tiers**: elite 1.25 HP / 1.1 power, boss 1.5 / 1.1 (`StatRules.TIERS`).
- **Trained fists**: an empty main hand with fist proficiency 2 or more hits with weapon power 15 + fist training and gets the fist crit bonus (`DamageCalculator.UNARMED_BASE_POWER`, `TRAINED_FIST_MIN_LEVEL`); `GearBudget.UNARMED_VALUE` reads the same constant. The damage formula itself is unchanged.
- **Balance sim**: `tools/fc.sh sim <duel|tier|levels|group|stats> [level] [seeds] [key=value ...]` (helpers in `tools/sim/`). Keys: `cap` (default 10), `smart_skills`, `loadout=default|newest`, `spend`, `preset`, `distance`, `score=both|hp|binary`, `quick`, `band=lo-hi`. Cells print `binary/hp-margin` scores; duel rows outside the 30-70 diagnostic band are flagged LOW/HIGH. Modes `party` (up to 4 vs catalog or generated mob/line/wave/elite/boss encounters), `role` (five limit-test scenarios per role, start distance 4, KPIs as mean [floor..ceiling] with ranking) and `sweep` (all 4-class compositions, swap delta, top-5 counts) use a per-unit metrics collector fed by each unit's `health_changed` signal; it self-checks every log entry against real HP changes and prints `METRIC FAIL` on any mismatch. Off-target values are marked X; `target=<key>:<lo>-<hi>` overrides a band. A full role run is about 28 min (split with `role=` or raise `SIM_TIMEOUT`).
- Tested in `bal2_rules_test_suite.gd`.

### Balance data pass (BAL2-A)

Seed `data/seeds/bal2.gd`: CONS growth 1 for ranger, rogue and mage (warrior CONS 2 STR 2); warrior carries a kite shield, and the gear budget fix D1 lowers kite mitigation to 6 and plate to 8 (warrior L5 EHP 468, every class budget 19-27); ranger starts with `aimed_shot` and brawler with `body_blow` at level 1; the shipped warrior, ranger, rogue and mage units carry explicit role loadouts; goblin_shaman fights with a cleric mace, robe, no off-hand and no potion; the brawler class is unarmed (trained fists).

Placeholder art (CLN-15, seed `data/seeds/cln15_art.gd`): goblin units now set their `art` key (archer, shaman, skirmisher, berserker, scout; goblin_brawler reuses berserker) instead of falling back to their class art.

### Cleanup batch 1 (CLN-3, 6, 7, 9)

Refactor only, no behaviour change: per-kind validation moved out of `content_catalog.gd` (1416 -> 993 lines) into `scripts/content/validators/` (`SkillValidator`, `ItemValidator`, `ClassUnitValidator`); the legacy `SkillDef.offhand` / `advanced` shims are gone (`is_off_hand()`, `grade`), and old skill keys or a version 1 skills file still load but print a deprecation warning; the second power formula `CharacterStats.get_melee_power` and other dead helpers were removed, so attack power comes only from `DamageCalculator`.

### Cleanup batch 2 (CLN-8, 9b, 14)

Refactor only, no behaviour change: shared test helpers live in `TestHelpers` (`report`, `report_ok`, `failed`, `unit`, sandbox `fork` / `cleanup` / `fork_worlds`; `SkillTestKit` folded in) and every test sandbox is unique per process (`TestHelpers.sandbox(tag)` -> `user://test_sandboxes/<pid>_<n>_<tag>`), so parallel suite runs no longer collide; the `ArenaTestSuite` pass-through is gone (`scenes/test_arena.gd` calls `SuiteRegistry.run_all`); `content_schema.gd` keeps its API while the per-kind field tables moved to `scripts/content/{skill,item,class_unit}_fields.gd`.

### Quest schema (M4-L slice 1)

Quest templates live in `data/quests/quests.json` (`QuestCatalog`); world quests bind them to world locations, encounters and flags and add metadata `links`, flag-conditioned `variants` (only whitelisted fields, so no branching inside a quest), a `turn_in` NPC and a `loot_pool` drawn from the new per-world loot table (`loot.json`). Validation and reference scans (incl. renames through links, variants and pools) are in `QuestRules` and `WorldCatalog`. The quest runtime and save v2 follow in slice 2. Tested in `quest_test_suite.gd`.

### Agent workflow tools

`CLAUDE.md` indexes `docs/` (game brief, roadmap, structure incl. placeholder folders, decisions, history of deprecated ideas, balance report, workflow) and defines the agent roles and parallel lanes (design-brainstorm, logic-builder, asset-builder, balance, change-reviewer; the main session plans via the `planner` skill, queue in `docs/TASKS.md`, specs in `docs/specs/`). `tools/fc.sh` gives compact checks (`parse`, `parse-all`, `arena`, `dojo`, `run`, `sim`, `safe_delete`, `all`); `check_project.ps1 -Fast -Quiet` runs the full check in ~30 s (see docs/EFFICIENCY.md), `tools/fc.sh docs` includes a markdown lint subset (`tools/md_lint.awk`), `tools/wt_remove.ps1` removes a finished worktree on OneDrive (clears the read-only folder flag git trips on) and `tools/balance_sim.gd` (`fc.sh sim duel|tier|levels`) runs headless AI-vs-AI balance tables. Claude Code skills (`godot-debug-checklist`, `patch-and-seed`) and agents (`design-brainstorm`, `logic-builder`, `asset-builder`, `balance`, `change-reviewer`, `housekeeper` for cleanup proposals) live in `.claude/`.

### Automated Check Script (`check_project.ps1`)

Runs in 4 phases:

- `[1/4]` Verifies global class cache and asset imports (`--headless --editor --quit-after 50`).
- `[2/4]` Validates GDScript syntax & static typing across every project script (`--headless --check-only`); a crash only while Godot shuts down after a clean parse is reported as a warning.
- `[3/4]` Executes the headless arena test harness (`--headless scenes/test_arena.tscn`), asserting exit code 0. (The Scene Runner's suite buttons run the same scenes.)
- `[4/4]` Executes the headless gear dojo suite (`--headless scenes/gear_dojo.tscn`), asserting exit code 0.

---

## How to Run

### Run Validation & Tests (Headless)

Run via PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File .\check_project.ps1
```

Or via Windows Batch:

```cmd
check_project.bat
```

### Run the Game (Scene Runner)

One launcher for everything:

```cmd
run.bat
```

Or open the project in **Godot 4.7+** and press Play: the main scene is the Scene Runner. From its menu:

- **Combat Arena** (pick any catalog encounter or any world's battle), **Content Forge**, **World Forge**, **Gear & Skill Dojo**, plus any other scene added under `scenes/` (listed automatically).
- In a running scene, the **☰ Menu** pill (bottom-right) or **F10** returns to the menu, **F6** restarts the scene, **F5** reloads edited scripts and restarts it.
- **⟳ Reload scripts** recompiles scripts edited on disk (the menu shows how many are pending); brand-new files or new `class_name`s need **⏻ Restart runner**, which starts a fresh process.
- **▶ Arena suite / ▶ Dojo suite** run the headless test scenes in a separate process and show their PASS / FAIL lines.

The Content Forge saves to `data/catalog/*.json` (run from the project, not an exported build). To run a catalog or world encounter directly:

```cmd
"C:\Program Files\Godot\godot.exe" --path . res://scenes/combat_arena.tscn -- --encounter=veterans_drill
"C:\Program Files\Godot\godot.exe" --path . res://scenes/combat_arena.tscn -- --encounter=tidewrack:strand_clash
```

---

## Roadmap & Features Yet to be Implemented

The following modular systems and capabilities are planned according to the master game design specification:

### 1. Planning Phase Follow-ups

*Status: Sequence planning implemented (chained inputs, validation, AI presets, skill-level AP, stance flow, live-chasing Engage).*

- [x] Sequence-based on-grid planning with chained inputs, validation, reorder/remove, AI presets, AP = skill level and stance flow.
- [x] CombatSession extracted from `test_arena.gd`; combat UI rebuilt as session-bound panels in a static dock host (`combat_arena.tscn`).
- [ ] Reactive defensive skills (Parry, Counter, Shield Bash: Defensive-stance skills that deal damage).
- [ ] Status effects (poison, intimidate, stagger) and Offensive debuff skills; Tactical buffs beyond Focus.
- [ ] Sequence-altering skills (Interrupt, Stagger delay, AP drain) with replay explanations of why a sequence changed.
- [ ] Per-step projected stance/AP annotations on the grid preview.
- [x] Ghost trail, destination ghost and reach preview of planned moves and attacks on the grid.
- [ ] Ally-targeted skills (the click-an-ally popup is ready; no ally skills exist yet).
- [ ] Reactive defense (Parry, Counter) and more status skills; AI use of area attacks.
- [ ] Skill and item commands in the planner (`SwitchWeaponCommand`, Dual Strike, battle-pouch items, level 4-5 skills) and real skill progression feeding `skill_cap`.
- [ ] Terrain gameplay rules: cover bonuses, movement cost and hazard damage (metadata already in `Terrain`).
- [ ] More bespoke icons for unique/rare gear (`Item.icon_id`).

### 2. Enemy AI Follow-ups (System 8)

*Status: Role-based AI planner implemented (Aggressor, Skirmisher, Sniper, Support, flee heuristic).*

- [x] AI intent evaluation: distances, weakest/nearest targeting, expected-damage tracking, BFS path-finding.
- [x] Role archetypes and the 25% HP flee-to-exit heuristic.
- [ ] Support abilities that buff or heal allies (needs ally-target commands).
- [ ] Coordinated behaviour: focus fire, flanking and formation keeping.
- [ ] Reactive intent: enemies reading the party's telegraphed plans (perception stat).
- [ ] Difficulty profiles and AI-driven reinforcement waves.

### 3. Asset Loading & Animation Test Suite (`Test Suite 2`)

*Status: Planned.*

- [ ] Dedicated automated test suite verifying SVG asset loads, dimensions, and texture caching.
- [ ] Animation lifecycle tests (Tween callbacks, duration timing, and queue free verification).

### 4. Advanced Combat Rules & Combo Engine (`System 6`)

*Status: Planned.*

- [ ] **Chained Flow Moves**: Skills that automatically transition into compatible follow-up stances.
- [ ] **Arena weapon commands**: `SwitchWeaponCommand` (1 block, toggles `Inventory.active_weapon_slot`) and a `DualStrikeCommand` combo for 1H pairs, so the dojo weapon rules apply on the round timeline.
- [ ] **Stance Disruption & Stagger**: Guard breaks that inflict the `Staggered` condition (unable to defend/dodge for $N$ blocks).
- [ ] **Area of Effect (AoE)**: Line, cone, and radial blast patterns on the 9x9 grid.
- [ ] **Environmental Hazards**: Fire, ice, and poison terrain tiles that apply effects on entry/end of round.
- [ ] **Status Conditions**: Bleed, Poison, Blind, Stun, and Haste.

### 5. Items & Consumable Inventory Engine (`System 7`)

*Status: Core models & interactive equipment UI implemented.*

- [x] Shared party bag container model and 3-slot combat battle pouches.
- [x] Stat-gated equipment with tiered thresholds, deficit penalties, and gear proficiency discounts.
- [x] Combat consumable usage and HP/buff application.
- [x] Party & Stash screen (`CharacterEquipmentModal`): party viewer, gear loadout and filterable stash in three columns.
- [x] Preset catalog (`ItemCatalog`) with weapons, armor, shields, and accessories.
- [ ] Equipment durability, weapon crafting, and socketed gems.

### 6. Character Progression & Stats Engine (`System 5`)

*Status: Core models & interactive equipment modal inspection implemented.*

- [x] 6-stat architecture (STR, DEX, INT | CONS, PERC, KNOW) and derived formulas.
- [x] 10-level cap with guaranteed class growths and free attribute point allocation.
- [x] Enforced class rules (`StatRules`): equal budget and growth, authored spend pool, tiers, level-based skill unlocks (SK2); editable gear Proficiency dict; Passives replace mastery (granted by world sources, never by classes).
- [x] Class skill paths by level plus event-learned extra skills (`Combatant.learn_skill`).
- [x] Integration with `Combatant` model.
- [x] UI Character Sheet & gear equip modal (`CharacterEquipmentModal`) and the modular Gear Dojo workspace (`gear_dojo.tscn`).
- [ ] Passive trait slot selection and talent trees.

### 7. Region Map & Exploration Engine (`System 2`)

*Status: placeholder; starting rules in `docs/ROADMAP.md` "Inputs for placeholder systems".*

- [ ] 2D top-down grid exploration with tile types (`SAFE` vs `DANGER`).
- [ ] Danger step counter triggering mob encounter rolls.
- [ ] Visible roaming elite/boss encounters with detection radii.
- [ ] Region exit point tiles that transition to the World Map.

### 8. World Map & Travel Engine (`System 1`)

*Status: placeholder; starting rules in `docs/ROADMAP.md` "Inputs for placeholder systems".*

- [ ] Regional node/route graph linking world biomes.
- [ ] Travel time consumption and road danger rolls.
- [ ] Seamless multi-layer transition: World Map $\leftrightarrow$ Region Map $\leftrightarrow$ Combat Arena.

### 9. NPC, Dialogue & World Story Engine (`System 9`)

*Status: placeholder; starting rules in `docs/ROADMAP.md` "Inputs for placeholder systems".*

- [ ] Branching dialogue trees with choices and skill checks.
- [ ] Global quest flag registry and world state persistence.
- [ ] Faction reputation meters and trading/merchant shops.

### 10. Gear Dojo Follow-ups

*Status: Modular window workspace implemented; extensions planned.*

- [x] Headless `DojoSession`, six panel components, floating window system, layout presets and persistence.
- [x] Quick Setup loadout presets and damage benchmarks.
- [x] Static panels with overlay windows; 4-slot party viewer; per-slot + custom stash filters; party-wide carry vs unit burden.
- [ ] Custom loadout save/load (name and store your own presets).
- [ ] Side-by-side gear comparison (hover a stash item to preview its delta on derived stats).
- [ ] Target dummy profiles (armor, resistances, per-stance behavior) and a multi-round combat sequence recorder.
- [ ] Optional native OS pop-out windows (`gui_embed_subwindows = false`) for multi-monitor setups.

### 11. Content Catalog & Forge Follow-ups

*Status: Catalog, validation, Content Forge and arena playtests implemented; content attributes to be fleshed out.*

- [x] JSON catalog for skills, items, classes, units and encounters, seeded from every existing object; engine reads it.
- [x] Schema-driven Content Forge: browse, template, edit, validate, save, engine preview and arena playtest.
- [x] Per-unit known-skill lists; class-derived or fixed vitals; catalog encounters loadable from the command line.
- [ ] Clean up and flesh out entry attributes (descriptions, missing stats, item/skill effects not yet modelled).
- [x] The scripted Goblin Ambush is now a catalog encounter with a data-driven reinforcement wave (the hand-scripted encounter and its legacy commands were retired).
- [ ] Explicit deployment positions per encounter, and a zone (terrain map) editor.
- [ ] Skill effects beyond the current kinds (heals, ally buffs, multi-hit, sequence-altering skills) once the engine supports them.
- [ ] Balance tools: batch-simulate an encounter N times and report win rates and damage.

### 12. Engine Consistency Follow-ups (from the cleanup audit)

*Status: open decisions found while retiring the legacy scripted layer.*

- [x] **One damage formula for arena and dojo** (section 2b): weapon + unit power, seeded x1.5 crits, percent armor with diminishing returns, used by battles, the planner, the forecast and the Dojo.
- [x] **Dojo skills from the catalog**: the console uses catalog skills with the battle rules (AP, stance, statuses, rounds), and the dummy can attack back. Dual Strike and off-hand weapon switching remain dojo-only weapon tests until they become catalog skills.
- [ ] **Gear in battles**: loadouts now drive attack power and armor in battles; weapon block costs and active-weapon switching still don't affect the arena.
- [ ] **Fixed units' stat block**: units without a class get the default stat sheet (the Warrior archetype at level 1), which sets their crit chance, armor (STR / 4) and status resistance; give them a neutral baseline or author classes for them.
- [ ] **Reinforcements**: one wave per encounter; multiple waves, conditional triggers (e.g. "when the scout escapes") and explicit spawn tiles.

### 13. World Engine Follow-ups

*Status: lore data model, inheritance, overrides, connections, World Forge and playable world battles implemented; the systems that consume quests, dialogue and maps are still roadmap items.*

- [x] Modular worlds with inheritance, namespaced ids, explicit overrides and cross-world connections.
- [x] Regions, locations, factions, characters, named items, disciplines (mapped onto base passives), quests, lore and encounters as data, all referencing base catalog assets with explicit tuning deltas only.
- [x] World Forge editor and playable world encounters (Scene Runner, `--encounter=world:id`).
- [ ] Quest runtime (accept, track steps, rewards) and dialogue, once the NPC / dialogue engine exists (`quests` and `characters` data are ready).
- [ ] World map travel over regions / locations (location exits are the graph; `encounters` mark where fights trigger) and persistent world state per save.
- [ ] Quest and lore rewards that grant passives and event-learned skills at runtime (the data and `Combatant.learn_skill` exist; no save/progress layer yet).
- [ ] Faction reputation and relations affecting encounters, prices and quests (relations are authored already).
- [ ] Conflict reporting when two parents define the same local id, and a graphical inheritance view in the World Forge.
- [ ] World packs: export / import a world folder as a shareable module with its base-catalog dependencies listed.
