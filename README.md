# Tactical RPG (Godot 4)

A modular, simultaneous-execution tactical RPG engine built in **Godot 4** (GDScript). Combat resolves across a synchronous **10-block round timeline** on a **9x9 spatial grid**, featuring rock-paper-scissors **stance dynamics**, tactical targeting, perimeter escape/withdrawal, and deterministic state scrubbing.

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
- **Simultaneous 10-Block Rounds**: Combat does not use traditional turn-based initiative. Instead, all units plan actions along a 10-block timeline that resolves synchronously with multi-block durations, interruptions, and stance interactions.
- **Dynamic Stance Matchups**: 6 core combat stances (`Neutral`, `Offensive`, `Defensive`, `Tactical`, `Utility`, `Mobile`) govern damage dealt and received through asymmetric multipliers.
- **Spatial Positioning & Withdrawal**: 9x9 grid maneuvering with center-ring deployment, range checks, and safe boundary withdrawal.

---

## Codebase Architecture & Directory Breakdown

```text
tactical-rpg/
├── assets/                       # Static assets (icon.svg only; all art is generated in code, see scripts/art/)
├── data/                         # [Placeholder] Game data definitions & balance tables
├── scenes/
│   ├── combat_arena.tscn         # Main scene: combat UI foundation (CombatSession + CombatDockHost)
│   ├── combat_arena.gd           # Builds the session for an encounter and mounts the dock host
│   ├── test_arena.tscn           # Test harness scene: runs ArenaTestSuite, then launches combat_arena
│   ├── test_arena.gd             # Harness: headless exit code for check_project.ps1
│   ├── gear_dojo.tscn            # Dedicated Gear, Skill & Damage Formula Benchmark scene
│   ├── gear_dojo.gd              # Thin bootstrap: builds DojoSession, mounts the window workspace or runs assertions
│   └── dojo/
│       └── dojo_assertions.gd        # 36 headless dojo assertions (rules, session actions, filters, carry, UI smoke tests)
├── scripts/
│   ├── core/                     # Foundational rules engine primitives & base models
│   │   ├── combat/               # Core unit, stance & command models
│   │   │   ├── battle_command.gd         # Abstract base command class (duration, cost, initiative)
│   │   │   ├── combatant.gd              # Core unit model (HP, SP, speed, range, stance, buffs, target)
│   │   │   ├── combat_types.gd           # Stances, enum helpers, matchup multipliers & formatting
│   │   │   └── damage_calculator.gd      # Authoritative damage formula (shared by arena, dojo & future AI)
│   │   ├── utils/                # Shared helpers
│   │   │   └── asset_utils.gd            # Runtime SVG texture loading
│   │   ├── specs/                # Character specs, 6-stat framework & progression models
│   │   │   ├── character_stats.gd        # 6 stats (STR, DEX, INT | CONS, PERC, KNOW), leveling (1-10), derived stats
│   │   │   ├── class_archetype.gd        # Base stats, guaranteed growths & innate weapon proficiencies
│   │   │   ├── mastery_system.gd         # Horizontal progression (weapon proficiencies, lore, faction scores)
│   │   │   └── stat_modifier.gd          # Composable flat/percent modifiers with contextual gating
│   │   ├── inventory/            # Gear, equipment requirements & inventory containers
│   │   │   ├── item.gd                   # Base item model (type, rarity, weight, value)
│   │   │   ├── equipment.gd              # Gear model with stat requirements, threshold tiers & timeline penalties
│   │   │   ├── consumable.gd             # Consumable items used in combat (ItemCommand) or exploration
│   │   │   ├── inventory.gd              # Party stash, equipped gear loadout & 3-slot combat battle pouches
│   │   │   ├── item_catalog.gd           # Presets catalog: Claymore, Warhammer, Bow, Shields, Armor, Draughts, Bombs
│   │   │   ├── party_carry.gd            # Party-wide carry capacity, load tiers & overload speed penalty
│   │   │   └── stash_filter.gd           # Stash filter model: gear-slot filter + custom dimensions & sorting
│   │   └── events/               # Headless simulation event data transfer objects
│   │       └── action_log_entry.gd       # Standardized action event model (ticks, actors, outcomes)
│   ├── combat/                   # Combat simulation, spatial arena rules & battle actions
│   │   ├── actions/              # Discrete battle command implementations
│   │   │   ├── attack_command.gd         # Melee & ranged attacks with stance & range gating
│   │   │   ├── defend_command.gd         # Defensive stance transition & incoming damage mitigation
│   │   │   ├── focus_command.gd          # Tactical buff command (+50% ATK for 2 rounds)
│   │   │   ├── move_command.gd           # Cardinal movement (1 tile) & sprinting (2 tiles)
│   │   │   ├── target_command.gd         # Intent lock onto specific opponent
│   │   │   ├── enter_neutral_command.gd  # Reset active combat stance to Neutral
│   │   │   └── hold_stance_command.gd    # Stance preservation across timeline blocks
│   │   ├── session/              # Headless combat session & encounter definitions
│   │   │   ├── combat_session.gd         # Battle state: rounds, resolution, snapshots, replay, selection, log
│   │   │   ├── combat_encounter.gd       # Encounter plug-in base (combatants, deployment, plans, triggers, art)
│   │   │   ├── goblin_ambush_encounter.gd # Scripted 3-round demo encounter (reference battle)
│   │   │   ├── skirmish_encounter.gd     # Player-planned encounter vs AI-planned goblins (arena default)
│   │   │   └── combat_log_builder.gd     # Resolved actions -> log cells; resolver messages -> BBCode
│   │   ├── planning/             # Per-round command plans (headless)
│   │   │   ├── plan_entry.gd             # One planned command: kind, block, direction, target; builds the BattleCommand
│   │   │   └── combat_plan.gd            # Per-unit timeline plan: validation, replace, AP budget, commit to resolver
│   │   ├── ai/                   # Computer-controlled planning
│   │   │   ├── ai_profile.gd             # Role (Aggressor/Skirmisher/Sniper/Support), strike power, flee & guard thresholds
│   │   │   └── tactical_ai.gd            # Plans a unit's 10-block round: BFS path-finding, targeting, role patterns, flee
│   │   └── arena/                # Arena spatial rules & resolution
│   │       ├── combat_grid.gd            # 9x9 grid, Manhattan distance, open/closed arenas, escape tiles, terrain walkability
│       ├── terrain.gd                # Terrain types (14) with walkability, cover, move cost & hazard metadata
│       ├── combat_zones.gd           # 8 preset 9x9 combat zones (forest, river, lava cavern, ...)
│   │       ├── combat_resolver.gd        # 10-block timeline resolution, action scheduling, casualty cleanup
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
│   │   └── arena_test_suite.gd       # 10 automated test suites with 84 assertions
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
│       │   ├── dojo_unit_inspector_panel.gd  # Component 1: identity, 6 stats, masteries (static layout)
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
│       │   └── combat_timeline_panel.gd  # 10-block ruler & focused unit track
│       ├── combat_grid_view.gd       # 9x9 grid rendering (54px tiles), tokens, tweens & floating text
│       ├── roster_panel_ui.gd        # Party status (2x2) and hostile status (5x2) docks
│       ├── unit_inspector_ui.gd      # Left dock (avatar, vitals, stats, action decks)
│       ├── tactical_forecast_ui.gd   # Right dock (range checks, damage forecast, arena intel)
│       ├── playback_controls_ui.gd   # Step prev/next, Auto-Play, Reset, Round navigation
│       ├── timeline_dock_ui.gd       # Bottom dock (10-block ruler, unit track, enemy intent mask)
│       ├── action_log_dock_ui.gd     # Top action dock displaying active block resolution
│       ├── combat_log_modal.gd       # Fullscreen expanded event stream modal with category filters
│       ├── timeline_modal.gd         # Fullscreen synchronized multi-unit timeline modal
│       └── character_equipment_modal.gd # Party & Stash screen: 3 columns (party viewer, gear loadout, party stash) sharing one DojoSession
├── check_project.ps1             # 4-step automated validation script (cache, syntax, arena suite, dojo suite)
├── check_project.bat             # Batch launcher for check_project.ps1
├── run_gear_dojo.bat             # Dedicated launcher for Gear, Skill & Dummy Benchmark Dojo
└── run_game.bat                  # Quick launcher for the combat arena (main scene)
```

