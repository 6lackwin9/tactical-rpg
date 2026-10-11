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
├── assets/                       # Custom vector SVG assets
│   ├── enemy/                    # Hostile unit assets
│   │   ├── sprites/              # Full-character illustration SVG assets (archer, berserker, brawler, dummy)
│   │   └── tokens/               # Overhead circular unit token SVG assets (dummy token included)
│   ├── party/                    # Player party unit assets
│   │   ├── sprites/              # Full-character illustration SVG assets (warrior, ranger)
│   │   └── tokens/               # Overhead circular unit token SVG assets
│   └── tiles/                    # Arena grid tile SVG textures (tile_floor, tile_center, tile_exit)
├── data/                         # [Placeholder] Game data definitions & balance tables
├── scenes/
│   ├── test_arena.tscn           # Visual interactive combat arena harness scene
│   ├── test_arena.gd             # Harness orchestration, replay scrubbing & UI wiring
│   ├── gear_dojo.tscn            # Dedicated Gear, Skill & Damage Formula Benchmark scene
│   └── gear_dojo.gd              # Dojo orchestration, training dummy benchmark & damage formula feed
├── scripts/
│   ├── core/                     # Foundational rules engine primitives & base models
│   │   ├── combat/               # Core unit, stance & command models
│   │   │   ├── battle_command.gd         # Abstract base command class (duration, cost, initiative)
│   │   │   ├── combatant.gd              # Core unit model (HP, SP, speed, range, stance, buffs, target)
│   │   │   └── combat_types.gd           # Stances, enum helpers, matchup multipliers & formatting
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
│   │   │   └── item_catalog.gd           # Presets catalog: Claymore, Warhammer, Bow, Shields, Armor, Draughts, Bombs
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
│   │   └── arena/                # Arena spatial rules, scenario setups & resolution
│   │       ├── combat_grid.gd            # 9x9 grid, Manhattan distance, open/closed arenas, escape tiles
│   │       ├── combat_scenario.gd        # Encounter setup, ally/enemy rosters, reinforcements, asset binding
│   │       ├── combat_resolver.gd        # 10-block timeline resolution, action scheduling, casualty cleanup
│   │       └── block_snapshot.gd         # Historical state capture for deterministic scrubbing/replay
│   ├── tests/                    # Headless test suites & validation harnesses
│   │   └── arena_test_suite.gd       # 7 automated test suites with 60 assertions
│   └── ui/                       # View layer, docks, components & full-screen modals
│       ├── combat_arena_controller.gd# CanvasLayer coordinator & pure view composer
│       ├── combat_grid_view.gd       # 9x9 grid rendering (54px tiles), tokens, tweens & floating text
│       ├── roster_panel_ui.gd        # Party status (2x2) and hostile status (5x2) docks
│       ├── unit_inspector_ui.gd      # Left dock (avatar, vitals, stats, action decks)
│       ├── tactical_forecast_ui.gd   # Right dock (range checks, damage forecast, arena intel)
│       ├── playback_controls_ui.gd   # Step prev/next, Auto-Play, Reset, Round navigation
│       ├── timeline_dock_ui.gd       # Bottom dock (10-block ruler, unit track, enemy intent mask)
│       ├── action_log_dock_ui.gd     # Top action dock displaying active block resolution
│       ├── combat_log_modal.gd       # Fullscreen expanded event stream modal with category filters
│       ├── timeline_modal.gd         # Fullscreen synchronized multi-unit timeline modal
│       └── character_equipment_modal.gd # 3-column modal: party tabs, 6 stats, equipped slots & stash
├── check_project.ps1             # 3-step automated validation script (cache, syntax, integration)
├── check_project.bat             # Batch launcher for check_project.ps1
├── run_gear_dojo.bat             # Dedicated launcher for Gear, Skill & Dummy Benchmark Dojo
└── run_game.bat                  # Quick launcher for interactive test arena in Godot
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

### 5. Modular UI & Visual Feedback Framework

