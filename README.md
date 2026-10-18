# Tactical RPG (Godot 4)

A modular, simultaneous-execution tactical RPG engine built in **Godot 4** (GDScript). Every unit plans a **6-block round** (AP-costed skills, one move, AI-planned enemies) that resolves simultaneously on a **12x9 spatial grid**, featuring rock-paper-scissors **stance dynamics**, tactical targeting, perimeter escape/withdrawal, and deterministic state scrubbing.

What changed lately: [PATCH_NOTES.md](PATCH_NOTES.md) lists every change on `develop` (updated at each release, 11:52 and 17:52 UTC), newest first, with plain-language lines for what you can see in the game.

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
- **Spatial Positioning & Withdrawal**: 12x9 grid maneuvering with party-left / enemies-right deployment, reach bands, and safe boundary withdrawal.
- **Content as data**: skills, items, classes, units and encounters live in a JSON catalog edited through `ContentCatalog.upsert` seed scripts (the Content Forge is a read-only viewer); the engine, the scenes and the tests all load from it.

---

## Codebase Architecture & Directory Breakdown

```text
tactical-rpg/
├── assets/                       # Static assets (icon.svg plus assets/art and assets/audio folders (file overrides through data/assets/manifest.json); placeholder art is generated in code, see scripts/art/)
├── data/
│   └── catalog/                  # Content catalog (JSON, edited by seed scripts; browsed in the Content Forge): the single source of game content
│       ├── skills.json               # Every skill tier (attacks, defends, focus, rest + engine skills marked "system")
│       ├── items.json                # Weapons, shields, armor, accessories, consumables ("starter_stash" flag)
│       ├── classes.json              # The six base classes: stats, growth, gear proficiency dict, skill path, loadout, art
│       ├── units.json                # Party members & enemies: vitals or class/level-derived, loadout, skills, AI profile
│       └── encounters.json           # Battles: zone, exits, round rules, party & enemy rosters
│   └── worlds/                   # Lore worlds (JSON, edited by seed scripts; browsed in the World Forge): modular, inheritable; reference catalog assets
│       ├── ashlands/                 # Root world: Ember Wardens vs the Ashen Horde (regions, locations, factions, characters, named item, disciplines, quests, lore, encounters, the CAMP1 campaign)
│       ├── tidewrack/                # extends ashlands: the same war re-skinned (overrides) plus its own coast, characters and encounter; a tuned boss
│       └── iron_reaches/             # separate world that connects to ashlands (a road into Warden Keep) without inheriting
│           └── world.json + regions / locations / factions / characters / items / disciplines / quests / lore / encounters .json per world
├── scenes/
│   ├── combat_arena.tscn         # Main scene: combat UI foundation (CombatSession + CombatDockHost)
│   ├── combat_arena.gd           # Builds the session for an encounter and mounts the dock host
│   ├── scene_runner.tscn         # Main scene: Scene Runner menu (launch scenes, hot-reload scripts, run suites)
│   ├── main_menu.tscn            # Player front door (UI1-L): continue, new game, load / save slots, settings, hub
│   ├── test_arena.tscn           # Test harness scene: runs SuiteRegistry.run_all (headless exit code); windowed opens the runner
│   ├── test_arena.gd             # Harness: headless exit code for check_project.ps1 and the runner's suite buttons
│   ├── gear_dojo.tscn            # Dedicated Gear, Skill & Damage Formula Benchmark scene
│   ├── gear_dojo.gd              # Thin bootstrap: builds DojoSession, mounts the window workspace or runs assertions
│   ├── content_forge.tscn        # Content Forge: read-only viewer of skills, items, classes, units & encounters; playtest in the arena
│   ├── content_forge.gd          # Mounts ContentForge; swaps in combat_arena for playtests ("Back to Forge")
│   ├── world_forge.tscn          # World Forge: read-only viewer of lore worlds; play their battles
│   ├── world_forge.gd            # Mounts WorldForge; swaps in combat_arena for "Play this encounter"
│   └── dojo/
│       ├── dojo_assertions.gd        # 42 headless dojo assertions: Dojo-1..18 here, then the two files below (CLN-12 split)
│       ├── dojo_session_assertions.gd # session actions, gear integrity, logging, benchmark, catalog skills, dummy attacks
│       └── dojo_ui_assertions.gd     # workspace panels, slot rules, stash filters, party viewer, carry, overlays
├── scripts/
│   ├── content/                  # Content catalog engine (headless)
│   │   ├── content_schema.gd         # Field lists per kind (tables in skill_fields.gd, item_fields.gd, class_unit_fields.gd, terrain_fields.gd; per-kind checks in validators/, TerrainValidator for terrain): types, defaults, ranges, enums, refs, conditional fields; normalising
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
│   │       ├── combat_grid.gd            # 12x9 grid, Manhattan distance, open/closed arenas, escape tiles, terrain walkability, standard deployment
│   │       ├── terrain.gd                # Terrain types (17: ids, glyphs, walkability in code); move cost, cover and hazard from catalog kind `terrain` (code defaults until S12T-A)
│   │       ├── combat_zones.gd           # 8 preset 12x9 combat zones (forest, river, lava cavern, ...)
│   │       ├── grid_path.gd              # Cheapest-cost routes (ties: fewer steps, centre, DIRS), move-block chunking with minimum progress; shared by planner, AI and commands
│   │       ├── cover_rules.gd            # Ranged cover: target tile or the blocking neighbour toward the attacker (S12T-L)
│   │       ├── hazard_rules.gd           # Hazard tiles roll their status on enter and/or at round end (S12T-L)
│   │       ├── combat_resolver.gd        # Round resolution: AP/stance at start, effects at end, initiative, casualty cleanup
│   │       └── block_snapshot.gd         # Historical state capture for deterministic scrubbing/replay
│   ├── audio/                    # Sound by key (S14-L3): SoundEvents keys, SoundBank lookup, AudioDirector voices, PlaceholderSfx blips; MusicDirector (MUSIC1)
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
│   ├── playtest/                 # Playtest recordings (debug): record schema, plan codec, outcome, fingerprint, recorder, replayer, sweep, store, trail report, logger hook
│   ├── tests/                    # Headless test suites & validation harnesses
│   │   ├── core_rules_test_suite.gd  # Core combat rules via the planner: guard & matchup, stances, Focus, casualties, withdrawal, feed
│   │   ├── class_rules_test_suite.gd # Six classes: budget/growth, spend variance, tiers, proficiency, passives, skill paths
│   │   ├── content_test_suite.gd     # Catalog parity, former Forge save checks over shipped data, round trips, validation, authoring, full-engine battle, Forge viewer
│   │   ├── playtest_test_suite.gd    # Playtest recordings: codec, round trip, redo, divergence, staleness, take-over, variant, sweep, trail report
│   │   ├── runner_test_suite.gd      # Scene Runner: scene list, launch/restart/menu, unsaved-work guard, hot reload
│   │   └── world_test_suite.gd       # Worlds: shipped data, inheritance, same-base/different-lore parity, former World Forge apply checks, tuning, renames, connections, viewer, runner
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
│       │   ├── combat_grid_panel.gd      # 12x9 grid tokens, hover cards; hands action FX to the presenter
│       │   ├── combat_presenter.gd       # All combat animation: owned tweens and temp nodes, instant mode, kill_all (S14-L2)
│       │   ├── fx_presets.gd             # Tween recipes and timings, log entry -> presets mapping
│       │   ├── fx_settings.gd            # Animation speed and instant mode (always on headless)
│       │   ├── combat_hud_metrics.gd     # Layout A sizes (COMBAT-UI3): strip, columns, tile, cards per text size (pure math)
│       │   ├── combat_hero_panel.gd      # Selected-hero panel under the party cards (COMBAT-UI3)
│       │   ├── combat_enemy_card_panel.gd # Merged enemy card: state, stats, Next, your hero's deal/take, Can use (COMBAT-UI3)
│       │   ├── plan_block_text.gd        # Planning-strip block kind, colour, label and hover text (COMBAT-UI3)
│       │   ├── strip_question.gd         # Small non-blocking question popup (missing-actions warning)
│       │   ├── combat_action_log_panel.gd # One-line log ticker dock
│       │   ├── combat_playback_panel.gd  # Step / play / reset / round advance
│       │   └── combat_timeline_panel.gd  # Block ruler & focused unit track (planning bar)
│       ├── runner/                   # Scene Runner (main menu)
│       │   ├── scene_runner.gd           # Scene list, launch / restart / back to menu, unsaved-work guard, suite runs, F5/F6/F10
│       │   └── script_reloader.gd        # Hot reload: recompiles scripts edited on disk (GDScript.reload)
│       ├── forge/                    # Content Forge (read-only asset viewer, TOOLS-RO)
│       │   ├── forge_source.gd           # What a ForgeForm shows (fields + reference choices); ContentForgeSource is the catalog's
│       │   ├── content_forge.gd          # Kind tabs, entry list (search/side/item filters), read-only form, preview, problems, reload, validate, playtest
│       │   ├── forge_form.gd             # Schema-driven read-only view: one disabled widget per field, conditional fields, no action buttons
│       │   └── forge_preview.gd          # What an entry becomes in the engine (stats, reach, damage, deployment), icons/portraits
│       ├── world/                    # World Forge (read-only world viewer, TOOLS-RO)
│       │   ├── world_forge.gd            # World picker, element tabs, merged list (inherited marked), read-only form, lore preview, problems, play
│       │   ├── world_forge_source.gd     # Plugs WorldSchema / WorldCatalog into the shared ForgeForm (refs resolve from the world shown)
│       │   └── world_preview.gd          # What an entry means: lore, what it plays as, where it differs from its base, who uses it
│       ├── combat_grid_view.gd       # 12x9 grid rendering (54px tiles), tokens, tile input, FX layer
│       ├── plan_overlay.gd           # Planning ghosts on the grid: route, destination ghost, reach, target ring, cost hint
│       ├── plan_action_popup.gd      # Small per-target popup of valid actions (AP/blocks, hover preview, 1-9/Enter)
│       ├── roster_panel_ui.gd        # Party status (2x2) and hostile status (5x2) docks
│       ├── playback_controls_ui.gd   # Step prev/next, Auto-Play, Reset, Round navigation
│       ├── timeline_dock_ui.gd       # COMBAT-UI3 planning strip: "<Hero> plan", continuous kind-coloured blocks with hover cards, Party/Hero tactics, Autoplay, Clear, Confirm round (warns about empty blocks unless Settings > Gameplay > Warn about missing actions is off). Before: top-row dock (block ruler sized to fit, unit track, control row: AP status / autoplay banner, Party and Hero Tactic dropdowns, Autoplay, Clear, Confirm; drag/remove steps; 14 px text floor)
│       ├── action_log_dock_ui.gd     # Top action dock displaying active block resolution
│       ├── combat_log_modal.gd       # Fullscreen expanded event stream modal with category filters
│       ├── timeline_modal.gd         # Fullscreen synchronized multi-unit timeline modal
│       └── character_equipment_modal.gd # Party & Stash screen: 3 columns (party viewer, gear loadout, party stash) sharing one DojoSession
├── playtests/                    # Shared playtest files: inbox/ (user recordings), repro/ (agent repros), scenarios/ (one seed each); see README.md
├── check_project.ps1             # 4-step automated validation script (cache, syntax, arena suite, dojo suite)
├── check_project.bat             # Batch launcher for check_project.ps1
├── run.bat                       # Pulls the latest main, then opens the Scene Runner (every scene, hot reload, test suites)
├── run_dev.bat                   # Same on the latest develop (finished work not yet released), from the sibling folder <checkout>-dev
└── run_opt.bat                   # Same for a design-review option build: run_opt.bat <a|b|c|branch>, from the sibling folder <checkout>-opt
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

### 3. 12x9 Spatial Grid & Spatial Rules

- **Arena Geometry**: Fixed 12x9 grid (`x 0..11`, `y 0..8`); no tile is marked as the centre. Tie-breaks that prefer central tiles measure to the true middle (between columns 5 and 6 on row 4). Was 9x9 with a centre ring until GRID12 (2026-10-17).
- **Standard Deployment**: the party lines up on the left, enemies on the right. Melee units take the front column (party x 5, enemies x 7), ranged units the back column (x 4 / x 8), rows filled from row 4 outward; a full column spills backwards. Front ranks start 2 tiles apart, as on the old centre ring.
- **Reach Bands**: attacks check the target's distance against the skill's reach band (melee 1, ranged 2-3, off-hand jab 1, area skills their own); a targeted attack adjusts one tile when it starts, otherwise an out-of-reach attack fails cleanly.
- **Perimeter Escape & Partial Withdrawal**: Stepping onto designated exit tiles (e.g. `(11, 4)`, the east edge) triggers safe partial withdrawal, removing the unit from active combat without penalty.
- **Reinforcement Ingress**: an encounter's reinforcement wave enters before a given block of a given round (e.g. the Goblin Ambush at Round 1 Block 4), deploying on the free tiles nearest the arena middle on the enemy (right) half; the AI plans it from the next round.

### 4. Deterministic Scrubbing & Replay System

- **State Snapshots (`BlockSnapshot`)**: Full state (HP, AP, stances, targets, buffs, guard, active flags, coordinates) captured per block and per log entry.
- **Two-Way Stepping by log point**: After a round resolves, `Step Prev` / `Step Next` walk the round's log entries one at a time (each entry keeps the board state right after it), so a 2-block move replays from where it started. Selecting a round jumps to its start; selecting a block jumps to the state just before that block. Playback is a review/testing aid only and never changes the battle.
- **Log export**: in debug builds every resolved round is written to `user://combat_last.log` (board cells plus the plain-text feed) for sharing and bug reports.
- **Multi-Round Historical Traversal**: Review previous completed rounds (Round 1, Round 2) without losing active simulation state.

### 5. Modular Combat UI (`combat_arena.tscn`)