---

## Currently Implemented Features & Functionality

### 1. 10-Block Simultaneous Resolution Engine

- **Synchronous Execution**: Block-by-block progression from Block 1 to 10. Actions specify start blocks and duration costs.
- **Initiative Sorting**: Multiple actions completing on the same block resolve deterministically via speed ordering `(priority * 1000) + speed`.
- **Casualty Action Cancellation**: If an active combatant is eliminated mid-round, all future scheduled commands for that unit are cancelled immediately.
- **Round Rollover**: Buff durations decrement cleanly, lingering stances carry over deterministically, and fallen/withdrawn units are purged from subsequent rounds.

### 2. Stance System & Matchup Multipliers

- **6 Discrete Stances**: `NEUTRAL`, `OFFENSIVE`, `DEFENSIVE`, `TACTICAL`, `UTILITY`, `MOBILE`.
- **Dynamic Combat Modifiers**:
  - `OFFENSIVE` vs `MOBILE`: $+15\%$ damage.
  - `OFFENSIVE` vs `TACTICAL`: $+25\%$ damage.
  - `DEFENSIVE`: Flat damage mitigation ratio (e.g. $50\%$ reduction against attacks).
  - `TACTICAL`: Channel focus buffs (+50% attack power for 2 rounds).
  - `MOBILE`: Mobility bonuses and position repositioning.