- **Pure View Coordinator (`CombatArenaController`)**: Zero layout coupling in simulation logic.
- **Custom SVG Visual Assets**: Custom vector graphics for characters, unit tokens, and grid tiles.
- **Allied & Enemy Status Roster Docks**: Real-time HP bars, SP gauges, status badges, and position coordinates.
- **Unit Inspector & Action Palette**: Unit profile portrait, speed, attack range, and action deck.
- **Tactical Forecast Dock**: Real-time Manhattan distance, in-range validation, matchup multiplier, and estimated strike damage.
- **Action Feed & Modals**:
  - Compact active block log dock.
  - Filterable fullscreen action stream modal (`ATTACK`, `DEFEND`, `MOVE`, `BUFF`, `EVENT`).
  - Synchronized multi-unit timeline modal.
- **Visual Combat FX**: Tweened attacker lunges, directional impacts, damage flash, stance pulse glows, and floating combat text (`-25 [MIT]`, `⚑ WITHDRAWN!`, `💀 DEFEATED!`).

### 6. Character Specs, 6-Stat Framework & Mastery Engine (System 5)

- **6-Stat Architecture**:
  - **3 Active Stats (Combat-Primary)**: `STR` (Physical Power, Armor Burden mitigation), `DEX` (Initiative Speed, Ranged Power), `INT` (Tactical & Skill Power, Buff potency).
  - **3 Passive Stats (Non-Combat-Primary)**: `CONS` (Travel endurance, Max HP scaling), `PERC` (Secret/trap spotting, Timeline intent unmasking), `KNOW` (Lore/dialogue, route danger reduction, tactical forecast intel).
- **10-Level Cap & Progression**:
  - Guaranteed stat growths per level based on Class Archetype (`Warrior`, `Ranger`, `Shaman`, `Brawler`, `Scout`, `Berserker`).
  - 1 free attribute point per level for customization and flavor.
  - Bulk of free points awarded via outside exploration events (ancient shrines, quest rewards, tomes).
- **Horizontal Progression (Mastery System)**:
  - Weapon Masteries (Sword, Bow, Staff, etc.): Provide stat threshold discounts and contextual stat bonuses when wielding archetypal weapons.
  - Lore & Faction Masteries: Contextual knowledge bonuses in specific biomes or dialogue interactions.

### 7. Equipment, Requirements & Inventory Engine (System 7)

- **Item Taxonomy**: Weapons, Shields, Armor, Accessories, Consumables, Materials, Quest items.
- **Stat-Gated Gear & Threshold Convention**: Equipment requirements use the convention `[Req <STAT level0>, <STAT level1>]`, where `level0` enables wielding without penalty and `level1` unlocks bonus effects (`+Wep power`, `+<STAT>`, or special combat effects like `1.5x against goblins`).
- **Dynamic Deficit Scaling**: Wielding weapons with a stat deficit applies a dynamic penalty ratio (`minf(0.60, deficit * 0.15)`) to effective weapon power and delays timeline action blocks.
- **Class Archetype Gear Restrictions**: Class archetypes enforce allowable gear proficiencies (`can_equip_gear`); accessories have no restrictions and can be equipped by any class.
- **Accessory Stat Propagation**: Equipped accessories (rings, amulets) propagate stat bonuses directly into character stats, dynamically updating effective attributes, max HP, and attack power.
- **Armor Burden & Speed**: Equipped heavy armor and shields inflict armor burden, directly reducing initiative speed (`8 + DEX - burden`).
- **Party Stash & Battle Pouches**:
  - Shared party stash capacity scales dynamically with party STR and CONS (`30 + (STR * 3) + (CONS * 1)`).
  - 3 combat battle pouch quick-slots per combatant for consumables used in combat (`ItemCommand` in Utility Stance).

### 8. Dedicated Gear & Skill Dojo (`gear_dojo.tscn`)