- **Headless `CombatSession`**: Owns one battle: the encounter's combatants, grid and resolver, round flow, block-by-block resolution, per-block snapshots, multi-round traversal, selection/hover and the action log. Panels only read it and call its actions; every change ends in `changed`, visual one-shots use `fx_requested` / `hover_changed`. The whole battle flow is headless-testable.
- **Pluggable `CombatEncounter`**: Defines *what* a battle is (combatants, deployment, per-round plans, mid-round triggers, narration, art). Every battle is a `CatalogEncounter` built from the content catalog (Goblin Skirmish is the arena default; Goblin Ambush adds a mid-round wave); region-generated encounters will plug in the same way. Its combatants are the single source of truth for simulation and rendering.
- **Panels (`CombatPanel`)**: Party roster, enemy roster, resolution log, combat grid (which also owns the unit hover card), tactical forecast, playback controls and timeline are independent panels bound to the session, each wrapping an existing view component. The hover card, roster and grid all render the replayed snapshot when reviewing a past block.
- **Static Dock Host (`CombatDockHost`)**: Fixed regions (`top`, `left_top`/`left`, `center`, `right_top`/`right`, `bottom_left`/`bottom_center`) filled from data-driven layout presets (`Standard`, `Mirrored`); sizes come from `CombatHudMetrics` (COMBAT-UI2-L1). The game UI stays static; **layout edit mode** (`⚙` / `F9`) overlays an `⇱ Undock` tag on each panel without moving anything, floats panels into `UiWindow`s (their dock keeps a same-size placeholder) and docks them back when closed. The host also owns the full-screen modals and shortcuts (`Space` play, `Q`/`E` blocks, `[`/`]` log, `L` log modal, `F`/`Tab` timeline, `Esc` close; no gear shortcut or button in battle: equipment is changed outside combat).
- **Custom SVG Visual Assets**: Custom vector graphics for characters, unit tokens, and grid tiles.
- **Allied & Enemy Status Roster Docks**: Real-time HP bars, plan status (`○ no plan` / `✔ n` / `⚠ n`), AP, stance and position.
- **Planning Phase (on the combat grid)**: Each round starts in PLANNING and you plan right on the main screen. Select a hero (click the token or roster card; cards show `○ no plan` / `✔ n` / `⚠ n` and AP), then click a target: an **enemy** opens a small popup of attack skills, **your own token** lists Guard/Brace/Focus/Rest, an **empty tile** offers *Move here* (the exit offers *Withdraw*). Each row shows `AP · blocks`; hovering a row (or just hovering a target) previews the route, destination ghost, weapon reach and cost on the grid, and hovering inspects the unit in the side panels. Pick with the mouse, `1`-`9`, or `Enter` / clicking the same target again for the first option; **right-click the grid or press Backspace to undo the hero's last step**; the step lands on the lowest free block and the hero's ghost trail and the timeline update. The timeline bar shows the hero's AP, next block and warnings, with **🤖 Party / Hero Tactic dropdowns** (the six tactics; the Hero one adds Custom, follows the hero whose track is shown and is disabled for guests; a pick redrafts at once, and asks first when the row holds edits; a manual edit shows `Custom (edited)`), **Autoplay** (`A`; `Space` pauses it, a banner says why it stopped), **Clear** and **Confirm Round** (`Enter`); right-click a timeline command to remove its step and drag one onto another to reorder. **Engage** combines target + move: the moves to get in range are inserted automatically and walk the planned path (see the one-tile adjustment below). Hard-invalid sequences (no path, bad target, too many blocks) block Confirm; running out of AP only warns. Once confirmed the timeline is a read-only visualisation.
- **Round rules**: rounds are **6 blocks** (roughly three actions). Each unit has an **AP pool of 12** that carries across the battle and regains **+4 each round**; **Rest** (1 free block) recovers 2 AP and returns to Neutral. Leveled skills cost AP equal to their level (a level 2 Attack or Defend always costs 2 AP unless an effect shifts it via `Combatant.ap_cost_modifier`). **Moving must come first** and happens once: a manual move, a move-to-tile or the approach of an Engage, up to **2 blocks** at the unit's speed-based tiles per block, 1 AP per tile. A 2-block move is split per block: its first stretch lands at the end of block 1, the rest at the end of block 2. A move blocked at resolution (e.g. an enemy stepped into the path) stops on the last free tile, refunds the tiles it didn't walk and logs **CUT SHORT** with what blocked it; a second half re-paths toward the destination if the first half was cut short. Allies plan around each other's planned moves: tokens stay on their real tiles and each planned move shows as a **clickable shadow** at its destination (click your own shadow for self skills; a shadow on a tile that is still occupied shrinks into the corner). A unit may take a tile an ally leaves in block 1 if that ally acts first: click that ally's token to get *Move here* or *Plan for `<ally>`*. Attacks need the target in the skill's reach: melee 1, **ranged single-target attacks 2-3** (or the unit's longer range; an **Off-hand Jab** covers distance 1, with the off-hand weapon or bare hands); area attacks have their own reach (Volley 2-3, Firebomb 2, Shockwave adjacent). Attacking a different target than the current lock inserts a free **Switch Target** block. A target already expected to fall can still be attacked (insurance, flagged in amber). `AiProfile.skill_cap` (and an optional known-skills list per unit) stands in for skill progression. Rounds, AP pool and regen are per encounter (`total_blocks`, `unit_ap`, `ap_regen`).
- **AP debt instead of mid-round stops**: overspending is allowed; the unit finishes its sequence in AP debt and pays next round: the debt comes out of its +4 recovery, **3+ debt Staggers** it (blocks 1 and 4 lost) and **5+ Stuns** it (blocks 1-2 lost). The planner shows the projected debt and penalty. Engage approaches **walk the planned path** (no live re-chasing); when a **targeted attack** starts it adjusts by one tile if the target shifted out of its reach band (1 AP, may go into debt, shown as its own *Chase* log cell): melee steps in, a ranged unit whose target stepped point-blank **backs off** to shoot. Only damage-focused attacks adjust: after a hand-picked tile move (Move here / Withdraw) the unit holds its tile, and area attacks never move. Each approach destination is fixed when first planned, and units claim tiles in planning order, so allies (and the AI pack, which drafts into one shared plan) never plan onto the same tile. Confirming the round starts resolving it automatically; Play afterwards replays it.
- **Statuses (`StatusEffects`)**: Stagger, Stun and Root (no moving) last one round and always apply to the **next** round, so they reshape the next plan instead of breaking the current one; lost blocks show as locked `✖` blocks on the timeline. Skills roll to inflict them with a **seeded roll** (replays stay identical): chance = potency - the target's Debuff Resistance (INT) - any passive resistance (`Combatant.status_resist`; 100 = immune). New skills: **Shield Bash** (Defensive-stance strike, may Stagger), **Hamstring** (may Root).
- **Area attacks (tile-targeted)**: **Volley** (plus shape), **Firebomb** (3x3) at a tile in weapon reach, **Shockwave** (3-tile line from an adjacent tile). They hit everyone in the area, **allies included** (friendly fire is previewed in orange and warned). Click an empty tile or an enemy to aim; the preview shows the affected tiles.
- **Damage visibility**: attack rows in the popup and attack commands on the timeline show the expected hit (`~25 dmg`, `DamageCalculator.expected`: see 2b). Log cells show the damage actually dealt (`[HIT -28]`); in debug builds (`ActionLogEntry.show_formulas`) hovering the tag explains the formula.
- **Timing**: an action's effect lands at the **end of its last block**; its stance and AP apply when it **starts**; Defend protects from its first block. Within a block, higher priority (Switch Target, Defend) resolves first, then speed.
- **Layout (COMBAT-UI2-L1, "compact columns")**: the timeline row is on top; a 220 px party column (hero cards: token, name, level, stance, plan state, HP and AP bars, effects, speed and reach; muted outlines for free slots; the column takes the whole height) on the left; the grid in the middle with the biggest tile that fits (`floor(min(h/9, w/12))`, at least 48 px; 62-63 px at 1280x720); a 220 px hostile column (token, name, stance, HP bar, behaviour line) above the forecast on the right; playback buttons, the phase badge and the one-line log ticker (click or Expand opens the log) in the bottom strip. All HUD text is at least 14 px at Normal and follows the text size setting (`UiTheme.text_scale()`); panel heights are fixed and cards drop lines (speed/reach, effects, behaviour) as text grows. A hero card click selects the hero. The settings cog is a 32 px button.
- **Unit hover card** (COMBAT-UI2-L2; replaces the Unit Inspector panel, read-only): rest the pointer on a token or a party / hostile card for 150 ms (or focus a card with the keyboard / controller) and a card opens inside the grid panel beside the unit, never covering its tile and kept inside the viewport and the board. It shows portrait, HP and **AP** gauges (AP debt shown), stance, position, speed, basic-attack reach band, target, effects and statuses, then for a hero the gear as read-only text and the plan steps, for a hostile the AI behaviour (role, strike power, guard and flee thresholds) and the numbers against the planning hero (distance and reach, stance factor, both strikes). A click on the card, on the unit's token or on its roster card, or the Pin button, pins it (it stays when the pointer leaves); Esc, a click elsewhere or a right click closes a pinned card. While a planning popup is open an unpinned card steps aside, and a click that opens the popup does not pin. Text follows the text size setting (14 px floor at Normal); at the largest sizes the card drops the gauge bars before it would outgrow the board. No control in battle opens the equipment modal.
- **Tactical Forecast Dock** (narrow, scrolls when the text is large): distance, reach-band check (in reach / too close / out of reach), stance matchup, the expected Strike hit and crit chance (section 2b), and the encounter's rules (blocks per round, AP pool and regen, exits).
- Both side docks have a content-independent width (at least 270 px, `PANEL_WIDTH`; extra window width is shared evenly) and their text wraps or ellipsizes, so selecting a hero or an enemy never moves the grid or the other dock; the grid's move/attack preview wraps and stays inside the grid (COMBAT-LAYOUT, checked in `combat_ui_framework_test_suite.gd`). New labels in the docks must wrap or ellipsize.
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
  - **Main-Hand Weapons**: Swords, Greatswords, Hammers, Bows, Staves. Every weapon has exactly one hand type (strict hand typing); plain 1H main-hand weapons go in the off-hand only in the Gear Dojo (`Equipment.allow_plain_dual_wield`).
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
  - Opened from the dojo `Stash` button (there is no gear button in battle); `Esc` closes it.
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
- **Encounters in the real engine** (`CatalogEncounter`): any encounter entry runs through the full combat stack (planning, AI, resolver, replay, UI). Repeated enemies are numbered (`Goblin Archer 2`); units the standard deployment can't fit go to the free tile nearest the arena middle on their half. One **reinforcement wave** per encounter (`wave_units`, `wave_round`, `wave_block`, `wave_label`) enters mid-round. `combat_arena` loads one from the command line: `godot --path . res://scenes/combat_arena.tscn -- --encounter=veterans_drill`.
- **Content Forge** (Scene Runner > Content Forge): kind tabs; searchable entry list showing names only (skills filter by kind, grade, with Move, Rest and the basic skills under **basic**, stance and learn-only; sort by name, kind or AP); a **display-only** form generated from the schema (label: value rows, no input widgets); for skills, an **in-game preview** with a ◀ Rank ▶ selector that shows the rank in the real combat action menu, row tooltip, resolution log and skills screen (FORGE-VIEW); an **engine preview** of what the entry becomes (skill cost / reach / damage, item stats and which classes can use it, class stats at levels 1/5/10, unit vitals / power / skills / gear, encounter deployment) with its icon or portrait and "used by" references. **Reload** (picks up seed-script edits) and **Validate All** (result on the status line, details in its tooltip). `tools/forge_shot.gd` saves a screenshot of any tab, entry and rank. Since TOOLS-RO the Forge never writes: content changes go through `ContentCatalog.upsert` seed scripts (`docs/workflow/content-edits.md`), and every check the old editor ran on Apply / Save runs in `content_test_suite.gd` over all shipped data. **▶ Playtest** runs the selected encounter (or a unit in a sandbox fight on its side) in the real combat arena, with *Back to Forge* to return.
- **Reuse in tests**: tests build units and encounters straight from the catalog (`ContentFactory.unit(ContentCatalog.get_entry("units", id))`, `CatalogEncounter.new(id)`). Tests that write call `ContentCatalog.fork_to("user://...")`, so the shipped catalog is never modified.

### 10. Scene Runner (`scene_runner.tscn`, main scene)