### 3. 9x9 Spatial Grid & Spatial Rules

- **Arena Geometry**: Fixed 9x9 grid (`0..8` on both axes) with landmark center at `(4, 4)`.
- **Center-Ring Deployment**: Initial spawn radius around `(4, 4)` offset by attack range so units can engage center on Block 1.
- **Spatial Range Gating**: Attack commands validate target distance against attacker range; out-of-range attacks fail cleanly.
- **Perimeter Escape & Partial Withdrawal**: Stepping onto designated exit tiles (e.g. `(8, 4)`) triggers safe partial withdrawal, removing the unit from active combat without penalty.
- **Reinforcement Ingress**: Mid-round triggers (e.g. Block 4 ambush) dynamically deploy reinforcements onto grid coordinates.

### 4. Deterministic Scrubbing & Replay System

- **State Snapshots (`BlockSnapshot`)**: Full state (HP, SP, stances, targets, active flags, coordinates) captured per block.
- **Two-Way Stepping**: Jump backward and forward through blocks (`Step Prev`, `Step Next`, ruler scrub) with restored visual states.
- **Multi-Round Historical Traversal**: Review previous completed rounds (Round 1, Round 2) without losing active simulation state.

### 5. Modular Combat UI (`combat_arena.tscn`)

- **Headless `CombatSession`**: Owns one battle: the encounter's combatants, grid and resolver, round flow, block-by-block resolution, per-block snapshots, multi-round traversal, selection/hover and the action log. Panels only read it and call its actions; every change ends in `changed`, visual one-shots use `fx_requested` / `hover_changed`. The whole battle flow is headless-testable.
- **Pluggable `CombatEncounter`**: Defines *what* a battle is (combatants, deployment, per-round plans, mid-round triggers, narration, art). The scripted 3-round Goblin Ambush is `GoblinAmbushEncounter`; future AI-planned or region-generated encounters plug in the same way. Its combatants are the single source of truth for simulation and rendering.
- **Panels (`CombatPanel`)**: Party roster, enemy roster, resolution log, unit inspector, combat grid, tactical forecast, playback controls and timeline are independent panels bound to the session, each wrapping an existing view component. The inspector, roster and grid all render the replayed snapshot when reviewing a past block.
- **Static Dock Host (`CombatDockHost`)**: Fixed regions (top row, left/center/right, bottom bar) filled from data-driven layout presets (`Standard`, `Mirrored`). The game UI stays static; **layout edit mode** (`⚙` / `F9`) overlays an `⇱ Undock` tag on each panel without moving anything, floats panels into `UiWindow`s (their dock keeps a same-size placeholder) and docks them back when closed. The host also owns the full-screen modals and shortcuts (`Space` play, `Q`/`E` blocks, `[`/`]` log, `L` log modal, `F`/`Tab` timeline, `G` party & stash, `Esc` close).
- **Custom SVG Visual Assets**: Custom vector graphics for characters, unit tokens, and grid tiles.
- **Allied & Enemy Status Roster Docks**: Real-time HP bars, SP gauges, status badges, and position coordinates.
- **Planning Phase (CombatPlan)**: In player-planned encounters (`SkirmishEncounter`, the arena default) each round starts in PLANNING: pick a command in the inspector's palette (Attack, Defend, Focus, Move/Sprint with a direction, Hold, Neutral), then click a free timeline block to place it; click a placed command to replace it, right-click to remove it. Attacks aim at the selected enemy (else the nearest). Every command costs AP (12 per round; Hold/Neutral are free) and blocks, and invalid placements are rejected with a reason. **Auto-plan** drafts a hero's round with the AI. Stepping/resolving locks the plan and schedules it together with the enemies' plans. **Reset round** rewinds units to how the round started.
- **Enemy AI (`TacticalAI`)**: Each goblin plans its whole round at round start from an `AiProfile`: *Aggressor* (nearest foe, closes in), *Skirmisher* (weakest foe, backs off when hurt), *Sniper* (holds max range, retreats from melee), *Support* (opens with Focus, then ranged). Units path-find around walls/water/lava with BFS, sprint when AP allows, guard once when hurt and flee to the exit at 25% HP. Plans commit blind, so moving targets can make them fizzle. Enemy tracks stay masked on the timeline; the inspector's *Threat Intel* shows each hostile's role, power and thresholds.
- **Unit Inspector & Command Palette**: Unit profile portrait, speed, range, AP left and the interactive palette (allies) or AI threat intel (hostiles).
- **Tactical Forecast Dock**: Real-time Manhattan distance, in-range validation, matchup multiplier, and estimated strike damage.
- **Action Feed & Modals**:
  - Compact active block log dock.
  - Filterable fullscreen action stream modal (`ATTACK`, `DEFEND`, `MOVE`, `BUFF`, `EVENT`).
  - Synchronized multi-unit timeline modal.