- **Separated Scene Architecture**: Distinct from `test_arena.tscn` (which focuses on 10-block visual combat and ambush scenarios), `scenes/gear_dojo.tscn` is a dedicated workbench for gear equipping, stat inspection, and damage formula validation.
- **Card-Based Active Loadout & Quick Pouches**: Base screen dock displays dedicated slot cards for Main Hand, Off Hand, Body Armor, Accessory 1, and Accessory 2, alongside a 3-slot Battle Pouch quick-belt with compact micro-card tooltips.
- **Character Equipment Modal (`CharacterEquipmentModal`)**:
  - **Column 1**: Party hero tabs, 6 primary stats with $+/-$ allocation, derived metrics (HP, Melee/Ranged Power, Speed, Shield Mitigation, Armor Burden bar), and horizontal weapon mastery ranks.
  - **Column 2**: Equipment loadout slots (Main Hand, Off Hand, Body Armor, Accessory 1 & 2) and 3 combat battle pouch slots with unequip actions.
  - **Column 3**: Party stash inventory with category filters (`ALL`, `WEAPON`, `ARMOR`, `ACCESSORY`, `CONSUMABLE`), class restriction badges, tier thresholds, and equip buttons.
- **Interactive Training Dummy & Damage Formula Benchmark**:
  - 500 HP Training Dummy with tunable defense mitigation ratio ($0\%$, $25\%$, $50\%$, $75\%$).
  - Live Action Console: Trigger Melee Strike, Ranged Volley, Focus Buff, Defend Stance, Throwing Fire Bomb, and Stance shifts.
  - Real-time mathematical damage calculation feed explaining:
    $$\text{Actual Damage} = \max\left(1, \left\lfloor \left(\text{Effective Power} \times \text{Stance Multiplier}\right) \times (1 - \text{Mitigation Ratio}) \right\rceil\right)$$
  - Live visual feedback: Floating combat damage numbers, hit sparks, hero attack lunges, and dummy health bars.

---

## Headless Testing & Automated Quality Assurance

All features are tested headlessly without requiring a graphical window or GPU.

The test suite in [arena_test_suite.gd](scripts/tests/arena_test_suite.gd) executes **7 test suites** covering **60 individual assertions**:

1. **Combat UI Framework & Behaviour Test** (8 assertions): UI dock creation, scenario mounting, token generation, inspector synchronization, tactical forecast, modal lifecycle, signal relay, and action FX dispatch.
2. **Warrior vs 3 Goblins Battle** (9 assertions): Multi-round mitigation, stance carryover, Focus buff persist/expire, multi-enemy defeat.
3. **9x9 Combat Grid & Withdrawal Test** (7 assertions): Center-ring deployment, spatial range gating, coordinate stepping to `(4, 4)`, exit tile escape, and casualty cleanup.
4. **Resolution Feed Logging Test** (7 assertions): Movement, skill, defend, attack, withdrawal, and defeat log tags.
5. **Goblin Ambush & Casualty Test** (9 assertions): Reinforcements arrival at Block 4, casualty action cancellation, multi-round rollover, and victory state.
6. **Character Specs & Inventory Test** (10 assertions): Archetype 6-stat baselines, derived attributes, level-up guaranteed growths, free point allocation, event point rewards, horizontal mastery contextual bonuses, equipment stat requirement check, threshold tiers boost, deficit wield penalties, and battle pouch consumable usage in combat.
7. **Gear Equip UI & Test Dummy Suite** (10 assertions): Equipment modal mounting, hero switching, stash equipping, Claymore Threshold Tier 1 (+4 Power) boost, Titan Warhammer deficit penalty, armor burden speed penalty, unequipped item return to stash, training dummy 500 HP baseline, base strike and 50% defense mitigation formula, battle pouch consumable damage & removal.

Additionally, `scenes/gear_dojo.tscn` contains **12 dedicated headless assertions** verifying dummy vitals, focus buff scaling, threshold tiers, weapon power melee/ranged gating, accessory stat propagation, and class archetype equip restrictions.

### Automated Check Script (`check_project.ps1`)

