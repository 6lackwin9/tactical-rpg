# Modular Tactical RPG - Comprehensive Game Design & Architecture Specification

## 1. Core Vision & Modular Rules-Engine Philosophy

This game is designed as a **Modular Tactical RPG** where every major domain is governed by an **independent, composable Rules Engine**. 
Instead of a monolithic game manager, each system operates on a clean, decoupled **Pipeline / Rules Evaluation Model**:

```
[Domain Context] ---> [Rule 1: Base] ---> [Rule 2: Modifier] ---> [Rule 3: Condition] ---> [Resolved Output]
```

- **Incremental Complexity:** Each engine launches with **Phase 1 (Simple Baseline Rules)** to achieve a stable, playable game loop quickly.
- **Additive Expansion:** New features (weather hazards, status conditions, class synergies, travel curses) are introduced as **additive rule components** without modifying the engine core.
- **Strict Decoupling:** All simulation rules inherit from `RefCounted`, allowing 100% headless automated testing via `godot --headless`.

---

## 2. Master System Blueprint (The 9 Modular Engines)

```mermaid
graph TD
    subgraph Macro Exploration Layer
        WM[1. World Travel Engine] -->|Transit & Encounter Check| RM[2. Region Exploration Engine]
        RM -->|Step on Exit Point| WM
    end

    subgraph Combat Layer
        RM -->|Encounter Triggered| ENC[3. Encounters & Arena Engine]
        ENC --> CGRID[4. Combat Grid & Spatial Engine]
        CGRID --> COMBAT[5. 10-Block Tactical Combat Engine]
        AI[6. Enemy & AI Decision Engine] --> COMBAT
    end

    subgraph RPG Progression & World Layer
        COMBAT --> CHAR[7. Character Spec & Stats Engine]
        COMBAT --> ITEMS[8. Items & Inventory Engine]
        RM --> NPC[9. NPC, Dialogue & World Story Engine]
        CHAR --- SKILLS[Skills & Stance Library]
    end
```

---

## 3. System Specifications

### System 1: World Map & Travel Engine
- **Macro Topology:** A connected node/region graph of key geographical regions (plains, mountains, forests, ruins, settlements).
- **Travel Mechanism:**
  - Selecting a connected destination initiates travel across the world route.
  - **Travel Time Rule:** Travel consumes in-game time units/hours based on distance and route terrain difficulty.
  - **Travel Encounter Rule:** Each travel leg rolls for random encounters based on the route's danger rating. If triggered, transitions to the Combat Layer using the route's environmental biome.
- **Phase 1 Baseline Rules:**
  - Simple point-to-point adjacency list.
  - Flat travel time (e.g., 3 time units per route).
  - Fixed % roll per travel leg for encounter.
- **Future Additive Rules:** Route weather conditions, road hazards, travel supply/food consumption, merchant caravans.

---

### System 2: Region Map & Exploration Engine
- **Local Topology:** A 2D top-down grid representing a specific visited region.
- **Zone Taxonomy:**
  - **Safe Zones:** Towns, camps, sanctuaries. Zero encounter rolls; enables party resting, trading, and dialogue.
  - **Danger Zones:** Wilderness, dungeons, ruins. Movement incurs encounter risk or steps into active hostile zones.
- **Exit Points:**
  - Designated edge or portal tiles on the region map.
  - Stepping onto an Exit Point tile opens the World Map for travel to nearby connected regions.
- **Phase 1 Baseline Rules:**
  - Discrete grid step movement (Vector2i coordinates).
  - Tile-based zone classification (`SAFE` vs `DANGER`).
  - Danger step counter: after $N$ steps in a danger zone, an encounter initiates.
  - Stepping on an exit tile prompts region departure.
- **Future Additive Rules:** Visible roaming wandering mobs, interactive terrain obstacles (bridges, locked gates), interactive loot containers.

---

### System 3: Combat Grid & Spatial Positioning Engine
- **Arena Geometry:**
  - A small, focused 2D grid (e.g., $9 \times 9$ or $11 \times 11$ tiles).
  - **Odd-numbered grid dimension convention:** An odd width/height ensures an unambiguous exact center tile `(center_x, center_y)`.
  - **Region Attributes:** Arena terrain type, background visuals, and environmental modifiers are inherited directly from the region where the encounter was triggered.
- **Deployment & Starting Placement Rule:**
  - Combatants spawn in a ring near the **center of the map**.
  - **Ranged-capability offset:** Spawn distance from center is randomized slightly based on the character's attack range, ensuring **every combatant can target the center of the grid from their starting tile**.
- **Movement Integration:**
  - Movement is scheduled as a `MoveCommand` (1 to 2 blocks) on the 10-block timeline.
  - Moves translate into spatial path traversal on the grid.