- **Visual Combat FX**: Tweened attacker lunges, directional impacts, damage flash, stance pulse glows, and floating combat text (`-25 [MIT]`, `⚑ WITHDRAWN!`, `💀 DEFEATED!`).

### 6. Character Specs, 6-Stat Framework & Mastery Engine (System 5)

- **6-Stat Architecture**:
  - **3 Active Stats (Combat-Primary)**: `STR` (Physical Power, Armor Burden mitigation), `DEX` (Initiative Speed, Ranged Power, Crit Hit Rate), `INT` (Tactical & Skill Power, Buff potency, Debuff Resistance).
  - **3 Passive Stats (Non-Combat-Primary)**: `CONS` (Travel endurance, Max HP scaling), `PERC` (Secret/trap spotting, Timeline intent unmasking), `KNOW` (Lore/dialogue, route danger reduction, tactical forecast intel).
- **Derived Combat Metrics**:
  - **Max HP**: $60 + (\text{CONS} \times 8) + (\text{STR} \times 2) + \text{Accessory HP}$.
  - **Initiative Speed**: $\max(1, 8 + \text{DEX} - \text{Total Armor Burden} + \text{Accessory Speed})$.
  - **Crit Hit Rate** (DEX-based): $5.0\% + (\text{DEX} \times 1.5\%)$ with $2.5\times$ base critical damage factor.
  - **Debuff Resistance** (INT-based): $\text{INT} \times 2.5\%$, mitigating status ailment chance and penalty severity.
  - **Effective Melee / Ranged Power**:
    $$\text{Attack Power} = \left(\text{Weapon Power} \times (1 - \text{Deficit}) + \text{Stat} \times \text{Scaling}\right) \times (1 + \sum \text{Buffs} - \sum \text{Debuffs})$$
    where $\text{Weapon Power} = \text{Base Power} + \text{Tier Bonus} + \text{Mastery Bonus}$.
- **10-Level Cap & Progression**:
  - Guaranteed stat growths per level based on Class Archetype (`Warrior`, `Ranger`, `Shaman`, `Brawler`, `Scout`, `Berserker`).
  - 1 free attribute point per level for customization and flavor.
  - Bulk of free points awarded via outside exploration events (ancient shrines, quest rewards, tomes).