Runs in 3 phases:

- `[1/3]` Verifies global class cache and asset imports (`--headless --editor --quit-after 50`).
- `[2/3]` Validates GDScript syntax & static typing across all 38 project scripts (`--headless --check-only`).
- `[3/3]` Executes the full headless test harness (`--headless scenes/test_arena.tscn`), asserting exit code 0.

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

Or open the project in **Godot 4.7+** and run `res://scenes/test_arena.tscn` or `res://scenes/gear_dojo.tscn`.

---

## Roadmap & Features Yet to be Implemented

The following modular systems and capabilities are planned according to the master game design specification:

### 1. Interactive Command Input & Planning Phase (Next Priority)

*Status: Architecture ready, planning UI in progress.*

- [ ] Interactive timeline command placement (drag/click to assign player commands onto Blocks 1–10).
- [ ] Action Point / Stamina cost budget verification during planning.
- [ ] Dynamic command replacement and block shifting.
- [ ] Extraction of `CombatSession` orchestration out of `test_arena.gd` to completely decouple playback from the harness.

### 2. Enemy AI Decision Engine (`System 8`)

*Status: Currently runs pre-scripted tactical scenarios; dynamic AI engine planned.*

- [ ] AI Intent Evaluation: Evaluate party vulnerabilities, calculate distances, and choose targets.
- [ ] Role Archetypes:
  - *Aggressor*: Closes distance and executes high-power offensive combos.
  - *Skirmisher / Flanker*: Maneuvers along perimeter, attacks, and retreats.
  - *Sniper / Ranged*: Maintains max range and retreats if enemies close within melee distance.
  - *Support / Shaman*: Buffs allies and debuffs player frontline.
- [ ] Dynamic Flee Heuristic: Automatically triggers `MoveCommand` toward exit tiles when HP falls below 25%.

### 3. Asset Loading & Animation Test Suite (`Test Suite 2`)

*Status: Planned.*

- [ ] Dedicated automated test suite verifying SVG asset loads, dimensions, and texture caching.
- [ ] Animation lifecycle tests (Tween callbacks, duration timing, and queue free verification).

### 4. Advanced Combat Rules & Combo Engine (`System 6`)

*Status: Planned.*

- [ ] **Chained Flow Moves**: Skills that automatically transition into compatible follow-up stances.
- [ ] **Stance Disruption & Stagger**: Critical attacks or guard breaks that inflict the `Staggered` condition (unable to defend/dodge for $N$ blocks).
- [ ] **Area of Effect (AoE)**: Line, cone, and radial blast patterns on the 9x9 grid.
- [ ] **Environmental Hazards**: Fire, ice, and poison terrain tiles that apply effects on entry/end of round.
- [ ] **Status Conditions**: Bleed, Poison, Blind, Stun, and Haste.

### 5. Items & Consumable Inventory Engine (`System 7`)

*Status: Core models & interactive equipment UI implemented.*

- [x] Shared party bag container model and 3-slot combat battle pouches.
- [x] Stat-gated equipment with tiered thresholds, deficit penalties, and weapon mastery discounts.
- [x] Combat consumable usage and HP/buff application.
- [x] UI Equipment & Bag Inspection screens (`CharacterEquipmentModal`).
- [x] Preset catalog (`ItemCatalog`) with weapons, armor, shields, and accessories.
- [ ] Equipment durability, weapon crafting, and socketed gems.

### 6. Character Progression & Stats Engine (`System 5`)

*Status: Core models & interactive equipment modal inspection implemented.*

- [x] 6-stat architecture (STR, DEX, INT | CONS, PERC, KNOW) and derived formulas.
- [x] 10-level cap with guaranteed class growths and free attribute point allocation.
- [x] Horizontal mastery system (Weapon Masteries, Lore/Biome Masteries, Faction scores).
- [x] Integration with `Combatant` model.
- [x] UI Character Sheet & gear equip modal (`CharacterEquipmentModal` & `gear_dojo.tscn`).
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