- **Boundary Escape & Partial Withdrawal Rule:**
  - Any combatant who moves onto or beyond the perimeter boundary tiles of the combat grid executes a **Withdrawal**.
  - **Partial Withdrawal:** Individual party members or enemies can escape independently. The combat continues seamlessly for all remaining active units.
  - Withdrawn combatants are removed from the active arena timeline without counting as casualties.
- **Combat Termination Rule:**
  - Combat officially terminates when **no hostile units remain on the combat grid** (all enemies are either defeated or have withdrawn, or all player units have withdrawn/fallen).
- **Phase 1 Baseline Rules:**
  - Flat $9 \times 9$ grid with impassable outer boundary except for designated escape borders.
  - Center-based deployment with 1-tile variance based on attack range.
  - 1 Move block = 1 orthogonal tile step.
  - Moving past perimeter = unit withdrawn from battle.
- **Future Additive Rules:** Cover mechanics (half/full cover), impassable obstacles, hazardous elemental ground tiles (fire, mud, ice), height/elevation modifiers.

---

### System 4: 10-Block Tactical Combat & Stance Engine
*(Detailed in `.context/COMBAT_SYSTEM.md`)*
- Fixed 10-block action timeline resolved simultaneously.
- 6 active combat stances (`Neutral`, `Offensive`, `Defensive`, `Tactical`, `Utility`, `Mobile`) plus `Staggered`.
- Action archetypes: Attack, Defend, Skill, Item, Move, Enter Neutral, Hold Stance, Target.
- Deterministic initiative ordering: `(priority * 1000) + speed`.

---

### System 5: Character Spec & Stats Engine
- **Core Attributes:** Max HP, Speed, Physical Power, Magic/Tactical Power, Defense, Range.
- **Phase 1 Baseline Rules:**
  - Base stat definitions with flat level scaling.
  - Stance compatibility flags.
- **Future Additive Rules:** Passive trait slots, equipment stat modifiers, class archetypes, attribute affinities.

---

### System 6: Skills & Combo Engine
- **Skill Definitions:** Actions that consume 1–3 timeline blocks, require specific stances or target locks, and apply damage, buffs, or debuffs.
- **Phase 1 Baseline Rules:**
  - Attack, Defend, Focus buff, Target lock, Hold Stance.
- **Future Additive Rules:** Chained Flow Moves (exiting naturally into another stance), AoE shapes on the combat grid, Stance Disruption / Stagger.

---

### System 7: Items & Inventory Engine
- **Inventory Model:** Shared party bag with consumable items and equippable gear.
- **Phase 1 Baseline Rules:**
  - Consumables used via `ItemCommand` (1–2 blocks, Utility Stance).
  - Healing potions (flat HP restore) and basic throwables.
- **Future Additive Rules:** Equipment slots influencing block speed and unlocked commands, durability, inventory weight.

---

### System 8: Enemies & AI Decision Engine
- **Tactical Intent Planning:** At the start of each 10-block round, AI evaluates the combat state and schedules actions.
- **Phase 1 Baseline Rules:**
  - Deterministic heuristics:
    - If HP $< 25\%$ and near grid edge $\to$ prioritize `MoveCommand` toward boundary (Attempt Withdrawal).
    - If engaged $\to$ alternate Defend and Attack based on target stance.
- **Future Additive Rules:** Role archetypes (Aggressor, Sniper, Support, Coward), threat weighting, flank movement positioning.

---

### System 9: NPC, Dialogue & World Story Engine
- **World State Model:** Key/value flag registry (`Dictionary[String, Variant]`).
- **Phase 1 Baseline Rules:**
  - Static NPC interaction prompts with branching choices.
  - Quest flags unlocking region exit points or triggering story events.
- **Future Additive Rules:** Faction reputation meters, bartering shop systems, timed world events.

---

## 4. Phase 1 Implementation Roadmap

```
Milestone 1: Combat Grid & Spatial Engine (In-progress)
  ├── 1.1 2D Grid Data Model (Size, Center, Perimeter)
  ├── 1.2 Center-ring Deployment with Range Offsets
  ├── 1.3 MoveCommand & Grid Coordinate Validation
  └── 1.4 Perimeter Boundary Escape / Partial Withdrawal Verification

Milestone 2: Region Map & Exploration (2D Grid)
  ├── 2.1 Region Grid & Tile Types (Safe vs Danger)
  ├── 2.2 Danger Step Encounter Trigger
  └── 2.3 Region Exit Points & Transition Hooks

Milestone 3: World Map & Travel Engine
  ├── 3.1 Region Node Graph
  ├── 3.2 Travel Duration & Road Encounter Roll
  └── 3.3 Seamless Transition (World Map <-> Region Map <-> Combat Grid)

Milestone 4: AI Intent & Withdrawal Logic
  └── 4.1 Automated AI Action Planning & Flee Thresholds
```