- **Horizontal Progression (Mastery System)**:
  - Weapon Masteries (Sword, Bow, Staff, etc.): Provide stat threshold discounts and direct Weapon Power bonuses ($+2\text{ Weapon Power}$ per rank) affecting combat damage calculations.
  - Lore & Faction Masteries: Contextual knowledge bonuses in specific biomes or dialogue interactions.

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
  1. **Unit Inspector**: portrait, level/HP, the 6 base attributes with BBCode hover breakdowns (base, mastery, gear, buffs) and weapon masteries. `Unspent Points` opens the **Allocate** overlay (the `-` button only undoes newly allocated points; `Reset`/`Confirm`/`Cancel`), so the inspector itself never changes layout.
  2. **Unit Gear**: Main Hand, Off Hand, Body Armor, Accessories and the 3-slot Battle Pouch as cards. `Equip`/`Swap`/`Load` open an overlay picker filtered to class-eligible stash items valid for that slot; `Unequip` returns gear to the stash. **Quick Setup** applies a one-click loadout preset (Sword & Board, Claymore Bruiser, Claymore + Backup Blade, Titan Hammer deficit test, Dual Wield, Archer with dagger backup, Blade & Dagger, Arcanist), spawning missing pieces and reporting items the class cannot wield. `Stash Manager` opens the Party & Stash screen. Cards for a 2H main + off-hand weapon show which weapon is `ACTIVE` or `sheathed`.
  3. **Combat Region**: hero and dummy tokens on a target mat with live HP bars, stance/guard/buff status, lunge, shake and glow animations, floating combat text, plus dummy `Toggle Defend` and `Full Heal` controls.
  4. **Test Console**: attack actions (Main/Off-Hand/Dual Strike/Ranged, auto-gated by the equipped weapons and active-weapon state), `Focus Buff`, `Shield Defend` toggle, `Stance Shift`, `Switch Weapon (1b)`, `Force Crit`, a skill-multiplier selector (×0.5 to ×2.0), the battle pouch selector, and **Benchmark ×10 / ×100** simulations (min/avg/max, crit count, expected damage per swing, swings to defeat the dummy) that never change dummy HP.
  5. **Derived Stats**: melee/ranged/off-hand power, initiative speed (burden and party overload shown separately), max health, flat mitigation, unit burden, party carry (with load bar), attack range, crit rate, and debuff resistance. Hover any row for the formula with the live numbers substituted.
  6. **Test Log**: renders every dojo event with the same action-cell UI as the combat log (`CombatLogModal.format_action_cell`), with category filters and a `Clear` button. Click an entry to see its step-by-step damage breakdown in the detail pane.
- **Comprehensive Combat Damage Formula Engine**:
  $$\text{Final Damage} = \max\left(1, \left\lfloor \text{Attack Power} \times \text{Skill Mult} \times (1 - \text{Mitigation}) \times (1 + \text{Stance Mult}) \times \text{Crit Factor} \times (1 + \text{Effect Bonus} - \text{Effect Penalty}) \right\rceil\right)$$
  - **Attack Power**: Base melee or ranged power derived from weapon power, stat scaling, and status multipliers.
  - **Skill Multiplier**: Move-specific damage multiplier (e.g., $1.0\times$ for strike, $1.5\times$ for heavy strike).
  - **Mitigation**: Opponent shield/armor mitigation ratio ($0\%$, $25\%$, $50\%$, $75\%$).
  - **Stance Multiplier**: $+/-$ matchup multiplier based on stance interaction.
  - **Crit Factor**: If critical hit ($P(\text{Crit Rate})$ or forced via console), applies $2.5\times + \text{bonus crit dmg}$; otherwise $1.0\times$.
  - **Effect Modifiers**: Monster-specific or elemental bonus/penalty multipliers.
- **Shared Combat Math**: Every strike goes through `DamageCalculator.compute`, the same authoritative formula the arena uses, and the breakdown shown in the Test Log is generated from that result.

---

## Headless Testing & Automated Quality Assurance

All features are tested headlessly without requiring a graphical window or GPU.

The test suite in [arena_test_suite.gd](scripts/tests/arena_test_suite.gd) executes **10 test suites** covering **84 individual assertions**:

1. **Combat UI Framework & Behaviour Test** (8 assertions): all 8 panels mounted in their dock regions, tokens & roster bound to the simulated combatants, selection → inspector, tactical forecast, log & timeline modal lifecycle, timeline block inspection, undock/dock + layout presets + action FX dispatch.
2. **Warrior vs 3 Goblins Battle** (9 assertions): Multi-round mitigation, stance carryover, Focus buff persist/expire, multi-enemy defeat.
3. **9x9 Combat Grid & Withdrawal Test** (7 assertions): Center-ring deployment, spatial range gating, coordinate stepping to `(4, 4)`, exit tile escape, and casualty cleanup.
4. **Resolution Feed Logging Test** (7 assertions): Movement, skill, defend, attack, withdrawal, and defeat log tags.
5. **Goblin Ambush & Casualty Test** (9 assertions): Reinforcements arrival at Block 4, casualty action cancellation, multi-round rollover, and victory state.
6. **Character Specs & Inventory Test** (10 assertions): Archetype 6-stat baselines, derived attributes, level-up guaranteed growths, free point allocation, event point rewards, horizontal mastery weapon power bonus, equipment stat requirement check, threshold tiers boost, deficit wield penalties, and battle pouch consumable usage in combat.
7. **Gear Equip UI & Test Dummy Suite** (10 assertions): Equipment modal mounting, hero switching, stash equipping, Claymore Threshold Tier 1 (+4 Power) boost, Titan Warhammer deficit penalty, armor burden speed penalty, unequipped item return to stash, training dummy 500 HP baseline, base strike and 50% defense mitigation formula, battle pouch consumable damage & removal.
8. **Combat Session Flow Test** (8 assertions): Through `CombatSession` headlessly, only deployed enemies in view before the ambush, ambush event at `R1-B04`, Round 1 resolving into replay with round advance offered, replay playhead reading per-block snapshots, round advance baselines with past rounds reviewable, Rounds 2-3 resolving to total victory, timer-free auto-play ticks, and round reset.
9. **Art, Terrain & Zones Test** (9 assertions): team-colored class tokens and portraits, encounter art follows sides, terrain glyph round-trip and walkability, all 8 zones valid, grid blocks blocked tiles, Goblin Ambush wins on every zone, tile and item icon rasterization.

10. **Planning & Enemy AI Test** (7 assertions): plan validation (timeline bounds, overlap/replace, AP budget, targets), party plan previewed on the timeline then locked and scheduled alongside the AI plans, Aggressor closing in and path-finding around walls, Sniper backing away, Support opening with Focus, wounded units fleeing and withdrawing, deterministic AI-vs-AI skirmishes on all 8 zones, and round reset rewinding state.
Additionally, `scenes/gear_dojo.tscn` runs **36 dedicated headless assertions** ([dojo_assertions.gd](scenes/dojo/dojo_assertions.gd)) when launched with `--headless`:

- **Rules & formulas (Dojo-1 to 18)**: dummy vitals, melee formula with sword mastery, 50% mitigation, focus buff scaling, threshold tiers, deficit penalties, melee/ranged power gating, accessory stat propagation, class archetype equip restrictions, 1H/2H and off-hand scaling, crit rate and damage factor, stat allocation bounds, and reset to baseline.
- **Session behavior (Dojo-19 to 25)**: Quick Setup presets, class-proficiency skipping, 2H shield refusal / off-hand weapon acceptance, 1-block active-weapon switching, 1H dual-wield Dual Strike, stash duplicate integrity, strike log entries with breakdowns, rejected illegal actions logged as `FAILED`, and benchmark simulation leaving dummy HP untouched.
- **UI smoke tests (Dojo-26, 27)**: all six panels mount in the workspace and the Test Log mirrors the session log; the Party & Stash screen shares the session (party list, selection, stash equip).
- **Slots, filters & carry (Dojo-28 to 32)**: off-hand-only weapons and shields, per-slot stash filters (shield under Off Hand) plus stat/rarity/damage/hand/usable/search/sort custom filters, a party viewer that always shows 4 slots, per-unit burden vs party-wide carry overload, and overlay windows opening/closing on request.

### Automated Check Script (`check_project.ps1`)

Runs in 4 phases:

- `[1/4]` Verifies global class cache and asset imports (`--headless --editor --quit-after 50`).
- `[2/4]` Validates GDScript syntax & static typing across all 67 project scripts (`--headless --check-only`).
- `[3/4]` Executes the headless arena test harness (`--headless scenes/test_arena.tscn`), asserting exit code 0.
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