- **Playtests** (debug builds, PLAYTEST1): every battle is recorded (toggle **Record playtests** in the PLAYTESTS panel, pref `record_playtests`, default on). **F8** (or the **✎ Note** button in the Scene Runner pill while recording) in any scene saves a feedback note (text, tags mechanic / ui / ux / bug) with a screenshot. The panel lists recordings (**Open** = playback with a banner and note jumps, **Notes**, **Take over** from a round, **Copy to repo** into `playtests/inbox/<id>/`) and scenarios (**Launch** with the scenario's seed). A recording that cannot be played back opens a live battle with the reason in the banner; that battle is not recorded. Nothing is hooked while no recording is open. Agents render noted moments with `xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . -s tools/replay.gd -- <file> --render=notes|rounds --out=<dir>`.
- **One launcher** (`run.bat`, or Play in the editor): a menu listing every runnable scene: Combat Arena (with an encounter picker over the content catalog, remembered between runs), Content Forge, Gear & Skill Dojo, and any other `.tscn` dropped into `scenes/` (shown under its file name). Scenes run inside the runner, so returning to the menu is instant.
- **In-scene controls**: a small **☰ Menu / ↻ / ⟳** pill (bottom-right; hidden when the real game, `main_menu.tscn`, is launched, hotkeys still work) and keys **F10** (menu), **F6** (restart the scene), **F5** (reload edited scripts, then restart). Leaving a scene with unsaved in-memory catalog or world changes asks for a second press and then discards them. **Window controls** (user 2026-10-18): **Fullscreen** in the runner menu and pill, or **F11** in any scene, toggles fullscreen and windowed; **Minimize** and **Exit** (desktop only) sit next to it (Exit asks for a second press when the scene has unsaved changes, like Menu). The toggle does not change the saved Display setting.
- **Hot reload** (`ScriptReloader`): remembers each script's modification time and recompiles the edited ones in place (`GDScript.reload`), clearing static caches built by the old code and re-reading the catalog JSON. New files or new `class_name`s can't join a running game's class registry: the menu lists them and **⏻ Restart runner** starts a fresh process.
- **Test suites**: **▶ Arena suite / ▶ Dojo suite** run `test_arena.tscn` / `gear_dojo.tscn` headless in a separate process (the menu stays usable) and show the PASS / FAIL lines with a summary.

### 11. World Engine & World Forge (`world_forge.tscn`)

The Content Forge catalog is the **lore-free base library** (skills, items, classes, units, encounters). Worlds are a second layer: they give those assets names, backgrounds, factions and places, and never carry gameplay of their own.

- **Worlds are folders** (`data/worlds/<id>/`): `world.json` (name, tagline, description, `extends`, `connects`) plus one JSON file per element kind. Kinds: **regions**, **locations** (exits, encounters), **factions** (relations), **characters** (party, NPCs, enemies), **named items**, **disciplines**, **quests** (steps, requirements, rewards), **lore entries** and **encounters**. Entries have local snake_case ids; the **qualified id** is `world:id`. Quests keep tracking across `connects` (QUESTX): a quest accepted in one world stays active, advances and completes while the party is in a connected world (connections followed both ways, one hop); step refs resolve from the quest's own world, quest ids stay world-qualified, and which quests are offered stays per world.
- **Base class + lore** (`WorldFactory`): a character names a **base unit** (`base_unit`), a named item a **base item** (`base_item`), a discipline a list of **base passives** (`passives.json`: gear training, stat edges, resistances, AP regen, skills). The base asset decides how it plays; the entry adds name, title, background, personality, faction, home, art. Two characters (in one world or two) can point at the same base unit with different lore and behave identically: same vitals, class, skills, AI, damage.
- **Explicit divergence only**: a character may add `tuning` deltas (`hp_pct`, `power`, `armor`, `speed`, `ap`), named items (worn on top of the base loadout, one per slot) and inherited **passives** (the character's own, its faction's `grants`, its disciplines' and its named items' `grants`). Item tuning (`power`, `mitigation`, `burden`) and extra stat bonuses likewise. The World Forge shows **"Differs from base"** and the base unit's numbers side by side, so you can always see where a world deviates (Captain Dredge: hp +20%, power +2, armor +1).
- **Inheritance**: `extends` lists parent worlds; `lineage()` is the load order (ancestors first, the world last). A world sees every ancestor entry. Unqualified references resolve in the entry's own world, then up the lineage (nearest wins); `world:id` is explicit. Everything is namespaced, so two worlds never collide.
- **Overrides**: an entry with `override: "ashlands:mara_voss"` replaces that ancestor entry **for its own world and every world below it** (the parent is untouched), and every reference to the old id (even the old id used in an inherited encounter) now resolves to the override. This is how one world re-skins another: `tidewrack` overrides Mara, Tev and the five goblins, so **playing the inherited `tidewrack:courtyard_skirmish` is the same battle with different people**.
- **Connecting worlds**: `connects` links worlds without inheriting: a location may exit into `ashlands:warden_keep` from `iron_reaches`; qualified references to a world that is neither an ancestor nor connected are validation errors. Connections are inherited by child worlds. A crossing remembers the world left in `GameState.explore["home_world"]` (optional key, no save version bump); entering a location of that world's lineage again restores it, so a Tidewrack party back from Iron Reaches is in Tidewrack, not plain Ashlands (EXPWORLD).
- **Safe editing** (`WorldCatalog`): validation reports unknown base assets, role mismatches (a party roster only takes party characters), duplicate references, slot clashes between named items, unknown tuning keys, override targets, circular regions / inheritance. **Blocking errors** (ids, enums, json shape, override validity) stop an entry from being stored; **unresolved references** don't (worlds are full of mutual references: two locations exiting into each other), they are listed as problems, shown with a warning mark, and checked by *Validate All* and before playing. Renaming an id rewrites every reference to it (plain refs, json refs such as exits / quest steps / faction relations, override pointers) across the world, its descendants and connected worlds; in-use entries and extended worlds can't be deleted.
- **Playing worlds**: `WorldEncounter` runs a world's encounter in the real combat engine with the world's characters (lore names in the roster, log and inspector, their named items and tuning); `EncounterSource` lists catalog and world battles together (`tidewrack:strand_clash`), so the **Scene Runner's arena picker** and `combat_arena -- --encounter=ashlands:pass_ambush` run them. Quests, dialogue and map travel are authored as data now and become playable with those systems.
- **World Forge** (Scene Runner > World Forge): a **read-only** viewer (TOOLS-RO): world picker, element tabs plus an Overview (manifest, inheritance, counts; the left list shows every world with its parents, click to switch); a merged entry list where inherited entries are marked with their origin world; the shared schema-driven form (read-only) (reference pickers show `Name [id]` plus the origin world); a lore preview with *Plays as*, *Differs from base* and *Used by*; its problems; **Reload / Validate All**; **Play this encounter** in the arena with a *Back to World Forge* button. Worlds change through WorldCatalog seed scripts; the former Apply checks run in `world_test_suite.gd`.
- **Shipped worlds**: `ashlands` (root: Ember Wardens vs the Ashen Horde, the Courtyard skirmish and a wave ambush), `tidewrack` (extends it: Harbour Guard vs Reef Raiders, overrides, own coast and battle, one tuned boss) and `iron_reaches` (connected: a guild road into Warden Keep).

---

### Web build (browser)

The `Web` export preset builds a single-threaded HTML5 version for cross-platform testing; `.github/workflows/web.yml` builds and deploys `main` once a day and on request (artifact `web-build`, itch.io, GitHub Pages). The browser build opens on the Main Menu (`run/main_scene.web`); the desktop keeps the Scene Runner. Setup and what does not work in the browser (developer panel, emoji icons): [docs/workflow/web.md](docs/workflow/web.md).

## Headless Testing & Automated Quality Assurance

All features are tested headlessly without requiring a graphical window or GPU.

The suites registered in [suite_registry.gd](scripts/tests/suite_registry.gd) (32 suites, about 300 assertions with the dojo; `check_project.ps1 -Fast -Quiet` prints the current count) run headlessly (each suite prints one line per assertion; the old hand-scripted 10-block battles were retired in favour of the Core Combat Rules suite):

1. **Combat UI Framework & Behaviour Test** (8 assertions plus the COMBAT-UI2-L1 layout checks: tile formula, fixed card and panel heights at three window sizes and four text sizes, card counts, hero card click, no gear path, log ticker): all 8 panels mounted in their dock regions, tokens & roster bound to the simulated combatants, selection → inspector, tactical forecast, log & timeline modal lifecycle, timeline block inspection, undock/dock + layout presets + action FX dispatch.
2. **Core Combat Rules Test** (7 assertions, [core_rules_test_suite.gd](scripts/tests/core_rules_test_suite.gd)): planned exactly like the game and resolved by the real resolver: the one damage formula (weapon + unit power, unarmed STR + DEX, unsuited weapons, authored power, armor curve and gear rating, x1.5 crits with their chance, seeded crits repeating, the planner's estimate = the formula), Brace mitigation with the Defensive-vs-Offensive matchup (exact damage, reading the seeded crit roll), stances persisting after resolving and into the next round (Shield Bash stays Defensive), Focus lasting its rounds then expiring, a defeated unit losing the rest of its round, Withdraw leaving the grid and ending the battle, and the full Goblin Ambush feed (moves, attacks with target HP, buffs/guards, the R1-B04 wave, withdrawal, defeats).
3. **Character Specs & Inventory Test** (10 assertions): Archetype 6-stat baselines (equal budget), derived attributes, level-up guaranteed growths, free point allocation, event point rewards, proficiency training and contextual passive bonuses, equipment stat requirement check, threshold tiers boost, deficit wield penalties, and battle pouch consumable usage in combat.
4. **Gear Equip UI & Test Dummy Suite** (10 assertions): Equipment modal mounting, hero switching, stash equipping, Claymore Threshold Tier 1 (+4 Power) boost, Titan Warhammer deficit penalty, armor burden speed penalty, unequipped item return to stash, training dummy 500 HP baseline, base strike and 50% defense mitigation formula, battle pouch consumable damage & removal.
5. **Combat Session Flow Test** (8 assertions): the catalog Goblin Ambush through `CombatSession` headlessly: only deployed enemies in view before the wave, the wave event at `R1-B04`, Round 1 resolving into replay with round advance offered, replay by log points, state carried into Round 2 with past rounds reviewable, later rounds (AI presets) playing on to a conclusion, timer-free auto-play (including Play pressed while planning), and round reset.
6. **Art, Terrain & Zones Test** (9 assertions): team-colored class tokens and portraits, encounter art follows sides, terrain glyph round-trip and walkability, all 8 zones valid, grid blocks blocked tiles, the Goblin Ambush (wave included) deploying and concluding on every zone, tile and item icon rasterization.
7. **Planning, Stances, AP & Enemy AI Test** (16 assertions): reach bands and the off-hand jab, move-first, Root, seeded status rolls and immunity, area friendly fire, allies moving into vacated tiles, AP debt penalties (Stagger/Stun blocks), on-grid input options (per target / tile), round rules (6 blocks, AP pool + regen, one move per round, centre tie-break for equal routes, target switching, insurance attacks, attack follow, damage in the log, auto-play on confirm), skill AP = level (Move per tile, Rest free) with effect modifiers and speed-based tiles per block, the rock-paper-scissors stance matrix and automatic +1 AP switch costs, sequences compiling to the lowest free blocks with Engage inserting 1-block moves, mid-sequence stance switching, projected "runs dry" instead of refusal, flagged invalid edits blocking Confirm, Confirm scheduling party and AI plans with AP spent at runtime, units stopping when they can't pay, Rest recovering AP, AI presets, role patterns (walls, sniper retreat, support Focus, flee, Engage chasing a moved target), deterministic AI-vs-AI skirmishes on all 8 zones, the one-tile chase only for targeted attacks (not after a tile move; ranged units back off), and round reset refilling AP.
8. **Content Catalog & Forge Test** (10 assertions, [content_test_suite.gd](scripts/tests/content_test_suite.gd)): the shipped catalog loads and validates; the engine's skills, items, stash, classes and Goblin Skirmish come from it; skills, items and classes round-trip through engine objects unchanged; validation catches bad ids, kinds, reach, slots, references and rosters and refuses unsafe deletes; on a sandbox copy: authoring a skill, item, class, units and an encounter, saving, reloading, renaming an id (references follow) and duplicating as a template; authored units bring class-derived vitals, gear, numbered duplicates, whitelisted skills and resistances into planning; the authored encounter (and the shipped Veterans' Drill) plays out in the full engine using the new skill; every check the old Forge ran on Apply / Save passes over all shipped data (ranked skill families, class and item problems, icon keys, save refusals); and the read-only Forge viewer (ranked skills, conditional item sections, no edit controls or actions, unit playtest in the arena and back, Validate All, browsing never dirties the catalog).
9. **Scene Runner Test** (5 assertions, [runner_test_suite.gd](scripts/tests/runner_test_suite.gd)): every runnable scene listed (runner and test harness hidden), suite output parsing, launching the arena with a chosen catalog encounter plus restart and back to the menu, the unsaved-Forge-work guard (second press discards), and hot-reloading a script in place.
10. **World Engine & World Forge Test** (11 assertions, [world_test_suite.gd](scripts/tests/world_test_suite.gd)): the shipped worlds load and validate (every element kind, lineage, descendants); inheritance (namespaced ids, overrides replace for the child only and old ids redirect, nearest-wins resolution, cycles and unknown parents rejected); same base / different lore (names differ, every stat, class, skill, AI plan and damage matches between `ashlands` and `tidewrack`, except the deliberately tuned boss); tuning deltas, named items (armor + mitigation, fixed vitals untouched) and disciplines apply exactly; world encounters (wave ambush, own battle, an inherited battle re-skinned) play to the end in the real engine with lore names, and `EncounterSource` lists both layers; on a sandbox copy: connections without inheriting, unconnected worlds refused, role / base / override / slot checks and safe deletes; authoring (new world, dangling references while drafting, overrides, renames rewriting plain / json / cross-world / override references, templates, save & reload); every own world entry passes the former World Forge Apply check (`blocking_errors`); and the read-only World Forge viewer (inherited and own entries, no edit controls or actions, play in the arena, Validate All) plus the Scene Runner launching a world battle.

Additionally, `scenes/gear_dojo.tscn` runs **42 dedicated headless assertions** ([dojo_assertions.gd](scenes/dojo/dojo_assertions.gd), which calls [dojo_session_assertions.gd](scenes/dojo/dojo_session_assertions.gd) and [dojo_ui_assertions.gd](scenes/dojo/dojo_ui_assertions.gd)) when launched with `--headless`:

- **Rules & formulas (Dojo-1 to 18)**: dummy vitals, melee formula with sword training, 50% mitigation, focus buff scaling, threshold tiers, deficit penalties, melee/ranged power gating, accessory stat propagation, class archetype equip restrictions, 1H/2H and off-hand scaling, crit rate and damage factor, stat allocation bounds, and reset to baseline.
- **Session behavior (Dojo-19 to 25)**: strict-hand assertions Dojo-13d, 21d and 21e (dual wield only with the dojo flag); Quick Setup presets, class-proficiency skipping, 2H shield refusal / off-hand weapon acceptance, 1-block active-weapon switching, 1H dual-wield Dual Strike, stash duplicate integrity, strike log entries with breakdowns, rejected illegal actions logged as `FAILED`, and benchmark simulation leaving dummy HP untouched.
- **UI smoke tests (Dojo-26, 27)**: all six panels mount in the workspace and the Test Log mirrors the session log; the Party & Stash screen shares the session (party list, selection, stash equip).
- **Slots, filters & carry (Dojo-28 to 32)**: off-hand-only weapons and shields, per-slot stash filters (shield under Off Hand) plus stat/rarity/damage/hand/usable/search/sort custom filters, a party viewer that always shows 4 slots, per-unit burden vs party-wide carry overload, and overlay windows opening/closing on request.
- **Catalog skills & the dummy fighting back (Dojo-33 to 38)**: the console's skills are the catalog's; a catalog attack skill (Hamstring) follows the battle rules (2 AP + 1 switch, Offensive, the exact seeded hit); Brace guards and the dummy's attack goes through the same formula (its power and stance vs the hero's guard and armor) and is logged as the dummy's; Counterattack answers every hero skill; a stunned dummy loses its action and Next Round ticks Focus, drops the guard and regains +4 AP (debt included); Rest recovers AP and a downed hero is refused until healed.

### Boss phases schema (BOSS1-L commit 1)

Encounters (catalog and world) gain `phases` (default `[]`): up to 4 boss phases `{id, name, trigger, effects, pause_autoplay}`, with trigger `hp_below {unit, pct}` or `round {round}` and 1 to 5 effects of type `line {text}`, `wave {units, label}`, `status {unit, status, rounds, magnitude}` (buffs only) or `ai {unit, role / target_pref / reckless / flee_ratio / defend_below / heal_below}`. Both catalogs validate them (`EncounterPhases.problems`), and an encounter's enemies, waves and phase waves hold at most 12 units. Commit 2 adds the runtime: a phase fires before the next block, or at the next round start when it is met at the end of the last block or by a round trigger; AI changes apply from the next enemy draft; phase waves stay off the field until their phase and do not block a win; the log shows a Phase entry (`CombatSession.phase_fired`, `CombatEncounter.phase_info`); round reset rewinds a phase; autoplay pauses on a phase unless `pause_autoplay` is false; the World Forge preview lists each phase; boss music also counts boss-tier units in phase waves. Tested in `boss_phase_test_suite.gd`.

Shipped phases (BOSS1-A): Grakka's `ashmaw_stand` and `ashmaw_stand_barred` carry "Crown of Nails" (Grakka below 50%: a line, fortify 5 and regen 4 for 3 rounds, reckless AI on the weakest hero, smokecallers heal below 90%; pauses autoplay); the barred stand adds "The Empty Gorge" line at round 4. Seed `data/seeds/boss1.gd`. When a phase fires live (BOSS1-U), a centred banner in the results popup style shows the phase name and line for 2 s on the combat screen; any key skips it; it never shows during replay or past-round viewing.

### Fight pacing schema (PACE1-L stage 1)

Data fields for the two-act boss (`docs/specs/PACE1.md` section 3); stage 2 adds the combat behaviour. A new buff `empower` (Empowered, 1..25 % outgoing power, 2 rounds, cap 25) is in `StatusRegistry` with grant `""`: no skill, and no phase `status` effect, can give it. Phase triggers `hp_below` take `or_round` (0 = none, else 2..max rounds; stage 2 makes the phase due at that round's start). The phase effect `enrage {unit, power}` (unit in `enemies`, power 1..25, default 20) is allowed once per encounter, and its phase must pause autoplay. Encounters (catalog and world) gain `pace` (`auto` default, `mob`, `elite`, `boss`); `EncounterPhases.pace_of` resolves auto from the enemy tiers, waves included. World encounters gain `start_status` (default `[]`, at most 2 story-reward start buffs `{status, rounds 1..5 (2), magnitude, requires_flags, text 1..120}`, any buff but taunt and riposte). Catalog encounters refuse it. The World Forge preview shows "or round N", the enrage effect, one `Start:` line per start buff and a `Pace:` line. Tested in `boss_phase_test_suite.gd` and `status_registry_test_suite.gd`.

Fight pacing behaviour (PACE1-L stage 2): an `hp_below` phase with `or_round` is due at that round's start when it has not fired (whichever comes first; it fires once). An `enrage` gives its unit `empower` at its power for the rest of the battle: fired mid-round it is queued (the round's remaining blocks keep their damage), fired at a round start it is active at once. Empower multiplies outgoing skill power by (1 + magnitude / 100) in `SkillDef.power_for` and the riposte counter (`RiposteRules.counter`), times weaken; heals are untouched. From the next planning phase the enraged unit's intent record has `enraged` and `revealed` true and its whole plan, with or without foresight (`CombatEncounter.is_enraged`); the phase signal info carries `enrage` (the power). Game battles (`BattleBridge.encounter_for` passes the game state to `GameEncounter`) grant each `start_status` entry whose `requires_flags` hold to every hero and guest on the field at round 1, before planning (one `Start` EVENT entry with its text, source `start:<encounter id>`); previews, the Forge and catalog battles grant nothing (`WorldEncounter.grant_all_start` is the hook for the sim's `start=on`). A boss fight (pace resolving to boss, phase waves included) without an enrage phase gets a soft warning in `ContentCatalog.warnings_for` / `warnings_all` and `WorldCatalog.world_warnings`; both previews show a `Pace:` line and the warning. The skill `grant_status` dropdown no longer offers empower. Tested in `boss_phase_test_suite.gd`, `weaken_test_suite.gd` and `status_registry_test_suite.gd`.

### Battle objectives schema (OBJ1-L stage 1)

Encounters (catalog and world) gain `objective` (`rout` default, `hold`, `escape`, `slay`), `objective_tiles` (hold; `[]` = the ring `[[5, 4], [6, 4]]`), `objective_rounds` (hold, default 3, 1..max rounds), `objective_unit` (slay, an id in `enemies`), `objective_count` (escape, 1..4 heroes), `objective_text` (at most 80 characters; empty = generated text), `deployment` (`standard` default, `ambush`, `centre`), `guests` (at most 2 party-side extras: catalog units with side party; world characters with role party, or npcs whose base unit has side party) and `protect` (guests that must survive). Old entries read as rout, standard, no guests. Both catalogs validate them with `EncounterObjectives.problems` (`scripts/combat/session/encounter_objectives.gd`); the World Forge preview shows `Objective:` (`EncounterObjectives.text_of`) and `Deployment:` lines, and the per-unit tile list is now labelled `Start tiles`. Tested in `objective_test_suite.gd`.

### Battle objectives runtime (OBJ1-L stage 2)

Deployment comes from `CombatGrid.DEPLOYMENTS`: `standard` is the GRID12 layout unchanged (party x 5 / 4, enemies x 7 / 8); `ambush` keeps the party at x 5 / 4 and splits the enemy roster by order into group A (x 7 / 8) and group B behind the party (x 2 / 1); `centre` puts the party on the ring (x 5 / 6) with A at x 8 / 9 and B at x 3 / 2. In ambush and centre, leftovers, the wave and BOSS1 phase waves alternate A, B and take the free tile nearest the group front on row 4. Guests are built after the heroes (`CombatEncounter.guests`, also in `allies`), deploy with the party rule, are planned like heroes, are drafted Balanced by party-wide AI drafts (`party_preset_for`), have no `hero_key` (no saved tactic), get no XP and are never written to `PartyState`. After each block (after round-end effects and BOSS1 `on_block_end`) `CombatEncounter.verdict(round, block, end_of_round)` decides: every hero down or a `protect` guest down is lost; every enemy that counts down or withdrawn is won; then hold (at each round end only: party side alone on the tiles +1, enemies alone -1, floor 0; won at `objective_rounds`, also on the last round), escape (`objective_count` heroes out, none left on the field, protect guests out) or slay (the unit down is won; the unit getting away is fled, "%s got away."); finally the party gone by the exits is fled. The session stores it in `CombatSession.outcome`; `BattleBridge.outcome`, the playtest end result and the sim's win score read it (the round limit with both sides standing stays fled). `EncounterObjectives.held` is kept in `BlockSnapshot.objective_state`, and `reset_round` restores it together with `reinforcements_spawned` (a reset round now re-runs its wave). Rout battles recorded before this change replay identically.

### Enemy intent tells (INTENT1-L)

