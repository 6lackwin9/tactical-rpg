# RPG Kit & Engine Architecture: Systems Analysis & Reuse Matrix

> **Category:** Technical Architecture & Engine Integration  
> **Status:** Systems Specification & Design Alignment  
> **Reference Lore:** [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)  
> **Reference Catalogs:** [RPG_KIT_CATALOG.md](file:///c:/dev/tactical-rpg/docs/RPG_KIT_CATALOG.md), [ENGINE_SPEC.md](file:///c:/dev/tactical-rpg/engine/docs/ENGINE_SPEC.md)

---

## 1. Architectural Philosophy: The Three-Tier Separation

The Aegis engine architecture follows a strict three-tier hierarchy:
1. **Engine Core (`engine/crates/`):** Game-agnostic runtime primitives (RNG, JSON value serialization, save envelopes, command API, canvas painter, timeline widget, audio player).
2. **RPG Kit (`kits/rpg/` & shared rules):** Modular, generic RPG systems and asset libraries that *any* tactical RPG built on Aegis can reuse (the 6 base classes, stat budgets, proficiency system, baseline status effects, 2D tile grid, A* pathfinding, inventory/stash, quest state machines).
3. **Game Layer (`games/emberwake/`):** Setting-specific world content, specialized narrative mechanics (the Corruption Economy, Strain Gauge, Calcification Clock, Marrow Grafts), unique campaign loops, and custom UI themes.

---

## 2. System-by-System Reuse & Gap Analysis

```text
+---------------------------------------------------------------------------------------------------+
|                                      SYSTEM ARCHITECTURE MATRIX                                   |
+------------------------------------+--------------------------------+-----------------------------+
| FULLY REUSABLE (RPG KIT / ENGINE)  | EXTENSIONS NEEDED (HYBRID)     | BRAND NEW SYSTEMS TO BUILD  |
+------------------------------------+--------------------------------+-----------------------------+
| • 2D Spatial Grid & A* Paths       | • Simultaneous 6-Tick Resolver | • In-Combat Strain Gauge    |
| • 6 Base Character Classes         | • Maneuver Blocks (3-Phase)    | • Calcification Permadeath  |
| • 6-Attribute Stat Curve (STR..KNOW)| • Stagger Bar & Interrupt Engine| • Aberration Crisis Engine |
| • Item & Equipment Proficiencies   | • Composite Move-Then-Act Step | • Marrow Graft Data/Surgeries|
| • Baseline Statuses (Bleed, Root..) | • Auto-Chase Rules (Reach 1/2) | • Timeline Scrubbing & Ghost|
| • Save/Load Envelope & Migrations  | • Timeline UI Widget Extension | • Threat Decals & Tethers   |
| • Dialogue Runner & Node Services  | • Safehouse Hub Facilities     | • Eldritch Dynamic Hazards  |
| • World Travel & Supply Tracking   | • Hit-Stop Micro-Pause Visuals |                             |
+------------------------------------+--------------------------------+-----------------------------+
```

---

## 3. Tier 1: What Can Be Reused Directly

### 3.1 2D Spatial Grid & Pathfinding (`emberwake_battle::grid`, `path`)
* **Existing Architecture:**
  - `Grid`: 2D discrete coordinate system with bounds checking, obstacle masking, and distance calculations.
  - `Path`: A* search with movement cost modifiers per terrain type, avoiding unit collisions.
  - Tactical map sizes ($12 \times 8$, $12 \times 9$, and narrow $10 \times 6$ chokepoints) are fully supported.
* **Reuse Assessment:** **100% Reusable.** The spatial foundation for positioning, distance checking, and tile occupation requires zero structural changes.

### 3.2 The Six Base Classes & Attribute System (`emberwake_data`, `emberwake_rules`)
* **Existing Architecture:**
  - 6 balanced archetypes: `warrior`, `ranger`, `rogue`, `brawler`, `priest`, `mage`.
  - Equal baseline stat budget ($62$ points) and growth ($5$ points/level) distributed across:
    `STR` (Strength), `DEX` (Dexterity), `CONS` (Constitution), `INT` (Intelligence), `KNOW` (Knowledge), `PERC` (Perception).
  - Proficiencies: Item families mapped to proficiency ranks per class.
* **Reuse Assessment:** **100% Reusable.**
  - The new setting archetypes map 1:1 onto the engine classes:
    `warrior` $\to$ The Vanguard, `rogue` $\to$ The Gutter-Stalker, `ranger` $\to$ The Arbalestier, `brawler` $\to$ The Slag-Brute, `priest` $\to$ The Heretic Acolyte, `mage` $\to$ The Ichor Alchemist.

### 3.3 Save Framework, Game Loop & Progression (`aegis_save`, `emberwake_world`)
* **Existing Architecture:**
  - `GameState`: World flags (`Dict`), active location, time counter, safe rest locations.
  - `PartyState`: Gold, supplies, shared party stash, active member selection (max active cap).
  - `UnitState`: XP, stat spend pools, learned skills, passives, loadout equipment slots.
  - `DialogueRunner`: Branching conversation graphs, conditional choices, shop integration, and quest turn-ins.
* **Reuse Assessment:** **100% Reusable.** Provides the exact plumbing needed for saving campaign progression, managing the safehouse stash, and dialogue interactions.

---

## 4. Tier 2: Existing Systems That Need Extension

### 4.1 Battle Resolver: From Sequential Blocks to Maneuver Phases
* **Current Engine State:**
  - `emberwake_battle::resolver::Resolver` schedules actions across blocks ($B_1 \dots B_{\text{total}}$). Actions pay AP and enter stance on the start block, then execute damage and effects on their final block.
* **Extension Required:**
  - Refactor actions into **Maneuver Blocks** with three distinct sub-phases:
    1. **Windup Phase (Ticks 1–2):** Action is telegraphed; unit's Stagger resistance is reduced; action can be cancelled by incoming Stagger interrupts.
    2. **Impact Phase (Tick 2 for Snap, Tick 4 for Standard, Ticks 5–6 for Heavy):** Damage, armor shred, knockback, and statuses apply instantaneously.
    3. **Recovery Phase (Ticks 5–6):** Unit balance reset; converts remaining unused ticks into Active Guard or Strain Venting.

### 4.2 Stagger Bar & The Interrupt Rule
* **Current Engine State:**
  - `stagger` and `stun` exist as generic turn debuffs in `emberwake_rules::status`.
* **Extension Required:**
  - Implement a real-time **Stagger / Poise Bar** for every active combatant, derived from Constitution, armor weight, and active Stance.
  - When blunt damage fills the Stagger Bar during an enemy's **Windup Ticks**:
    - Trigger **Timeline Shatter**: The queued action collapses immediately.
    - Active ticks convert into an amber **Stagger Block**.
    - Target's guard drops, active evasion is reduced to 0%, and their round is forfeit.

### 4.3 Movement Integration: Advance & Disengage
* **Current Engine State:**
  - `PlanStep` separates `Mode::Move` and `Mode::Engage` into distinct steps.
* **Extension Required:**
  - Support **Composite Maneuvers**:
    - **Advance (Move-Then-Act):** Ticks 1–2 spent moving 1–2 tiles, followed immediately by Ticks 3–5 windup and strike.
    - **Disengage (Act-Then-Move):** Fast Snap attack on Tick 2, followed by Ticks 3–4 tactical retreat.
  - **Auto-Chase Logic:**
    - Reach 1 (Daggers/Fists): Tenacious Chase (steps forward 1 tile if target retreats during windup).
    - Reach 2 (Spears/Halberds): Range Hold (hits if target remains in Reach 2).
    - Ranged: Arc Hold (rotates facing toward moving target unless total cover intervenes).

### 4.4 Timeline UI Widget (`aegis_ui::timeline::Timeline`)
* **Current Engine State:**
  - `Timeline` draws a ruler of blocks with a playhead and track rows with segments.
* **Extension Required:**
  - Add **Timeline Playhead Scrubbing**: Allows the player to drag the playhead across Ticks 1–6 (or use keys `1`–`6` / scroll wheel) to inspect predicted unit positions frame-by-frame.
  - Display Speed Tier color codings (Snap, Standard, Heavy) and interrupted crack overlays.

---

## 5. Tier 3: Brand New Systems to Build

### 5.1 The Corruption Economy: In-Combat Strain System
* **Domain:** `emberwake_battle` & `emberwake_rules`
* **Components to Build:**
  - `Unit.strain`: Integer from `0` to `100`.
  - `StrainBracket` state calculator:
    - **0%–24% (Dormant):** Standard performance.
    - **25%–49% (Acute Resonance):** $+15\%$ Critical Chance, $+1$ Movement Tile, $-25\%$ Healing Draught efficacy.
    - **50%–74% (Flesh-Tension):** $+25\%$ Damage, $+2$ Armor Rating, $-20$ Stance Stability (takes double Stagger).
    - **75%–99% (Aberrant Edge):** Bonus Bleed/Stagger on hit, $10\%$ recoil self-damage.
    - **100% (Rupture):** Biological breakdown.
  - **Aberration Crisis Engine:** Triggers at 100% Strain with a D3 event roll:
    1. *Unchecked Spasm:* Out-of-turn strike hitting nearest entity (ally/foe), followed by 1 round of Stun.
    2. *Biological Collapse:* AP drops to 0, internal Bleed, strips active armor mitigation.
    3. *Flesh Breach:* Adds $+1$ permanent point of Calcification immediately.
  - **Dormant Strain Floor:** Tracks permanent minimum baseline strain (e.g., $10\%$ or $30\%$) imparted by aberrant grafts.

### 5.2 Out-of-Combat Calcification (The Permadeath Clock)
* **Domain:** `emberwake_world::state` (`UnitState`)
* **Components to Build:**
  - `UnitState.calcification`: Integer from `0` to `5`.
  - **Post-Battle Evaluation Hook:** Automatically advances Calcification if an operative ends battle $\ge 75\%$ Strain or suffers a Flesh Breach.
  - **Stage Mutation Modifiers:**
    - *Stage I (Vein-Blackening):* $-5$ Max Composure, $+5\%$ Debuff Resistance.
    - *Stage II (Calloused Chitin):* $+1$ Permanent Armor, $-1$ Movement Tile.
    - *Stage III (Sensory Rot):* Fear immunity, $-15\%$ ranged accuracy beyond 3 tiles.
    - *Stage IV (The Shakes):* 2-handed weapon slot lock, natural armor-piercing on melee strikes.
    - *Stage V (Terminal Drift):* **Permanent Roster Loss** (unit retreats to catacombs or transforms into a hostile encounter).

### 5.3 Marrow Grafting & Hub Surgeries
* **Domain:** `emberwake_data` & `emberwake_world`
* **Components to Build:**
  - `GraftDef` catalog: Harvested organs from aberrations/bosses with stat modifications, Strain generation profiles, and heretical skill unlocks.
  - `UnitState.grafts`: Array of installed aberrant organs.
  - **The Apothecary's Slab Interface:**
    - Graft installation and extraction.
    - Strain chemical flushes (lowers temporary dormant strain).
    - Calcification Cleansing (reverses 1 stage of Calcification at the cost of $-10\%$ permanent Max HP).
    - Tincture of Still Blood crafting (locks Strain Gauge for 1 round).

### 5.4 Ghost Projections & Threat Telemetry Decals
* **Domain:** `aegis_ui` & `emberwake_gui`
* **Components to Build:**
  - **Ghost Projection System:** A forward-simulating dry-run evaluator that computes where every combatant will be at tick $T$ without mutating real game state, rendering translucent silhouettes with accurate windup/impact poses.
  - **Decal Renderer:**
    - Ground-locked danger zone outlines with impact tick badges (`[ ⚠ T4 ]`).
    - Entity-locked targeting reticles connected by pulsing bezier threat tethers.
    - Decal Dissolution: Instant particle dispersion upon interrupt.
  - **Hit-Stop Micro-Pause:** $30\text{–}60\text{ ms}$ rendering freeze and anvil sound effect on successful stagger or critical hit.

### 5.5 Dynamic Eldritch Hazards
* **Domain:** `emberwake_battle::grid` & `resolver`
* **Components to Build:**
  - `tar_bog`: Ground liquefaction hazard (slows movement, highly flammable).
  - `inquisition_searchlight`: Sweeping beam hazard (reveals stealth, grants enemies $+20\%$ hit chance).
  - `warded_tallow_smoke` vs `blight_mist`: Smoke blocks ranged targeting and dispels mist; mist ticks $+5\%$ Strain per turn.
  - `molten_slag_vent`: Timed environmental hazard erupting on specific timeline ticks (e.g. Tick 5).

---

## 6. Implementation Roadmap: Proposed Phasing

```text
PHASE 1: Core Combat & Timeline Upgrade (The 6-Tick Engine)
├── Refactor Resolver into Maneuver Blocks (Windup, Impact, Recovery)
├── Implement Speed Tiers (Snap, Standard, Heavy) & Advance/Disengage
└── Add Poise / Stagger Bar and Timeline Shatter Interrupts

PHASE 2: The Corruption Economy (Strain & Calcification)
├── Integrate Unit Strain Gauge (0%–100%) and Bracket Modifiers
├── Build Aberration Crisis Roll Table (Unchecked Spasm, Collapse, Breach)
└── Implement UnitState Calcification Clock (Stages I–V) and Stage V Permadeath

PHASE 3: UI/UX Telemetry & Ghost Scrubbing
├── Implement Timeline Scrubbing (Keys 1–6, mouse scrub, spacebar dry-run)
├── Build Ghost Projection forward-simulation renderer
├── Render Ground Impact Badges ([ ⚠ T4 ]) and Bezier Threat Tethers
└── Add 30–60ms Hit-Stop and Decal Dissolution VFX

PHASE 4: Heretical Grafts & Safehouse Hub Facilities
├── Author GraftDef catalog and Inoculated skill tree branches
├── Build The Sluice Undercroft safehouse screen (Apothecary Slab, Tinker's Hearth, Water-Lock)
└── Implement Surgical Triage, Strain Flushes, and Calcification Cleansing
```