### Run Dedicated Gear & Skill Dojo

Launch the Gear, Skill & Dummy Benchmark Dojo in windowed mode:

```cmd
run_gear_dojo.bat
```

### Run Interactive Combat Arena

Launch the combat arena in windowed mode:

```cmd
run_game.bat
```

Or open the project in **Godot 4.7+** and press Play (main scene `res://scenes/combat_arena.tscn`), or run `res://scenes/gear_dojo.tscn`. `res://scenes/test_arena.tscn` runs the test suite first, then opens the arena.

---

## Roadmap & Features Yet to be Implemented

The following modular systems and capabilities are planned according to the master game design specification:

### 1. Planning Phase Follow-ups

*Status: Core planning implemented (palette, timeline placement, AP budget, replace/remove, auto-plan, round reset).*

- [x] Interactive timeline command placement and AP budget validation.
- [x] Dynamic command replacement and removal.
- [x] CombatSession extracted from `test_arena.gd`; combat UI rebuilt as session-bound panels in a static dock host (`combat_arena.tscn`).
- [ ] Drag-and-drop on the timeline and block shifting (push later commands).
- [ ] Ghost path / range preview of planned moves and attacks on the grid.
- [ ] Skill and item commands in the palette (`SwitchWeaponCommand`, Dual Strike, battle-pouch items).
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
- [ ] **Arena weapon commands**: `SwitchWeaponCommand` (1 block, toggles `Inventory.active_weapon_slot`) and a `DualStrikeCommand` combo for 1H pairs, so the dojo weapon rules apply on the 10-block timeline.
- [ ] **Stance Disruption & Stagger**: Critical attacks or guard breaks that inflict the `Staggered` condition (unable to defend/dodge for $N$ blocks).
- [ ] **Area of Effect (AoE)**: Line, cone, and radial blast patterns on the 9x9 grid.
- [ ] **Environmental Hazards**: Fire, ice, and poison terrain tiles that apply effects on entry/end of round.
- [ ] **Status Conditions**: Bleed, Poison, Blind, Stun, and Haste.

### 5. Items & Consumable Inventory Engine (`System 7`)

*Status: Core models & interactive equipment UI implemented.*

- [x] Shared party bag container model and 3-slot combat battle pouches.
- [x] Stat-gated equipment with tiered thresholds, deficit penalties, and weapon mastery discounts.
- [x] Combat consumable usage and HP/buff application.
- [x] Party & Stash screen (`CharacterEquipmentModal`): party viewer, gear loadout and filterable stash in three columns.
- [x] Preset catalog (`ItemCatalog`) with weapons, armor, shields, and accessories.
- [ ] Equipment durability, weapon crafting, and socketed gems.

### 6. Character Progression & Stats Engine (`System 5`)

*Status: Core models & interactive equipment modal inspection implemented.*

- [x] 6-stat architecture (STR, DEX, INT | CONS, PERC, KNOW) and derived formulas.
- [x] 10-level cap with guaranteed class growths and free attribute point allocation.
- [x] Horizontal mastery system (Weapon Masteries, Lore/Biome Masteries, Faction scores).
- [x] Integration with `Combatant` model.
- [x] UI Character Sheet & gear equip modal (`CharacterEquipmentModal`) and the modular Gear Dojo workspace (`gear_dojo.tscn`).
- [ ] Passive trait slot selection and talent trees.

### 7. Region Map & Exploration Engine (`System 2`)

*Status: Specification defined in `.context/GAME_DESIGN.md`.*

- [ ] 2D top-down grid exploration with tile types (`SAFE` vs `DANGER`).
- [ ] Danger step counter triggering mob encounter rolls.
- [ ] Visible roaming elite/boss encounters with detection radii.
- [ ] Region exit point tiles that transition to the World Map.

### 8. World Map & Travel Engine (`System 1`)

*Status: Specification defined in `.context/GAME_DESIGN.md`.*

- [ ] Regional node/route graph linking world biomes.
- [ ] Travel time consumption and road danger rolls.
- [ ] Seamless multi-layer transition: World Map $\leftrightarrow$ Region Map $\leftrightarrow$ Combat Arena.

### 9. NPC, Dialogue & World Story Engine (`System 9`)

*Status: Specification defined in `.context/GAME_DESIGN.md`.*

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