While planning, `CombatSession.enemy_intents()` lists each active enemy's first scheduled action as `{unit, kind, target, cell, blocks, skill, revealed, readers, plan}` (`intent_of(enemy)` gives one record; rules in `IntentRules`, `scripts/combat/session/intent_rules.gd`). Kinds are the action kinds in `IntentRules.KINDS`, `idle` ("Holding") for an enemy that was on the grid when the round was scheduled but has no action, or `arriving` for a unit placed after scheduling (a wave); an area attack carries its aimed tile. An enemy whose actions this round target a party unit with the new `foresight` passive effect (`Combatant.has_foresight()`, `foresight_source` names the passive) shows its whole round `plan` in block order. The tells are display only. Enemy drafting (`CombatEncounter.schedule_enemy_ai`) re-drafts any packmate whose plan a later packmate's draft broke (bounded passes), so no on-grid enemy idles by accident. In planning (INTENT1-U) each enemy shows its next action as a chip and arrow on the grid (dashed for move; `I` hides them, setting "Show enemy tells"), "Next:" on its hostile card, an INTENT line in the hover card, the tell in the timeline banner, and `Targets you in B<n>` in the forecast; an enemy read by a Warden's Eye holder gets an eye badge, its full plan in the hover card and an unmasked read-only timeline track. UI checks in `combat_ui_framework_test_suite.gd`. Tested in `intent_test_suite.gd`. Bryn Hale starts with the shipped source, passive `wardens_eye` ("Warden's Eye", seed `data/seeds/intent1_a.gd`, INTENT1-A).

### Combat extensions (M1)

Support skills (heal/buff on allies, `SupportRules`), gap-close attacks (`dash_tiles`), haste/bleed/taunt statuses, `learned_only` skills, a `reckless` AI profile and AI fixes (kiting, no idle turns, support healing); tested in `combat_extensions_test_suite.gd`, `combat_ext_attacks_test_suite.gd` and `combat_ext_ai_test_suite.gd` (helpers in `combat_ext_kit.gd`). Example skills `mend`, `rally`, `taunt_shout`, `lunge`, `bleeding_cut` are in the catalog but in no class path yet (M2).

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

`cleanse` (support skills can remove debuffs from an ally, combinable with heal and buff), the `DISENGAGE` jump (move away from the nearest foe), the L-shaped `SWEEP` area for the Brawler, and a gear-budget value for unarmed loadouts. Example skills `purify`, `backflip`, `sweep_kick` are in the catalog (learned_only). Tested in `engine_pieces_test_suite.gd` (unarmed and gear budget in `unarmed_test_suite.gd`).

### Foundation cleanup (slice A)

Basic skills are now six single-tier skills (strike, guard, move, rest, switch_target, offhand_jab); the old variants (power strike, heavy blow, brace, fortress, deep focus) are gone. New: a `StatusRegistry` for all statuses and stacking rules, the `unarmed` skill token, derived weapon hands (`main_1h`, `main_2h`, `off`), and a `skill_potency` passive hook. Tested in `status_registry_test_suite.gd` and `foundation_test_suite.gd`.

### Rest and stance (B1)

Rest now puts the unit in Tactical stance (free entry). The skill after Rest pays the usual `CombatTypes.STANCE_SWITCH_COST` (1) only if its stance differs: Rest then Focus has no switch cost, Rest then an attack or Defend costs +1.

### Orb and tome, item effects (C1-L)

Orb and tome are off-hand-only focus weapons; items carry an `effects` list (spell/heal potency, status potency, spell reach, resist) that applies only while equipped; a two-handed main weapon cannot be paired with them. The Dojo Arcanist preset is now wand plus tome. Tested in `focus_gear_test_suite.gd` and `focus_gear_budget_test_suite.gd`.

### Weaken and data-driven riposte (D1-L)

New `weaken` status (outgoing skill power -15 to -25%, cap 25%, refresh to the longest duration) with a `status_magnitude` skill field; zero-power attacks apply a status without dealing damage; riposte AP, mitigation and duration now come entirely from skill data. Tested in `weaken_test_suite.gd`.

### Status expansion schema (S12S-L commit 1)

The status registry gives every status a magnitude range and unit, and adds six rows with placeholder numbers: slow, expose, fortify, regen, poison and burn. They have no effect in combat yet (S12S-L commit 2). `status_magnitude` is validated per status on attack, support and defend skills, the `{magnitude}` text token shows its unit, and each status records which unit applied it (undo restores it). Tested in `status_expansion_test_suite.gd`.

First carriers (S12S-A, placeholder numbers): five new families unlock at level 6, after every level-5 family, so no default loadout changes: Crippling Shot (ranger, slow), Rend Armor (warrior, expose), Envenom (rogue, poison), and for the priest Aegis (fortify) and Rejuvenate (regen). Seed `data/seeds/s12s.gd`.

### Status expansion effects, AI and sim (S12S-L commits 2-3)

The six new statuses now act in combat. Slow takes one tile off each move block (never below 1; haste and slow cancel). Expose lowers and fortify raises the armor rating by their magnitude (never below 0), and the attack, the planner's damage estimate and the forecast all use that rating. At round end each unit takes bleed, then poison (its stacked magnitude), then burn (a % of max HP), then regenerates (a % of max HP). Each step has its own log line, and a unit defeated earlier in the round end does not regenerate. Poison applications add up to a cap of 30. Support and defend grants carry the skill's magnitude and the caster's name, and status chips show the magnitude with its unit (for example "Poisoned 12 HP"). A magnitude of 0 on a status that has a range becomes the range minimum. The AI keeps poisoning a foe until the cap. It aims expose at its main target, then at a foe next to an ally, and slow at the nearest foe that can reach the frailest ally. It puts fortify on the ally with the most adjacent foes and regen on the most wounded ally when no heal fits. The balance sim counts regen as healing done by the regen caster (`regen_received`) and credits damage over time to the unit that applied it (`dot_dealt`). Tested in `status_expansion_effects_test_suite.gd` and `status_expansion_ai_test_suite.gd`.

### Area statuses and taunt from defend skills (M2C2B-L)

Area attacks now roll their status once per unit hit (allies included); defend skills can grant riposte (immediate) or a support buff like taunt (queued). Tested in `skill_engine_test_suite.gd`.

### Progression and saves (M3)

Party state, XP curve (starting values in `data/progression/xp_curve.json`), level-up spend, a builder that turns a saved unit into a combatant, and a versioned single-slot JSON save with migrations (`user://saves`). No UI yet. Tested in `progression_test_suite.gd`, `save_test_suite.gd` and `save_compat_test_suite.gd`.

Save compatibility (SAVE-COMPAT): `scripts/tests/fixtures/saves/save_v<N>.json` holds one sample save per save version, built from a played Ashlands state (party, gear, quests, claimed loot, gold, shops, exploration, world clock). `save_compat_test_suite.gd` decodes each one every gate run (migrate + validate against the shipped catalog and worlds), so a change that breaks old saves fails the gate. A `SaveGame.VERSION` bump or a format-breaking change adds or updates the fixtures in the same task (`tools/fc.sh run tools/gen_save_fixtures.gd`, then review the diff); renaming a shipped id the fixtures use fails the suite on purpose.

### Class skill content (M2C2B)

All six classes now have their real kits: three-tier skill families, eight advanced families that replace two starters in one slot, intimidate for the Warrior, riposte at AP 3/2/1, orb and tome focus items with effects, and rogue and mage party units. Default loadout = the first five path families. Authored by idempotent seeds in `data/seeds/`.

### AI skill use (AI1)

The tactical AI (enemies and preset drafts) now uses disengage, area attacks (never hitting its own side), cleanse-only skills, taunt, ally buffs, debuff attacks and riposte. The old `AiProfile.smart_skills = false` path was removed after the BAL2 rerun (CLN-16). Tested in `ai_skills_test_suite.gd` and `ai_skills_tactics_test_suite.gd` (fixtures in `ai_skills_kit.gd`).

### Skill unlock rule (SK2)

Skills open by unit level: a tier is castable at the family unlock level plus a per-tier offset (0, 2, 4). AP is only the per-round budget. Advanced families need level 5 or higher; KNOW and the new `unlock` passive effect can lower unlock levels (max 2). Unlock levels freeze during battle. Tested in `skill_unlock_test_suite.gd`.

### Atomic family edits (SK2-A)

`ContentCatalog.upsert_many` applies a set of entries all-or-none (a skill family whose ranks must change together). The SK2 data step flags the 11 advanced families, moves the warrior's riposte to level 3 and adds the passive templates `field_medic`, `arcane_study` and `riposte_drill`.

### Forge skill tab (SKILL1)

A skill is one **family template** plus up to two **rank modifiers** (absolute values resolved against the base, no cascade); rank names derive from the family ("Mend II"). `data/catalog/skills.json` is now version 2 (49 ranked entries on disk, still 127 flat rows in memory; `SkillRanks` folds and expands). Descriptions are templates with tokens (`{power}`, `{heal}`, `{rounds}`, ...; `SkillText`), caps 32 name / 240 description. New fields: `grade` (novice, advanced, expert reserved) replaces `advanced`, `requires` (all listed families, strictly lower grade) replaces `evolves_from` (seven pairings such as greater_mend requires mend), `weapon_hand` (main, off; `both` disabled) replaces `offhand`, and `origin` (system/user): system assets have key fields locked (data rule, `ContentCatalog.key_field_errors`), can be cloned through the API, and only user assets can be deleted (`promote` / undoable `demote`). Basic skills are granted to every unit from level 1 and are not valid in class paths. Saving never drops data: a kind that loaded with errors is save-blocked until fixed. `tools/fc.sh safe_delete <kind> <id>` scans references before a delete. The Forge skill view (read-only since TOOLS-RO) is driven by `SkillFormModel` (sections per kind, limits, problem reasons), `SkillFilter` and `SkillRanksPanel`. Migration seed `data/seeds/skill1_ranks.gd`. Tested in `skill_ranks`, `skill_text`, `skill_model`, `origin` and `skill_form_model` suites.

### Balance rules and sim v2 (BAL2-L)

- **Range**: the +1 attack range at PERC 18 (`CharacterStats.RANGE_PERC_THRESHOLD`) now applies only to ranged weapons (base range above 1); melee units stay at range 1.
- **Unit gear `none`**: a unit slot set to `none` (`ContentSchema.NO_ITEM`, pouch `["none"]`) opts out of the class default loadout; a blank slot still means "use the class default". Units only.
- **Tiers**: elite 1.25 HP / 1.0 power (BAL3 Q3; was 1.1), boss 1.5 / 1.1 (`StatRules.TIERS`).
- **Trained fists**: an empty main hand with fist proficiency 2 or more hits with weapon power 15 + fist training and gets the fist crit bonus (`DamageCalculator.UNARMED_BASE_POWER`, `TRAINED_FIST_MIN_LEVEL`); `GearBudget.UNARMED_VALUE` reads the same constant. The damage formula itself is unchanged.
- **Balance sim**: `tools/fc.sh sim <duel|tier|levels|group|stats> [level] [seeds] [key=value ...]` (helpers in `tools/sim/`). Keys: `cap` (default 10), `loadout=default|newest`, `spend`, `preset`, `distance`, `score=both|hp|binary`, `quick`, `band=lo-hi`. Refused plans print `PLAN FAIL <unit> <reason>` (with the step and the unit holding the tile) and every mode ends with `plan checks: N PLAN FAIL over M fights`. Cells print `binary/hp-margin` scores; duel rows outside the 30-70 diagnostic band are flagged LOW/HIGH. Modes `party` (up to 4 vs catalog or generated mob/line/wave/elite/boss encounters), `role` (five limit-test scenarios per role, start distance 4, KPIs as mean [floor..ceiling] with ranking) and `sweep` (all 4-class compositions, swap delta, top-5 counts) use a per-unit metrics collector fed by each unit's `health_changed` signal; it self-checks every log entry against real HP changes and prints `METRIC FAIL` on any mismatch. Off-target values are marked X; `target=<key>:<lo>-<hi>` overrides a band. A full role run is about 28 min (split with `role=` or raise `SIM_TIMEOUT`).
- Tested in `bal2_rules_test_suite.gd`.

### Balance sim levers (BAL3-S)

- **Start jitter**: by default every fight shifts each unit's start tile by up to 1 tile inside its own half (never onto a unit, wall or exit) and shuffles the party order, both seeded by the fight seed (same seed, same layout); `jitter=off` restores the standard deployment for A/B. Applies to every mode; role mode applies jitter before the start-distance placement.
- **Generated encounters**: mob 5 minions; line 4 standard (warrior, brawler, ranger, rogue); wave 3 standard + 3 minion wave; elite = elite + rogue, mage, warrior standard; boss = boss + ranger, priest standard + ranger, priest minion wave at round 2. Threat = sum of tier HP x tier power (minion 0.24, standard 0.56, elite 1.25, boss 1.65; party of 4 = 4.00): 1.20 / 2.24 / 2.40 / 2.93 / 3.25.
- **Role scenarios**: `aoe_swarm` starts its 6 melee minions (3 brawler, 2 rogue, 1 priest) in a 2x3 block of adjacent tiles at the start distance; `striker_priority`'s priority mage is standard tier at L3 and below (elite above); `ranged_dive` unchanged.
- Tested in `bal2_rules_test_suite.gd`.

### Balance data pass (BAL2-A)

Seed `data/seeds/bal2.gd`: CONS growth 1 for ranger, rogue and mage (warrior CONS 2 STR 2); warrior carries a kite shield, and the gear budget fix D1 lowers kite mitigation to 6 and plate to 8 (warrior L5 EHP 468, every class budget 19-27); ranger starts with `aimed_shot` and brawler with `body_blow` at level 1; the shipped warrior, ranger, rogue and mage units carry explicit role loadouts; goblin_shaman fights with a cleric mace, robe, no off-hand and no potion; the brawler class is unarmed (trained fists).

Placeholder art (CLN-15, seed `data/seeds/cln15_art.gd`): goblin units now set their `art` key (archer, shaman, skirmisher, berserker, scout; goblin_brawler reuses berserker) instead of falling back to their class art.

### Cleanup batch 1 (CLN-3, 6, 7, 9)

Refactor only, no behaviour change: per-kind validation moved out of `content_catalog.gd` (1416 -> 993 lines) into `scripts/content/validators/` (`SkillValidator`, `ItemValidator`, `ClassUnitValidator`); the legacy `SkillDef.offhand` / `advanced` shims are gone (`is_off_hand()`, `grade`), and old skill keys or a version 1 skills file still load but print a deprecation warning (CLN-3b later removed both: the old keys are no longer translated, and a version 1 skills file is a load error that blocks saving); the second power formula `CharacterStats.get_melee_power` and other dead helpers were removed, so attack power comes only from `DamageCalculator`.

### Cleanup batch 2 (CLN-8, 9b, 14)

Refactor only, no behaviour change: shared test helpers live in `TestHelpers` (`report`, `report_ok`, `failed`, `unit`, sandbox `fork` / `cleanup` / `fork_worlds`; `SkillTestKit` folded in) and every test sandbox is unique per process (`TestHelpers.sandbox(tag)` -> `user://test_sandboxes/<pid>_<n>_<tag>`), so parallel suite runs no longer collide; the `ArenaTestSuite` pass-through is gone (`scenes/test_arena.gd` calls `SuiteRegistry.run_all`); `content_schema.gd` keeps its API while the per-kind field tables moved to `scripts/content/{skill,item,class_unit}_fields.gd`.

### Quest schema (M4-L slice 1)

Quest templates live in `data/quests/quests.json` (`QuestCatalog`); world quests bind them to world locations, encounters and flags and add metadata `links`, flag-conditioned `variants` (only whitelisted fields, so no branching inside a quest), a `turn_in` NPC and a `loot_pool` drawn from the new per-world loot table (`loot.json`). Validation and reference scans (incl. renames through links, variants and pools) are in `QuestRules` and `WorldCatalog`. The runtime (slice 2) adds `QuestRunner` and `RewardGrant` (turn-in is all-or-nothing; claimed loot leaves every pool) and save version 2 with a built-in migration that refuses damaged quest data instead of silently dropping it. Tested in `quest_test_suite.gd`.

Sample quests (M4-A, seed `data/seeds/m4a.gd`): four lore-free templates (reach_and_clear, two_stop_delivery, clear_site, signal_and_return), an Ashlands loot table (wardens_drill, rallying_cry, keep_ring, keep_stores) shared by three quest pools, the new Ashlands quests cinder_gate_watch and warden_oath (variants for a fallen Mara or Tev) and the existing quests bound to templates.

Ashlands campaign "The Long Dry" (CAMP1-A1/A2, seeds `data/seeds/camp1_unit.gd` then `camp1_story.gd`): after the oath, `steel_for_the_gate` (auto start) sends the party to Slag Gate (now the playable `walled_town` map, Orsa's steel at cost) and raids the wagon on the south road; Mara's steel choice either bars the Cinder Gate (Wardens +10, the finale loses its wave) or arms the party (an `arm_plate` plus 4 steel rows at Hest); `keep_one_fire_lit` opens the `wastes_track` to the new Ashmaw Camp, where Grakka Ashmaw's parley is the first dialogue `fight` (boss unit `goblin_warlord`, encounters `ashmaw_stand` / `ashmaw_stand_barred`); Mara's epilogue sets `ashlands.campaign_done`.

### Region map schema (EXP1-L commit 1)

Lore-free region map templates live in `data/maps/maps.json` (`MapCatalog`, rules in `MapRules`); world locations bind a template with `map`, `zone_bindings` and `markers` (encounters, patrolling roamers, chests, lore, exits), and a world manifest names its `start_location`. Reference scans rewrite markers and bindings on rename. Tested in `exploration_test_suite.gd`.

The exploration runtime (EXP1-L commit 2) walks a region map step by step: a seeded step counter (threshold 12-20 steps, 10% elite chance, from the zone binding) starts mob fights, markers start placed fights, roamers patrol fixed paths, chests grant loot once, exits move between locations (world-map exits: TRAV1). `BattleBridge` builds every game fight from the world encounter plus the active party (`GameEncounter`), awards XP, calls the quest hooks and maps the result to won, fled (round limit or the whole party withdrawing) or lost (party wakes at the last safe tile or spawn, fully healed). `GameRolls` gives every save its own deterministic random streams, so a reload replays the same fights. Save version 3 adds the position, exploration state and roll counters, with a built-in 2 to 3 migration.

Debug region map (EXP1-L commit 3, Scene Runner > Region Map (debug), `scenes/overworld/region_map_debug.tscn`): pick a world and start a new game, walk with the arrow keys, press E to interact, use exit buttons in menu locations, and fight in the real combat arena (BATTLE1-L, below); point-of-interest (`point`) markers show the cyan ring icon (asset key `overlay_poi`, the old combat centre-ring marker, GRID12); it writes no save: only a game started from the Main Menu autosaves to slot 0 (REVIEW-F1; debug slot 9 before UI1-L, slot 0 until REVIEW-F1). EXP1 is complete. The world picker, New game and the debug status line (location id, position, steps, rolls) appear only in this debug scene (`RegionMapView.debug_tools`, set by the scene file; MAPDEV1-U): a game opened from the Main Menu shows the place name and "Supplies n / 99" in the bar instead, the region map's New game is refused there, and the log names places ("Entered Warden Keep."). Chest notes name the items ("Found: Healing Draught x2, 4 supplies").

Region map controls (POI-FIX): arrow keys move, E interacts with the npc, chest or lore marker you face; bumping one says "press E"; left-click a marker next to you to interact (a far one says to walk next to it first); E on nothing says so; a hint line under the top bar names the keys. MAPKEYS1-U: W A S D move like the arrow keys; holding a key keeps walking (first step on press, repeat after 0.25 s at about 9 steps/s; stops on release, a fight, a conversation or a popup); the party token glides between tiles and the camera follows it smoothly; marker names stay by their own marker and never overlap (crowded ones are hidden until you hover the marker).

Full-screen maps (MAPVIEW1): the region map fills the window and the top bar, hint and side column (menu, fight panel, dialogue, shop, log) float over it on semi-transparent panels. Tiles are 32 px (1x pixel art, nearest filter) and drawn from the ui icons `region_ash_floor_0..2`, `region_basalt_wall_0..2` and `region_water_0..2` (picked by a stable hash of the tile) when the asset manifest has them, else as today's colours; a map larger than the view scrolls with a camera that follows the party and stops at the map edges, and a smaller map is centred. The world map fills the window the same way: when the manifest has the ui background `<world>_world_map` (Ashlands: `ashlands_world_map`, 1920x1080) it is drawn letterboxed behind the routes and pins, and a location with `map_pos` [x, y] (0..1) gets its pin at that point; otherwise pins keep the circle layout. Tested in `map_view_checks.gd` (run by the exploration suite) and `world_map_checks.gd`.
Readable maps (MAPREAD1-L2): every visible marker on the region map draws its icon (tile keys `overlay_npc/chest/chest_open/lore/exit/exit_locked/encounter/roamer/poi`, generated by `MapArt` until files override them; an npc with a character `art` key shows its unit token; a used-up marker is 40% opacity, a roamer pulses each second, a used chest shows the open one, a locked exit the padlock; the letter glyphs are gone). npc and exit names are always drawn above their tile; hovering a marker tile or facing one shows `label · verb` in the hint line; a gold `!`/`?` badge (ui `badge_quest`/`badge_ready`) marks an npc with a quest or a ready turn-in; the side column lists a "Here" legend (npcs, exits, chests, lore, points, fights; hovering a row pulses its tile). Walking onto a point logs its `note` ("You look around." without one, "Nothing more here." once used) instead of the raw flag line. World map pins use the ui keys `pin_<kind>` (`pin_here` ring + "You are here"), a gold `!` marks pins a quest step points at, hovering a pin opens a card (name, kind, region, visited, `TravelRules.trip` time and supplies, `TravelRules.quest_steps_at`), and a legend (pin kinds, open/closed/one-way roads) sits bottom-left. Map text follows the Text size setting (`RegionMapView.name_font_size`). Tests: `map_read_checks.gd`.

Real battles from the map (BATTLE1-L): a fight on the region map (step counter, marker or roamer) or from a dialogue opens the combat arena with the saved party at its real levels, gear and loadouts (`BattleScreen` in `scripts/ui/overworld/`, built on `BattleBridge.encounter_for`). A map fight starts with a short skippable transition (banner "Ambush!" or `Battle: <name>` with the enemy count; instant with Skip animations). When the battle has an outcome it settles about a second after the last blow and the results open as a popup over the still-visible arena (BATTLE-FLOW). Won awards XP and advances quests, lost wakes the party at the last safe place fully healed, fled steps back to the tile before the fight; then the results popup shows each active member's XP bar (progress to the next level, the gained part highlighted), every non-XP reward (gold, supplies, items, loot, reputation, quests advanced, when the fight gives them), and Replay (plays every round again with the popup minimised), minimise (a "Victory ▴" pill) and ✕ in its header; ✕, Esc and right click autosave and return to the map. Tested in `battle_flow_test_suite.gd`. HP is never carried between fights (full heal after battle, decision M3). If the arena cannot start, the fight panel shows why with Start battle and Dev: won / fled / lost buttons; `RegionMapView.dev_outcomes` shows those buttons always. Tested in `battle_link_test_suite.gd`.

Quest ready flag (QUEST2): a quest that becomes ready sets `quest.<qid>.ready` (kept through turn-in, cleared by abandon); the Ashlands warband and Cinder Gate ambush markers hide once their fight is won while the quest is active.

Region map data (EXP1-A, seed `data/seeds/exp1a.gd`): maps `walled_town` and `gorge_ruins`; Ashlands starts at Warden Keep; the new Cinder Gate location; generic Horde characters and mob encounters; markers for patrols, chests, lore, a beacon and the gate that opens once the courtyard is held. World-map exits: TRAV1-A.

### Campaign attrition sim (BAL4-ATTR)

- `tools/fc.sh sim campaign [level] [seeds] [heal=none|full|rally|<pct>] [rest=<b,b>] [extra=<b>:<enc>,..] [comp=c1,c2,c3] [elevel=..] [steel=arms|gate]` plays the Ashlands story fights in quest order (`courtyard_skirmish`, `pass_ambush`, the road raid `ash_raiders`, then `ashmaw_stand` or `ashmaw_stand_barred` for the steel-to-gate choice), read from the world data (`tools/sim/sim_world.gd`).
- Party: the warrior plus two recruits (all 10 trios by default) at party level 5; foes keep their authored levels; each fight uses the encounter's `max_rounds`.
- Attrition: HP carries over, statuses reset, downed heroes stay down (`heal=full` revives, `heal=<pct>` heals the standing heroes by that share of max HP, `heal=rally` is `HealRules.hp_after` "won" per hero and a downed hero gets up; `rest=<b,b>` fully restores everyone before those story battles; `extra=<b>:<enc>` plays a filler fight before story battle b, a loss there is a fall at b, labelled with the extra fight, not counted in wins or the HP columns); a run ends at the first uncleared battle. Prints battles survived, the battle where it fell and party HP% after each battle, per trio and steel choice.
- World encounters in party mode: `enc=world:<id>[,world:<id2>...]` or `world:<world>/<id>` (default world ashlands) runs a story encounter: story characters mapped to base units at authored levels (`elevel` shifts them), the encounter's `max_rounds`, objective, deployment, phases and guests; an objective win counts as cleared; any `comp=` of 1-4 classes (sweep accepts world ids too). Example: `tools/fc.sh sim party 5 1 comp=warrior,ranger,priest enc=world:ashmaw_stand`. The campaign mode uses the same full encounters.
- BAL5-SIM keys: `world=<id>` (world for `enc=` lookups; story characters spawn through `WorldFactory.apply_character`, tuning applied), `size=` / `lead=` (sweep `size=3 lead=warrior` gives the 10 warrior trios; party and sweep default 10 seeds), `tier_foe=mirror|<class>`, `phase=on|off` (generated boss phase, default on), `s2=on|off`; aoe_swarm is a trio; tank prints allies hp_end on won fights; support prints heal casts, early-down share and down round; jitter reach refusals print PLAN SKIP and are skipped.
- MORALE1-S: world-mode `full` encounters carry the five morale fields (`morale`, `morale_break`, `morale_leader`, `morale_groups`, `morale_text`; `tools/sim/sim_world.gd` `MORALE_KEYS`), so `world=<id> enc=<id>` sims fight with the data's morale.
- Tested in `bal4_campaign_test_suite.gd`.

### Balance data pass (BAL3-A)

- `sweep_kick` power_mult 0.7/0.9/1.05 -> 0.8/1.0/1.15 (brawler aoe_swarm foes/cast 1.43 -> 1.57), seed `data/seeds/bal3.gd`.
- Elite tier power 1.1 -> 1.0 (HP 1.25 kept); the sim warrior tank loadout drops `riposte` so `shield_wall` is drafted.
- BAL5-A (T3 and the DR3 potion trigger fired in BAL5-R1b): `shield_wall` also grants fortify 4 / 6 / 8 (ranks 1-3, same rounds as its taunt) through `extra_grants`; the Hest healing potion costs 30 gold (was 15; sell value unchanged). Seed `data/seeds/bal5.gd`.
- Tried and reverted (no effect on their metric): `firebomb` reach 3, `mend` heal 10/12/14, `renewal` regen, kite shield mitigation 4-5. Details in `docs/BALANCE_REPORT.md`.

### AI pressure levers (BAL3-AI)

- **Incoming damage**: `TacticalAI.incoming_damage` estimates the strike chains of foes that would pick a unit by a given block (taunter first, then the foe's own preference; area attacks, crits, guard and cover left out).
- **Focus under pressure**: Focus goes before a cast only while the incoming damage before that cast resolves is below 50% of current HP (`FOCUS_PRESSURE_RATIO`).
- **Faster tier**: a unit expected to take at least its HP this round takes the highest-damage option that resolves by block 2 (`FAST_BLOCK`). Applies to every unit; the pick is by damage, so under pressure it may override the combo order.
- **Heal reservation**: a SUPPORT unit with no heal in its draft appends its lowest affordable heal tier, last in the round, on the friend projected lowest at or below `heal_below` (0.7) by round end; recomputed every round; it never adds a move.
- **Taunt pull**: a unit whose top target is a taunter it can strike or reach within this round's move commits to it: the taunter is its only target (it walks to it when the strike does not fit), and a hurt skirmisher's disengage and the low-HP guard while the taunter is out of reach (both would cost the move) are skipped.
- **Early taunt (BAL5-AI T2)**: a taunter also casts taunt when any foe would pick an ally this round (`_likely_target`), besides the 3-tile rule; when the taunt cannot go up in place, it walks toward the nearest diver and taunts after the walk (and still strikes if a strike fits there).
- **Healer self-care (BAL5-AI S1)**: a SUPPORT unit expecting at least 50% of its current HP as damage this round heals itself first with its lowest fitting heal tier (counts toward `MAX_HEALS`); at full HP or with no fitting self-heal it steps back one tile.
- **Friendly-fire tolerance (BAL5-AI A2)**: an area cast may hit one ally when it hits at least 3 foes and costs that ally under 25% of its current HP; on equal damage, fewer allies hit ranks first.
- **Minion skill ceiling**: minion-tier units without an explicit `skill_cap` use AP ceiling `StatRules.MINION_SKILL_CAP` (2, tiers I-II).
- Tested in `ai_skills_test_suite.gd`, `ai_skills_tactics_test_suite.gd` and `planning_ai_extra_test_suite.gd`.

### Plan claims (PLAN1)

Preset auto-plans are always confirmable: an ally's approach, go-to or landing tile (support approaches included) is fixed once planned and later drafts keep off it (`CombatPlan.pinned_cell`), speed ties keep planning order, and the party preset re-drafts any ally a later draft made invalid. Enemy packs share the same planner, so they no longer plan onto a packmate's tile, and `schedule_enemy_ai` re-drafts any packmate a later draft broke (bounded passes, then trims to the valid prefix). Tested in `plan_claims_test_suite.gd`.

### Item families and items v2 (ITEM1-L commit 1)

Items are now built from item families (`data/catalog/item_families.json`, 20 families such as sword, shield, plate, orb): a family holds the hands, slots, scaling, base numbers and bands; an item stores its family plus a list of modifiers and tier thresholds (`items.json` version 2), and memory keeps flat resolved rows (`ItemFamilies`, `ItemModifiers`, `ItemText`). Strict hand typing applies everywhere except the Gear Dojo. A system item's modifiers and tiers are locked as whole lists, and one edit may change either the modifiers or the flat numbers, never both. The migration seed `data/seeds/item1_migrate.gd` (run in ITEM1-A) keeps every item value and class total identical. ITEM1-A ran it: 31 items now belong to 20 families (`item_families.json`), `items.json` is version 2, and item descriptions that quoted numbers use tokens (seed `item1_tokens.gd`), so the kite shield and plate texts now show their current mitigation. Tested in `item_families` and `item_modifiers` suites.

### Item tiers (ITEM1-L commit 2)

An item tier can now require a stat (training lowers the threshold), a proficiency level or a unit level, and can grant power, mitigation, stat bonuses or effects; armor and shields can carry tiers too. Gear is applied in a fixed order (static gear bonuses, then tier stats, then gear effects) in one pass, so a tier's bonus never unlocks another tier. `GearBudget.tier_potential` shows half a tier's value as a preview and never counts toward the budget. Shipped items keep their power-only weapon tiers, so no current loadout changes. Tested in `item_tiers_test_suite.gd`.

### World map and travel (TRAV1-L)

Worlds gain the kind `routes` (`data/worlds/<id>/routes.json`: `from`, `to`, `two_way`, time 1-20, zone, encounter / repeat / elite chances, encounter and elite lists, up to 4 scripted road fights, open and closed flag conditions), validated with warnings (a chance with nothing to fight, a one-way route into another world with no way back) and reference scans (`TravelRules`). World map nodes are the route endpoints; a location with a `world_map` exit must be one, and its exit's `sets_flags` now apply. Worlds linked by `connects` share their two-way routes. `TravelRunner.travel` auto-paths a trip (least time, then fewest legs) leg by leg: each leg advances the world clock (`GameState.time`), then runs a scripted fight or a seeded road roll (`travel` stream; repeat chance -1 = half the encounter chance); a fight pauses the trip (won completes the leg, fled stays put, lost wakes at the leg's start). Fog hides nodes until visited. Save version 5 adds `time` (migration from 4). Save version 6 adds party supplies (0..99, a new game starts with 10 or the world's `start_supplies`): each leg costs its route time; buy them at shops (`supply_price`, fixed, ignores reputation) and find them in chests (`supplies`, overflow above 99 is lost); at 0 road fights are x1.5 likely (rounded down, capped at 100); travel is never refused. The world map and shop panel show the count. The debug world map (Scene Runner > World Map (debug), `scenes/overworld/world_map_debug.tscn`; also opened by the region map on a world map exit) shows known locations as pins and routes as lines (closed dashed, one-way arrows), travels on a click, fights road battles in the real arena and shows the clock. Its debug status line (location id, time, supplies, rolls) and, standalone, the world picker and New game appear only in the debug scenes (`WorldMapView.debug_tools`; the region map passes its own flag when it opens the world map, MAPDEV1-U); in play the status reads "Warden Keep   Time 3   Supplies 10 / 99" (plus the out-of-supplies warning at 0) and arrivals name the place. Tested in `travel_test_suite.gd`.

Ashlands supplies (TRAV2-A, seed `data/seeds/trav2a.gd`): a new Ashlands game starts with 10 supplies (manifest `start_supplies`), Hest's stores sell them at 2 gold each (`supply_price`), and the chests `stores` (Warden Keep, 4), `ruin_cache` (Ruined Courtyard, 6) and `camp_cache` (Cinder Gate, 6) top them up. Iron Reaches and Tidewrack have no supply source yet.

Battle autoplay logic (AUTO1-L; its controls are in the timeline row, COMBAT-UI2-L3): `CombatAutoplay` drafts every hero's plan each round from a party tactic (Balanced, All-out Attack, Focus Weakest, Guard & Counter, Kite at Range, Economy) with per-hero overrides or Custom, confirms and plays it, and advances; the player can pause (the next round stops drafted), and it pauses itself when a hero is downed or drops to 30% HP or lower that round, or when a draft is invalid. Save version 7 keeps the tactics and the on/off switch per save (off in a new game). Playtest rounds record `source` (player or auto). Tested in `autoplay_test_suite.gd`.

Battle items, schema (ITEMUSE1-L stage 1): every hero has a battle pouch (`UnitState.pouch`, up to 3 consumable ids, repeats allowed) that `PartyBuilder` loads into `inventory.battle_pouch` in slot order; save version 8 stores it (older saves load with empty pouches). Consumables gain `use_reach` (-1 = type default: potion 1, elixir 0, bomb 3; else 0..4, 0 = self), `PlanStep.Mode.ITEM` (`PlanStep.use_item`) is the step shape and `AiProfile.use_items` (off) the AI switch. Tested in `item_use_test_suite.gd`.

Battle items in play (ITEMUSE1-L stage 2): in planning, `Use <item> (<n> left)` is one more option per distinct pouch item on its valid targets (potion: self or an ally within 1 tile, elixir: self, bomb: a foe within 3). The step costs 1 AP and the item's blocks with no auto-move, and resolves as an `ItemCommand` where support skills resolve: heal capped at missing HP, bomb flat damage (no armor, guard or stance, never riposted), elixir replaces the Focus boost. The item leaves the pouch only when the command resolves; a downed user or target, or a target out of reach, keeps the item and the log says why. Undo never touches the pouch, and `reset_round` / `reset_battle` give used items back. After a world battle, whatever the outcome, `GameEncounter.write_back_pouches` saves each hero's unused items (guests are skipped). The Gear tab model (`GearModel.pouch_add` / `pouch_remove`) fills the pouch from the stash. Heroes no longer get their class's default pouch in battle (`PartyBuilder.HERO_KEEPS_CLASS_POUCH`), and the shipped classes carry no pouch (ITEMUSE1-A2, decision "No free class potion"). Autoplay and enemies never use items; the sim key `pouch=<id,id,id>` gives every party unit that pouch and lets TacticalAI drink below 35% HP or finish a foe with a bomb. Playtest recordings store each ally's battle pouch at battle start (`pouches`), and replays rebuild it, so item steps replay the same way (older recordings without it replay as before).

Battle pouch on the Gear screen (ITEMUSE1-U): out of battle, the Gear screen has a "Battle pouch" section under the slots and stash: three slot rows (item name + Remove, or a muted "Empty") and the stash consumables the pouch accepts, each with its count and an Add button (disabled while the pouch is full). Add and Remove edit the `GearModel` draft like gear (the stash count changes at once; nothing is written until Confirm; Cancel or Back discards). Slots and stash sit in a scroll area (`GearScroll`) so Confirm / Back stay on screen at every text size. Tested in `party_screen_test_suite.gd` (`_test_pouch_screen`).

Between-battle healing, schema (HEAL1-L stage 1): save version 9 stores each hero's missing HP (`UnitState.hp_lost`, 0 = full; older saves load at full HP) and the last rest place entered (`explore.last_rest`, "" = the start location); world locations gain `rest` (default false), a place where the party rests on arrival. Rules (HEAL1-L stage 2, `HealRules` in `scripts/progression/heal_rules.gd`): HP carries over between game battles instead of a full heal; a win rallies every hero by 25% of max HP (downed heroes get up with that), fleeing leaves at least 1 HP and no rally; entering a rest place (`rest: true`) fully heals the roster and records it as `last_rest`; Make camp costs 3 supplies and works on world-map nodes only; a defeat wakes the party healed at the checkpoint, free at a rest place, else a forced rest for 3 supplies, else back at the last rest place (or the start location) with supplies kept; XP, gold, items, flags, quests and time are untouched. `GameEncounter` starts heroes at their stored HP and the battle screen writes it back; previews, the Dojo and the sims stay at full HP. The HP display, Make camp button and defeat line come with HEAL1-U. Tested in `heal_test_suite.gd`, `save_test_suite.gd` and `save_compat_test_suite.gd`.

Road data (TRAV1-A, seed `data/seeds/trav1a.gd`): the old road from Warden Keep to the Ruined Courtyard (time 2, a one-shot `ash_raiders` ambush before the courtyard is held), the high road to the Cinder Gate (time 4, closed until `ashlands.courtyard_held`), and the two-way south road from the Iron Reaches Slag Gate to Warden Keep (time 6, owned by Iron Reaches). Warden Keep (`signpost`) and the Cinder Gate (`wastes_road`) have exits to the world map.

### Reputation (REP1-L)

Flag conditions take an optional `op` (`==` `!=` `>=` `<=` `>` `<`, default `==`). Reputation is the int flag `rep.<world>.<faction>` in -100..100, changed by the new dialogue effect `add_flag {key, delta, min, max}` and by the world quest field `reputation` (`{faction: delta}`, delta -50..50 and not 0, at most 4 factions, applied once on completion; not overridable by variants). Faction keys are references, so a faction rename rewrites them. A rep key that names no faction of the world is a warning (`RepRules.key_warnings`). `RepRules` (`scripts/dialogue/rep_rules.gd`) also holds the bands and shop price factors used by commit 2. In play: a quest's `reputation` applies once on completion (clamped; a non-number rep flag refuses the turn-in), and the shopkeeper's faction band sets prices: hostile <= -50 (shop closed, no shop choice), unfriendly -49..-10 (buy 115% / sell 85%), neutral, friendly 10..49 (95% / 105%), honoured >= 50 (90% / 110%). The dialogue and shop panels show the speaker's "Faction: band", and choices gated on reputation show `[Faction band]`. World Forge **Validate all** now also lists soft warnings in yellow (`WorldCatalog.world_warnings`), including rep keys that name no faction. No save version bump. Tested in `reputation_test_suite.gd`.

Reputation data (REP1-A, seed `data/seeds/rep1a.gd`): Ember Wardens standing from the Ashlands quests (hold_the_courtyard +10, cinder_gate_watch +10 and Ashen Horde -10, warden_oath +15) and Forge Guild +10 from iron_for_embers; Hest offers spare stores once at standing 10 or more; the quest `ember_patrol` opens at Hest at standing 30 (+5, sets `ashlands.ruins_swept`). Effects in play arrive with the REP1-L runtime commit.

### Dialogue and shops schema (DLG1-L commit 1)

Worlds gain two kinds: `dialogues` (node trees with choices, conditions and effects; generated service choices for quest givers) and `shops` (stock, loot-table sales, buy and sell multipliers). Characters get a `dialogue` and `gone_flags` (flag conditions that remove them), the world manifest gets `start_gold` and custom `service_lines`, quests can reward `gold` and chests can hold gold. A lint warns when a quest giver can disappear without a quest variant that re-routes the quest. Base service lines live in `data/dialogue/services.json` (`ServiceCatalog`); rules in `DialogueRules` and `ShopRules`. Commit 2 adds the runtime: `DialogueRunner` (open a conversation, view it, pick choices that apply all their effects or none, confirm quest turn-ins, leave shops), shops with whole-number prices (buy rounds up, sell rounds down), limited stock and loot-table sales, party gold (from quests, chests, selling and dialogue; start gold per world) and save version 4 (party gold, shop stock) with a migration from version 3. Gone NPCs disappear from maps and conversations. Tested in `dialogue_test_suite.gd`.

Dialogue and shop panels (DLG1-L commit 3): in the debug region map, press E next to an NPC to talk. The dialogue panel shows the speaker, the text and the choices (locked ones greyed out), a Continue button, and a turn-in step where you pick who receives the reward and which loot to take. A shop choice swaps in the shop panel (gold, buy list with stock left, sell list); Close returns to the conversation without repeating the node's effects. A dialogue fight opens the same combat arena (BATTLE1-L). Tested in `dialogue_panel_checks.gd` (run by the dialogue suite). Follow-ups (DLG1-F): one purchase buys at most 99, gold that would pass the 9,999,999 cap from selling or a dialogue gift is reported in a note, and confirming a quest turn-in re-checks it first.

Dialogue bar and shop popup (UIR-6): dialogue opens as a full-width bar along the bottom of the region map (portrait from the character's unit art, name, line, numbered choices on the right; 1-9 pick, Esc or right click = Farewell when the node offers it, Cancel on a turn-in step, Continue on an end node). Locked choices show why, from `DialogueRules.lock_reason` ("Needs 1 Throwing Fire Bomb", "Needs 40 gold", "Needs KNOW 14", a quest or reputation band; a plain story flag reads "Locked", a holding `unless` "Not now"; a reason the `[stat]` / `[faction band]` prefix already shows is not repeated). The bar sits in its own canvas layer (1) and the camera keeps the party above it. Shops open as a popup over the dimmed map (canvas layer 31, like Settings) with Buy / Sell tabs, an item detail line (stats, owned count across stash and worn gear, who can use it) and "Nothing to sell" / "Nothing for sale"; Esc, right click and Close return to the conversation. Both follow Settings > Text size. Tested in `dialogue_bar_checks.gd` (run by the dialogue suite).

### Forge item tab (ITEM1-L commit 3)

The Content Forge shows items and item families (read-only since TOOLS-RO) with dedicated sections: identity with the rendered description, family and build chips, a base-values grid (base + adjustments = result, with band markers), modifier and tier tables (tiers for weapons, armor and shields alike), and a preview with the per-modifier gear budget. Key fields of system items (including the modifier and tier lists) can only change inside the system-edit scope (`ContentCatalog.key_field_errors`); a family change is stored with `upsert_many`, which re-resolves its items. Filter chips narrow the item list by family, hand, rarity, origin, usage, special, tiers and effects. Logic in `ItemFormModel`, widgets in `ItemTables`; tested in `item_form_model_test_suite.gd`.

### Asset pipeline (S14-L1)

Art keys now resolve through `AssetRegistry` (`scripts/art/`): a row in the manifest `data/assets/manifest.json` (managed by `AssetCatalog`) points a key and variant at a file under `assets/art/<namespace>/` or at another key (alias); otherwise the generated placeholder template is used, then a fallback, so a texture is never missing. Unit files are composed onto the team-coloured token. Item icons resolve the same way: an item's `icon_id` may name any item art key (bespoke, type or manifest; validated, and offered in the Item Forge icon picker), otherwise the icon follows its gear type. Skills and statuses get generated placeholder icons (`SkillArt`, `StatusArt`). A manifest row whose key still has a template can always be removed (the placeholder comes back); a manifest-only key cannot be removed while content uses it. `tools/fc.sh export_placeholder <ns> <key> <variant> [team]` writes a template as a PNG to start real art from. The first rows (S14-A1) alias `goblin_brawler` to the berserker art and override the warrior token and portrait, the runed staff icon and rubble tiles with PNG exports of their placeholders, so shipped art looks as before (rubble tiles now all use their first variant). Tested in `asset_pipeline_test_suite.gd`.

### Audio (S14-L3)

Sound plays by key, never by file path. `SoundBank` looks up an `sfx` key through `AssetRegistry` (a manifest row pointing at `assets/audio/sfx/<key>.wav` or `.ogg`); a key with no file is silent and warns once. `SoundEvents` lists the keys (hit, crit, mitigated, miss, heal, buff, debuff, defend, move_step, defeat and four UI keys) and which presenter preset plays which. One `AudioDirector` in the combat arena and one in the Scene Runner (`AudioDirector.active` is the newest one still in the tree, so the runner's takes over again when a nested arena closes) hold eight SFX voices and two UI voices, play the presenter's `fx_played` sounds and the playback buttons' click, and set the Master, Music, SFX and UI bus volumes (`default_bus_layout.tres`) from the new `SettingsFile` (`SettingsFile.load_file()`, `user://settings.cfg`, keys `audio/master|music|sfx|ui`, never part of a save). The `miss` key has no file yet, so a miss is silent until `assets/audio/sfx/miss.*` exists. Headless runs, tests and the sim create no voices. `tools/fc.sh gen_sfx <key> <wave> <hz_from> <hz_to> <ms> <envelope> <gain_db>` writes a placeholder blip; S14-A2 ships one generated blip per key (`assets/audio/sfx/*.wav`, seed `data/seeds/s14_a2.gd`), so real sounds later replace a file under the same key. Tested in `audio_test_suite.gd`. SOUND-PREVIEW: moving a volume slider in Settings > Sound puts the draft level on its bus at once and plays a short sample (Master `ui_confirm`, Music `ui_confirm` on the Music bus, Effects `hit` on the SFX bus, Interface `ui_click` on the UI bus), at most one per 0.15 s (`VolumePreview`); closing the popup without Apply puts the saved levels back. Headless runs record the samples (`AudioDirector.previews`) instead of playing. Tested in `sound_preview_test_suite.gd`.

Music (MUSIC1): `MusicDirector` plays one looping track on the Music bus, crossfading over 1 s and fading in over 2 s from silence; it lives under the tree root, so it survives scene changes. Scenes post requests by context (menu, field, travel, town, shop, danger, combat, encounter, boss) and the highest-priority visible request wins, so a battle plays over the map and keeps playing under the results popup. Each context finds its track through a data-only fallback chain (`<region>_<context>`, parent regions, world and lineage, then the plain region or world key for the field family, then `<context>` alone, then a parent context), so a new track needs only a manifest row. Ten generated loops ship (`menu_theme`, `ashlands`, `iron_reaches`, `tidewrack`, `combat`, plus MUSIC2 `town`, `danger`, `shop`, `travel`, `boss`; `python3 tools/gen_music.py`, seed `data/seeds/music1_a.gd`). Towns and fortresses play `town`, camps, ruins and dungeons `danger`, an open shop `shop`, the world map `travel`, a fight with a boss-tier unit `boss`. Spec `docs/specs/MUSIC1.md`.

Placeholder art (ART1): every Ashlands character (own unit key in its `art` field), class, item, skill, status and battle terrain has generated placeholder art, plus region map tiles, map markers, world-map pins, an Ashlands world map, location backdrops, menu art and four 48x30 Ashlands map templates (`python3 tools/gen_art.py [group ...]`, needs Pillow and numpy; seed `data/seeds/art1_a.gd`). Map, pin and backdrop art shows once the full-screen map view uses those keys. Spec `docs/specs/ART1.md`. Since MAPREAD1-A the four Ashlands locations run on the 48x30 `ashlands_*` templates and carry `map_pos` pins placed on the world-map art (hand-tuned to the drawing), and the beacon, wastes road, signpost and hearth markers have authored labels (the beacon also a note); seed `data/seeds/mapread1_a.gd`.

Placeholder art pass 2 (ART2): combat chips for the coming UI rows (13 enemy intent kinds as `ui/intent_<kind>`, `badge_foresight`, `chip_slay`, `chip_guest`), the `overlay_objective` ring tile, and the `miss` sound, which now plays on a missed attack (generator `tools/gen_art.py chips overlays`, seed `data/seeds/art2_a.gd`).

Story recruits (RECRUIT1-A): a new Ashlands game starts with Bryn Hale alone (`start_party` ["bryn_hale"], `party_max` 3). Brask Toller's muster at Warden Keep offers one recruit after the Courtyard is held and another after the oath: Sella Marsh (ranger), Kit Vane (rogue), Oren Dusk (mage), Alda Wren (priest) or Dagny Coal (brawler). Priest and brawler catalog party units exist; the early solo and duo fights are resized (`courtyard_skirmish` two raiders, Skarn and Vex, new `ash_straggler`, patrol after the pass is cleared). Seed `data/seeds/recruit1.gd`; placeholder art for the seven new characters from `tools/gen_art.py characters`.

Battle objectives in Ashlands (OBJ1-A): the Courtyard is a hold (keep the centre ring 3 rounds, or rout them); the new "Raid on the Steel Wagon" (`wagon_raid`, the south road scripted fight, replacing `ash_raiders` there) is an ambush with Wenn Tallow, Orsa's Carter, as a protected guest; both Ashmaw stands are slay Grakka. Seed `data/seeds/obj1.gd`; Wenn's placeholder art from `tools/gen_art.py characters`. Objectives take effect once the OBJ1-L runtime is on `develop`.

### Combat presenter (S14-L2)

All combat animation now goes through `CombatPresenter` (`scripts/ui/combat/`), which `CombatGridPanel` feeds from `CombatSession.fx_requested` and from its refresh slide. Timings and tween recipes live in one table (`FxPresets`; the durations are the old hard-coded ones) and `FxPresets.for_entry` maps a log entry to its presets: melee bump or ranged pulse, hit or mitigated shake and flash, a miss (log tag `MISS`) as a short sidestep with grey "Miss" text and no shake or flash (REVIEW-F3 Q3), heal, buff and defend pulses, a status icon pop when the entry names a real status, floating text (the old labels) and a defeat fade. `FxSettings` holds the speed (one of four fixed levels 0.5, 1, 1.5, 2; any other value snaps to the nearest, loaded from the UI1-L settings file at startup and on Apply) and instant mode, which applies every end state at once and is always on in headless runs and tests, whatever the settings file says. A preset's distance is measured on screen only when the entry has a target or is a move; otherwise it comes from the log coordinates. The presenter owns every tween and temporary node and ends them all (tokens at their end states) when the log resets, the encounter remounts, the timeline steps back or too many pile up; `fx_played` fires once per preset that actually ran (a preset skipped for a missing actor or target plays no sound) and drives the S14-L3 sounds. Nothing in resolution, the log or the sim changed. Tested in `animation_lifecycle_test_suite.gd`.

### Forge class tab (CLASS1-L)

A class's `skill_path` is now a map of skill family to unlock level (`{family: level}`, kept in entry order; `ClassArchetype.path_steps()` sorts by level then position, and the default loadout is the first five). CLASS1-M migrated the shipped classes to the map shape (and gave every unit `heal_below` 0.7), so the old list converter is switched off (`ClassUnitFields.LIST_PATH_CONVERTER` false; a list path is now a validation error). Key fields of system classes (id, AI role, base stats, growth, proficiency) only change inside the system-edit scope; gear, pouch and path are free fields. Units gain an AI `heal_below` threshold (default 0.7). The Forge shows classes (read-only since TOOLS-RO) in five sections with a path table, proficiency rows, gear pickers grouped by item family and live stat, gear-budget and coverage panels (`ClassForgeModel`, `ForgePreview.class_panels`); soft warnings never block saving. Tested in `class_path` and `class_form_model` suites.

### Main menu, new game and save slots (UI1-L)

`scenes/main_menu.tscn` (Scene Runner > Main Menu) is the player's front door: **Continue** loads the newest save (autosave included; hidden when there is none), **New game** lists the playable worlds with their fixed starting party (world manifest `start_party`, inherited like `start_location`; a world is playable when both resolve) and starts with an autosave (asks first when a readable autosave would be replaced), **Load game** shows the autosave and five manual slots (one shared list across worlds; each row names its world) with a header read by `SaveGame.peek` (world, location, party levels, time; corrupt files show as unreadable, newer versions are listed but not loadable; the rows scroll and a Back button stays below the list, as on the Save game screen), **Settings** edits `user://settings.cfg` (`SettingsFile`: four volume sliders, animation speed (Slow 0.5x / Normal 1x / Fast 1.5x / Very fast 2x) and text size (Small 0.85x / Normal 1x / Large 1.15x / Extra large 1.3x) as four-notch sliders (MENU-POLISH: the level name shows under the slider; the text size previews live and is restored by closing without Apply; a Skip animations checkbox sits on the speed row and greys the speed slider while ticked; SETTINGS-POPUP: Settings opens as a centred popup panel (up to 900x620, shrinking to fit the viewport with a margin) over a dimmed backdrop, from the main menu and from the hub, with the screen below still drawn behind the dim; its header has the title on the left and a close cross on the right (cross, Esc and right click all take the same cancel path, so the unapplied-changes question still applies; a click on the backdrop does nothing), there is no Back button, the rows sit in three tabs (a `TabContainer`, the panel keeps one size on every tab, each tab scrolls on its own): Sound (four volumes), Display (window mode, window size, In use, vertical sync, frame rate cap, text size; on web Fullscreen and text size only) and Gameplay (animation speed with Skip animations); Restore defaults (a flat text button; its tooltip says it resets every tab; the draft becomes the defaults, display keys included) and Apply sit small at the bottom right; a window mode or size applies live as soon as it is picked and opens the Keep / Revert countdown at once, so Apply is for the other keys; there is no sample label, the whole menu is the text size preview; right click is Back on every menu screen, like Esc, but not over the map or a fight, and is taken in `MainMenuView._input` before the GUI, because panels and buttons swallow it otherwise), and (DISPLAY1) a Display block on desktop: Window mode (Windowed / Borderless fullscreen / Fullscreen), Window size (960x540 to 2560x1440 logical sizes, only those that fit the screen, scaled by the screen scale on high-DPI displays; an "In use" row reads the real mode and size back from the window; disabled with "Used in windowed mode" in a fullscreen mode, the saved size is kept), Vertical sync and Frame rate cap (30 / 60 / 120 / 144 / Unlimited), keys `display/window_mode|resolution|vsync|max_fps`; vsync and the cap apply on Apply, but a changed mode or size is applied live and a "Keep these display settings? Reverting in 10 s" box (Keep / Revert, Revert focused) writes the two keys only on Keep, while Revert, Esc and the timeout restore the old ones and write nothing; the saved settings apply once at boot; the web build shows only a Fullscreen checkbox (not remembered, follows the browser's own exit) and writes no display key; headless runs use a recording backend and never touch a window; `display_test_suite.gd` covers it; a saved value off the list snaps to the nearest level on load, a tie to the lower, with no file version change; never part of a save; the block is centred in a label / control / value grid, and `MenuStyle` gives every menu-screen button a border with hover, pressed and focus states and the checkbox a visible tick box; `settings_ui_test_suite.gd` checks the level lists, the snapping and that the main menu, Load, Save and Settings screens fit 1280x720 at every text size and speed), and **Tools** (debug builds) returns to the Scene Runner. A started game opens the region map. CONTROLS-HELP: Settings has a fourth tab, **Controls**, a read-only list grouped Menus / Maps / Combat / General (one row per action: its name, then its keys and mouse buttons); the InputMap actions are read live and the keys the scenes handle themselves come from one table, `ControlsList` (`scripts/ui/menus/controls_list.gd`), whose rows name the handling script and the tokens that must appear in it (`controls_help_test_suite.gd` scans the sources, so the help cannot drift); rebinding is not supported. During play Settings opens over any game scene (region map, world map, town or dialogue, fight) with Esc, when nothing else took it (an open log, timeline or action popup closes first, so the next Esc opens Settings), or with the **☰ Menu** button at the top right; the game underneath is paused (`process_mode` disabled; a mounted fight also gets `keys_blocked`, the results popup's flag) until the cross, Esc or right click closes it, and closing it returns to the game menu below (UIR-2). Game menu (UIR-2, the hub as a popup, Q1 B): during play Esc or ☰ Menu opens the game menu over the paused, dimmed map: the world and location, one tile per party member (level, HP), then Party (with points to spend), Journal, Save game, Load game, Settings and Quit to main menu, with a line saying what the focused entry does or why it is off; Return to map closes it. Party, journal and save/load screens open in the same popup. Saving and party changes wait while a battle, conversation or shop is open. New game: pick a world card, then Start. Save/load rows have a fixed height, and overwrite, delete and load-over-unsaved questions name what is lost and focus No. The main menu shows a muted version label at the bottom right. Tested in `game_menu_test_suite.gd`. Sound tab (VOLUME-FIX, MUTE): a **Mute** checkbox sits first (key `audio/mute`, default off) and mutes the Master bus without changing the levels; it applies as a draft like the sliders (restored on close without Apply) and slider samples stay silent while muted. Volumes now reach the global buses through `AudioDirector.apply_saved()` on Apply, at startup and when a director comes up, with or without a director in the tree (before, they applied only when one existed, so a lowered Master could do nothing). Tested in `audio_settings_test_suite.gd`. The Controls tab ends with a **Testing** group (arena edit mode F9, Scene Runner F5/F6/F10). The settings popup lives in its own canvas layer (31, above the results popup at 30) so it shows over a mounted arena. The **hub** is the game menu popup described above (UIR-2; Party opens the UI2-L party screens, Journal the UI3-L quest journal, both inside the popup). Overwriting or deleting a save, leaving a screen with unapplied changes and quitting with unsaved progress all ask first; saving is refused during a battle, conversation or shop (the region map reports it through `CurrentGame.set_busy`). Slots: 0 = autosave, 1-5 manual (`SaveGame.write` refuses anything else); no save version change. Every screen is a headless model (`MenuRouter`, `MainMenuModel`, `NewGameModel`, `SaveSlotsModel`, `SettingsModel`, `HubModel` in `scripts/ui/menus/`) over `CurrentGame` (the game in play); views only render. The Ashlands (and Tidewrack, which inherits it) start with warrior, ranger, rogue and mage (UI1-A); Iron Reaches is not playable yet. Each readable save can be **exported** to a JSON file anywhere (a file dialog; default name like `ashlands_slot2_20261017-1445.json`), and **Import** reads such a file from any world into a manual slot (migrated and validated like a load; it keeps its own save time so the autosave stays the resume point; asks before replacing a slot; not offered in the web build) (SAVE-IO, `SaveGame.export_to` / `import_from`, tested in `save_io_test_suite.gd`). Writes go to a free `<path>.fc_tmp` name and are checked; an export over an existing file keeps it as a backup until the new one is in place, so a failed move never loses it. Import refuses files over 8 MiB and clamps a future save time to now; a file with a missing, zero or negative version reads as unreadable, so Continue skips it (REVIEW-F1). Tested in `menus_test_suite.gd`. UIR-1 (shared menu pieces): `ScreenFrame` (centred title, scrolling column up to 1000 px, action bar pinned at the bottom with Back first and the main action last), button width classes (wide 260, row 140, step 48; text centred, never fit-to-text among peers), a gold-edged selected style instead of the "▶ " prefix, and destructive questions (overwrite, import over a slot, delete, quit without saving) focus No (`MenuRouter.ask(..., destructive)`).

### Map readability data (MAPREAD1-L1)

Map markers gain an optional `label` (at most 32 characters) and, for points only, a `note` (at most 160). `ExploreRunner.marker_view(gs, location, marker)` reads a marker out for the map views as `{kind, state, label, verb, badge, icon}`: state `present`, `locked`, `spent` (an opened chest, a point whose flags all hold) or `absent`; badge `quest` (the NPC gives an available quest) or `ready` (a quest turns in to them); icon `overlay_<kind>` (`overlay_poi` for points). Locations gain `map_pos` (`[]` = automatic layout, or `[x, y]` between 0 and 1) for placing world-map pins on a backdrop. `TravelRules.view` nodes now carry `kind`, `region`, `quest` (an active quest's current step goes there; `TravelRules.quest_nodes`) and `map_pos`. Read-only; the icons, labels, legend and pins arrive with MAPREAD1-L2. Tested in `exploration_test_suite.gd`.

### Story recruits schema (RECRUIT1-L commit 1)

A world manifest's `start_party` may now list world characters with role `party` as well as party-side catalog units (an id that is both is an error), and a new manifest field `party_max` caps the party (-1 inherits the nearest parent's value, otherwise 1..4; a resolved start party longer than the cap is an error; `WorldCatalog.party_max`). A party character may carry passives but not named items, disciplines or tuning yet (`WorldCatalog.party_character_errors`). Dialogue choices gain the effect `recruit` {character} (choices only, at most one per choice, the character must be a valid party hero). The runtime (recruiting, hero names, the party cap in play) arrives with commit 2. The World Forge `start_party` picker still offers catalog units only. Tested in `dialogue_test_suite.gd` and `menus_test_suite.gd`.

### Story recruits schema (RECRUIT1-L commit 1)

A world manifest's `start_party` may now list world characters with role `party` as well as party-side catalog units (an id that is both is an error), and a new manifest field `party_max` caps the party (-1 inherits the nearest parent's value, otherwise 1..4; a resolved start party longer than the cap is an error; `WorldCatalog.party_max`). A party character may carry passives but not named items, disciplines or tuning yet (`WorldCatalog.party_character_errors`). Dialogue choices gain the effect `recruit` {character} (choices only, at most one per choice, the character must be a valid party hero). Commit 2 adds the runtime: a world character in `start_party` becomes a hero with its own name (party, results and new-game screens, and in battle), a `recruit` choice adds the character at the level of the strongest active hero (refused when already a member or the party is full; a choice for a member is hidden), the active cap follows the world's `party_max` (Party screen "Active n / m"), and old four-hero saves in a three-hero world load with three active and one in reserve (no save version change). Tested in `recruit_test_suite.gd`. The World Forge `start_party` picker still offers catalog units only. Tested in `dialogue_test_suite.gd` and `menus_test_suite.gd`.

### Party screen, level-up and battle results (UI2-L)

Character popup (UIR-3): the hub's **Party** entry now opens one Character popup (`scripts/ui/menus/character_view.gd`): a member strip (Q/E or shoulder buttons switch member), tabs Stats / Level up / Skills / Gear, an X (Esc and right click) instead of Back, a fixed detail pane for the focused row, and one "Discard your changes?" question on close, tab or member switch while a draft is open. Level up shows Stat / Now / Added / New with Auto and Revert beside the rows and Confirm in the action bar; Skills shows each family's active rank only, with its description; Gear keeps the battle pouch and the Gear board as an optional view. Battle results still open the level-up and skills bodies standalone. In play, questions and the Gear board show above the popup (canvas layers: popup 31, Gear board 32, question 33). The text below describes the older separate screens; `PartyModel` now owns the tabs, drafts and the discard rule, and `LoadoutModel` family rows carry `active_tier`, `description` and `next_unlock`.

The hub's **Party** entry (a dot marks members with unspent level-up points) opens the roster: toggle who is in the active four (at most 4, at least 1), and see the selected member's stats split into base, class growth and spent points, the derived block (max HP, speed, tiles per move, crit, melee and ranged power, armor, debuff resist, AP), XP to the next level, passives and gold. From there **Level up** drafts points with + / - or Auto (the class weights) and shows every derived value before and after; Confirm asks "Spend these points? This is permanent." first and unspent points may wait (battles auto-spend them on a copy). **Skills** slots up to 5 class and 3 extra families (basics are always active; locked tiers show their unlock level; Use default returns to the class pick). **Gear** equips stash items per slot (the validator's proficiency, hand and requirement messages verbatim; an explicit item goes back to the stash, the slot reverts to the class default) or opens the drag-and-drop gear board on the same draft (`PartyGearBridge` over `CharacterEquipmentModal`). Every edit is a draft that Cancel or Back drops (Back asks first). After a fight on the region map the **battle results** screen lists XP and levels of the active members, new skill tiers (with Open loadout when a slot is free) and quests advanced, offers Level up per member (skippable); ✕ autosaves and returns to the map (Esc and right click do the same; BATTLE-FLOW made it a popup over the arena). Models: `PartyModel`, `StatsModel`, `LevelUpModel`, `LoadoutModel`, `GearModel`, `BattleResultsModel`, `PartyPreview` (derived block via `PartyBuilder.build(state, false)`, which leaves unspent points unspent). Tested in `party_screen_test_suite.gd`.

Battle report data (BREPORT1-L): after every battle (won, lost or fled) the results data carries a per-hero battle report (damage, taken, healed, effects, combos, best moment), counted live by `BattleMetrics` (the same collector the sim uses, with round checkpoints so a rewound round is not double counted) and built by `BattleReport`; `tools/sim/sim_observer.gd` stays as an alias that extends `BattleObserver`, so suites that extend it by path keep working. `BattleResultsModel` exposes `has_report`, `report_columns`, `report_rows` and `best_moment`. Each applied status is credited to the unit that applied it (STATCRED1-L; boss-phase grants to their enemy recipient). The popup view lands with BREPORT1-U. Tested in `battle_report_test_suite.gd`.

Enemy morale schema (MORALE1-L, schema slice): encounters gain `morale` (default false), `morale_break` (0..90, default 50), `morale_leader` (an id in `enemies`; empty = first boss/elite), `morale_groups` (enemy index lists covering every enemy once; [] = one group) and `morale_text` (at most 80 characters). Both catalogs validate them with `EncounterMorale.problems`; the World Forge preview shows a `Morale:` line. In battle (MORALE1-L): enemy groups lose morale on kills, crits and the leader's fall; Shaken and Wavering groups flee sooner, and a Broken group surrenders at once (a break win counts as a win). Bosses, elites, reckless and never-flee units and slay targets are Fearless. Intents carry `morale_state` and `morale_group`. Tested in `morale_test_suite.gd`.

Encounter purse schema (PURSE1-L, schema slice): encounters (catalog and world) gain `purse` (default `{}` = no purse): `gold_min` (0..500, default 0), `gold_max` (`gold_min`..500, default `gold_min`), `loot` (a loot id of the encounter's world or a qualified `world:id`, item rewards only; catalog encounters take only qualified ids) and `once` (default true). Unknown keys are errors; gold above 100 and an empty purse are warnings. Validated by `PurseRules.problems` / `warnings` (`scripts/exploration/purse_rules.gd`). Tested in `purse_test_suite.gd`.

Encounter purse in play (PURSE1-L stage 2): a won fight (a morale break counts) pays its purse after XP and the quest hook (`BattleBridge.apply` -> `PurseRules.pay`); lost and fled pay and record nothing. The gold is `randi_range(gold_min, gold_max)` on its own generator seeded from the encounter's qualified id, the world clock and its earlier wins, so a reload never rerolls and combat replays are untouched; it goes through `PartyState.add_gold` (cap note). `once: true` purses pay only the first win, `once: false` every win; the loot entry is claimed and stashed on the first paying win (skipped when already claimed or its `requires_flags` fail). Save version 10 stores `GameState.purses` {encounter qid: wins} (older saves migrate to `{}`). Map, road and dialogue fight results carry top-level `gold` / `loot`, shown as "Gold +N" and "Loot: ..." on the results popup.

### Quest journal (UI3-L)

The hub's **Journal** lists the current world's quests the party knows about, grouped as Ready to turn in, Active, On hold (a started quest a story change made unavailable) and Completed; quests never started stay hidden. The selected quest shows its resolved text (variants applied, so an NPC's death changes the giver or puts the quest on hold on the next open), who gave it and who takes it back, progress, the current objective only (no later steps), rewards, the loot it still offers with how many picks, and the linked quests the journal already shows. Arrow keys move the selection. It is read only: `JournalModel` (`scripts/ui/menus/`) re-reads `QuestRunner` on every open and never changes the game; unknown ids show in brackets. Tested in `journal_test_suite.gd`.

### Agent workflow tools

`CLAUDE.md` indexes `docs/` (game brief, roadmap, structure incl. placeholder folders, decisions, history of deprecated ideas, balance report, workflow) and defines the agent roles and parallel lanes (design-brainstorm, logic-builder, asset-builder, balance, change-reviewer; the main session plans via the `planner` skill, queue in `docs/TASKS.md`, specs in `docs/specs/`). `tools/fc.sh` gives compact checks (`parse`, `parse-all`, `arena`, `dojo`, `run`, `sim`, `safe_delete`, `all`, and `gate`, the commit gate without PowerShell); `check_project.ps1 -Fast -Quiet` runs the full check in ~30 s (see docs/EFFICIENCY.md), `tools/fc.sh docs` includes a markdown lint subset (`tools/md_lint.awk`), `tools/wt_remove.ps1` removes a finished worktree on OneDrive (clears the read-only folder flag git trips on) and `tools/balance_sim.gd` (`fc.sh sim duel|tier|levels`) runs headless AI-vs-AI balance tables. Claude Code skills (`godot-debug-checklist`, `patch-and-seed`) and agents (`design-brainstorm`, `logic-builder`, `asset-builder`, `balance`, `change-reviewer`, `housekeeper` for cleanup proposals) live in `.claude/`. The workflow runs on a desktop machine or in Claude cloud threads; `docs/workflow/` holds the steps that differ (sessions, branches, merges, gate time) and the checklist for switching between them. In the cloud each area thread runs the six lane agents and `change-reviewer` reviews every task before it lands on `develop` (a `Review:` line in its squash commit); a Merge & Release thread batches `develop` into `main` with one PR. `.github/workflows/gate.yml` runs the fast gate and the docs lint on that PR (a required check on `main`) and on pushes to `main`.

### Automated Check Script (`check_project.ps1`)

Runs in 4 phases:

- `[1/4]` Verifies global class cache and asset imports (`--headless --import`).
- `[2/4]` Validates GDScript syntax & static typing across every project script (`--headless --check-only`); a crash only while Godot shuts down after a clean parse is reported as a warning.
- `[3/4]` Executes the headless arena test harness (`--headless scenes/test_arena.tscn`), asserting exit code 0. (The Scene Runner's suite buttons run the same scenes.)
- `[4/4]` Executes the headless gear dojo suite (`--headless scenes/gear_dojo.tscn`), asserting exit code 0.

Godot is found through the `GODOT` environment variable, then `godot` or `godot4` on PATH, then `C:\Program Files\Godot\godot.exe`. The script runs on Windows PowerShell 5.1 and on PowerShell 7 (`pwsh`) on Linux or macOS. `tools/fc.sh gate` runs the same steps as `-Fast -Quiet` in bash, so a Linux machine or cloud container only needs a Godot 4.7 binary.

### Playtest recordings and replay (PLAYTEST1-L)

In debug builds, battles can be recorded as versioned JSON in `user://playtests/<id>.json`. A recording holds the encounter, the seed, a content fingerprint, the confirmed plans and RNG state per round, the outcome per round, a UI trail (scenes, panels, modals, buttons, keys, clicks, hovers, plan edits, engine errors, asset fallbacks) and F8 notes. `tools/fc.sh replay <file|id>` replays a recording headlessly (exit 0 match, 1 divergence or broken plan, 2 invalid) and supports `--variant`/`--then`/`--write`, `--seeds`/`--fallback` and `--trail-report` (dead clicks, repeat clicks, long hovers, refused confirms, errors, undo churn, UI open/close problems, leaks, asset fallbacks). `CombatSession` emits `round_started`, `round_confirmed`, `plan_refused`, `round_concluded`, `round_reset` and `plan_edited`; `AssetRegistry.resolve_observer` is a playtest-only hook. The folder layout and the one-scenario rule are in `playtests/README.md`. Tested in `playtest_test_suite.gd`.

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

On Linux or macOS (bash only, or `pwsh ./check_project.ps1 -Fast -Quiet` if PowerShell 7 is installed):

```bash
GODOT=/path/to/Godot_v4.7-stable_linux.x86_64 tools/fc.sh gate
```

**Per-system gates** (user 2026-10-18): the arena suites are tagged with 8 systems (`SuiteRegistry.SYSTEMS`: rules, ai, combat_ui, balance, world, party_save, content, menus). `tools/fc.sh gate auto` runs the gate with only the systems the change touches (`tools/fc.sh systems` shows the pick; a path it cannot place means every system, and a docs, CI or tool-only change skips the arena), `tools/fc.sh gate rules,ai` names them, and plain `tools/fc.sh gate` runs everything. CI runs every system as its own parallel job on the release PR, so a red job names the broken system; the `gate` check passes only when all of them pass. A new suite gets one `if _on("<system>"): results.append(...)` line in `scripts/tests/suite_registry.gd`.

### Run the Game (Scene Runner)

One launcher for everything:

```cmd
run.bat
run_dev.bat
run.bat opt a
```

`run.bat` fetches and fast-forwards `main` first (about 1-2 s when nothing changed), rebuilds the script cache only when the code changed (about a minute), shows the branch and commit, and launches; `run_dev.bat` (or `run.bat dev`) does the same on `develop`, where finished tasks land before each release, from a sibling folder `<checkout>-dev` (a git worktree created on first use, about 15 MB), so switching between the two never waits for a cache rebuild. When a pull changed the code, it first copies that game's saves to `user://save_backups/<date>-<old commit>` (newest 10 kept), so a change that breaks a save can be undone by copying a backup back into `user://saves`. The dev game keeps its own saves and settings (`run.bat` points `%APPDATA%` at `%APPDATA%\tactical-rpg-dev` for it), so dev saves never mix with main's. Offline, a failed pull, local changes or a folder on another branch: it launches the copy on disk and changes nothing. `run.bat opt <a|b|c|branch>` (or `run_opt.bat`) opens a design-review option build: `a`/`b`/`c` is the newest remote branch `claude/design-opt-<date>-<letter>`, anything else a branch name; it runs in the sibling worktree `<checkout>-opt` (detached, switched to the branch tip after a fetch unless that folder has local changes) with its own saves and settings under `%APPDATA%\tactical-rpg-opt`; Restart runner inside that folder reopens the same option on its newest commit.

Or open the project in **Godot 4.7+** and press Play: the main scene is the Scene Runner. From its menu:

- **Combat Arena** (pick any catalog encounter or any world's battle), **Content Forge**, **World Forge**, **Gear & Skill Dojo**, plus any other scene added under `scenes/` (listed automatically).
- In a running scene, the **☰ Menu** pill (bottom-right) or **F10** returns to the menu, **F6** restarts the scene, **F5** reloads edited scripts and restarts it.
- **⟳ Reload scripts** recompiles scripts edited on disk (the menu shows how many are pending); brand-new files or new `class_name`s need **⏻ Restart runner**, which starts a fresh process. On Windows in a git checkout, **⏻ Restart runner** instead runs `run.bat` (fetch and fast-forward this folder's branch, back up saves, relaunch) and reports `Pulled <old> → <new>` on the next start.
- **▶ Arena suite / ▶ Dojo suite** run the headless test scenes in a separate process and show their PASS / FAIL lines.

The Content Forge and World Forge are read-only viewers; content changes go through `ContentCatalog.upsert` / WorldCatalog seed scripts (`docs/workflow/content-edits.md`). To run a catalog or world encounter directly:

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
- [x] Animation lifecycle tests (Tween callbacks, duration timing, and queue free verification): `animation_lifecycle_test_suite.gd` (S14-L2).

### 4. Advanced Combat Rules & Combo Engine (`System 6`)

*Status: Terrain (S12T) and combos (S12C-L engine, S12C-A first combo skills) built.*

- [x] **Combos (S12C-L)**: skill fields set on the family (ranks may not set them; Forge sections Flow move and Combo; tokens `{flow}`, `{exploit}`, `{disrupt}`). **Flow**: `ends_in_stance` (ATTACK, DEFEND, FOCUS, SUPPORT) - the skill fights in its own stance, then switches to the end stance for free after its last block (`[Flow]` log; the next action's switch cost reads the end stance; a command that could not act does not flow). **Stance break**: `disrupt: STANCE_BREAK` - a damaging hit the target survives turns an Offensive or Defensive target Tactical and ends its running guard (`[Break]`). **Exploit**: up to 2 `{status, power_mult 1.05-2.0}` rows - the hit's power uses the highest multiplier among the target's active listed debuffs (`(exploits X)`), and `exploit_consume` removes that status after the hit. Per-hit order: compute, damage, status roll, riposte, break, consume. Plan options preview flow and exploit. The AI orders a break skill first against a Defensive or riposting (active or planned) foe and an exploit skill first only when its status is active; target ranking is unchanged (user decision). Tested in `combo_rules_test_suite.gd`.
- [x] **Extra grants (BAL5-L)**: skill field `extra_grants` `[{status, rounds 1-3, magnitude}]`, at most 1 entry, DEFEND or SUPPORT only; a registry buff other than `grant_status` (SUPPORT: support buffs; a taunt extra needs target SELF), magnitude inside the status range. Granted with the skill's own grant at the same timing (a defend buff such as riposte at once, every other buff next round; SUPPORT on the cast target). Ranks may override the whole list; token `{extra}` renders each rank's list ("Fortified 6 armor for 2 rounds"). Tested in `extra_grants_test_suite.gd`.
- [x] **First combo skills (S12C-A, placeholder numbers)**: Feint (rogue, level 6, dagger, ends in Defensive), Opportunist (rogue, level 7, dagger, x1.3 vs Exposed and x1.2 vs Slowed, not consumed) and Crushing Blow (brawler, level 6, unarmed/fist, breaks stance). Opportunist pairs with the warrior's Rend Armor and the ranger's Crippling Shot. No default loadout changes. Seed `data/seeds/s12c.gd`.
- [ ] **Arena weapon commands**: `SwitchWeaponCommand` (1 block, toggles `Inventory.active_weapon_slot`) and a `DualStrikeCommand` combo for 1H pairs, so the dojo weapon rules apply on the round timeline.
- [ ] **Stance Disruption & Stagger**: Guard breaks that inflict the `Staggered` condition (unable to defend/dodge for $N$ blocks).
- [ ] **Area of Effect (AoE)**: Line, cone, and radial blast patterns on the 12x9 grid.
- [x] **Terrain rules (S12T-L)**: entering a tile costs its terrain `move_cost` (an exit tile costs 1, since stepping on it leaves the arena; cheapest route; a move block holds `move_per_block()` cost units and always takes its first tile, a move fits the round in at most 2 blocks; a dash is a `dash_tiles` cost budget; disengage ignores cost). Ranged hits get cover: the larger of the target tile's `cover_guard` and a blocking neighbour's toward the attacker, added to guard under `GUARD_CAP` 0.9 (`CoverRules`); melee and riposte never do. Hazard tiles (burning ground `f`, toxic bog `p`, ice `i` in lava_cavern, swamp_bog and frozen_pass) roll their status once per status per walk, dash or disengage landing and/or at round end after the status ticks (`HazardRules`, source `terrain`). Plan options show `(cover -N%)` and `⚠ Hazard: <terrain> (<status>)`. AI drafts add `AI_HAZARD_PENALTY` (3, placeholder) route cost per hazard tile and ranged AI prefers covered cells among equal ones; player plans use true costs. Numbers come from catalog kind `terrain` (one system row per type: `move_cost`, `cover_guard`, `hazard_status` / `potency` / `magnitude` / `rounds` / `on`); S12T-A ships all 17 rows in `data/catalog/terrain.json` (seed `data/seeds/s12t.gd`): wall cover 0.3, tree and rubble 0.15 (rubble cost 2), burning ground burn 5, toxic bog poison (cost 2), ice slows on enter; a missing row falls back to code defaults (old move costs, no cover or hazards). Tested in `terrain_rules_test_suite.gd`.
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
- [x] TOOLS-RO: the Content Forge and World Forge are read-only viewers (edit, save, clone, delete, promote and lock UI removed); their save-time checks run in the test suites over all shipped data.
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

Combat HUD layout A (COMBAT-UI3, user 2026-10-18; layout a UI preference, the continuous colour-coded blocks and the confirm warning his decisions): a slim planning strip on top shows the selected hero's plan as continuous blocks coloured by action kind (hover a block for how it plays out) next to the Party / Hero tactic pickers, Autoplay, Clear and Confirm round. Confirm round confirms at once; when a living hero has empty blocks a small question asks first ("Ranger has empty actions. Confirm anyway?", Back / Confirm, Confirm focused; Enter confirms, Esc goes back), unless Settings > Gameplay > **Warn about missing actions** (`ui/warn_missing_actions`, default on) is off; autoplay never asks. Left: compact party cards (name, HP bar, HP and AP) and a selected-hero panel (state, speed, reach, AP left, gear, plan, empty blocks). Right: hostile cards (name, HP, Next) and one merged enemy card for the hovered or picked enemy (state, HP / Speed / Reach, Next, `Your <hero>: deals about X · takes about Y`, Can use); the old forecast dock and floating hover card are gone (HOVER1-U absorbed). Bottom: "Round n · Planning" with a hint and the log ticker; click it, Expand or L opens the full log. The grid takes the freed height. Tested in `combat_ui_framework_test_suite.gd` (no overlap at 1280x720 at every text size, and 1920x1080) and `settings_ui_test_suite.gd`.

COMBAT-UI3 follow-up (user 2026-10-18 17:00 and 17:05): the strip's Party and Hero tactic pickers fill any blocks a draft leaves empty with a level-1 strike or a Rest (`TacticalAI.fill_steps`; sims, enemy AI and autoplay's own rounds are unchanged). Hostile cards show HP right-aligned on the name row. Strip block labels use a smaller font (80% of the HUD font) and fall back to the action kind's symbol when a name does not fit its block; the full name and description stay on hover.
