# UI Architecture: Screen Breakdown & Exploration/Combat Separation Plan

> **Category:** UI/UX Architecture & Navigation Framework  
> **Location:** `games/emberwake/lore/systems/screen_architecture_and_separation_plan.md`  
> **Target Crates:** `aegis_ui`, `emberwake_gui`, `emberwake_world`  
> **Related Documents:** [gameplay_loop_and_ui_prototype.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/gameplay_loop_and_ui_prototype.md), [rules_engine_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/rules_engine_architecture.md)

---

## 1. Plan for Separating Combat UI from Exploration UI

In *Emberwake*, tactical combat and district infiltration require completely different cognitive spaces:
* **The Exploration UI (`RegionMap` / `DistrictMap`):** Macroscopic, tense, atmospheric, spatial navigation. The player manages lantern stealth cones, safehouse alert levels, street chokepoints, and resource caches.
* **The Combat UI (`BattleScreen`):** Microscopic, deterministic, simultaneous. The player manages the 6-tick timeline, telegraph scrub slider, speed tier interrupts, AP/Strain gauges, and poise meters.

The separation is achieved cleanly through the engine's native **Screen Stack Architecture** (`aegis_ui::screen::Screen` and `cx.nav`):

```text
                                  [ THE ENGINE SCREEN STACK ]
                                                |
                 +------------------------------+------------------------------+
                 |                                                             |
                 v                                                             v
     [ BASE EXPLORATION SCREEN ]                                  [ FOCUSED COMBAT SCREEN ]
       `RegionMap` / `DistrictMap`                                  `BattleScreen`
     - Node graph & lantern patrols                               - 6-Tick multi-track timeline
     - Fog of war & street chokepoints                            - 32x32 tactical grid & positioning
     - Safehouse alert & Red Tithing clock                        - Real-time Poise & Strain gauges
     - Event log & node inspection                                - Skill deck execution & scrubbing
                 |                                                             ^
                 | (Trigger: Encounter on Node / Ambush)                       |
                 +-----------------> cx.nav.push(Nav::Push(BattleScreen)) -----+
                 |
                 | (Combat Ends: Victory / Defeat / Retreat)
                 +<----------------- cx.nav.push(Nav::Pop) --------------------+
                 |
                 v (Returns to Exploration & Pops Aftermath Modal)
     [ BattleResults Modal ] (Loot, Organ Harvest, Calcification Advances)
```

### 1.1 State Boundary & Decoupling
1. **Zero Leaked Render State:** `BattleScreen` and `DistrictMap` have completely separate draw loops and widget trees (`Ui<Action>`). Combat widgets are destroyed when popped from the stack.
2. **Persistent vs. Transient Data:**
   - **`GameState` (Persistent in `GameCx`):** Operative roster, current bunks, inventory, campaign clocks, safehouse facility levels.
   - **`CombatSession` (Transient in `BattleScreen`):** Active 6-tick resolver, poise values, in-combat strain percentage, hostiles, planned maneuver blocks.
3. **The Outcome Handoff (`settle_outcome`):**
   When `BattleScreen` finishes, it writes the combat result into a clean payload:
   - Final operative HP and remaining Strain.
   - Harvested Aberrant Grafts and salvaged black iron slag.
   - Any permanent Calcification advances triggered by Stage V or Crisis rolls.
   - The stack pops `BattleScreen`, returns to `DistrictMap`, and presents the `BattleResults` modal.

---

## 2. Complete Screen Breakdown: Existing vs. To-Be-Built

The table below catalogs every screen needed for *Emberwake*, highlighting what is reused from the existing build and what requires new development:

```text
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| SCREEN NAME       | EXISTING ASSET / FOUNDATION       | WHAT CAN BE LEVERAGED             | WHAT REQUIRES     |
|                   |                                   | (EXISTING CODE REUSED)            | BUILDING OUT      |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 1. BattleScreen   | `emberwake_gui::battle`           | • 32x32 tactical tile grid        | • Bind Timeline   |
|    (Combat UI)    | `aegis_ui::timeline`              | • Unit sprite depth-sorting       |   to 6 ticks      |
|                   |                                   | • Movement reach & target rings   | • Add Poise Bar & |
|                   |                                   | • AP cost calculator & hit log    |   Strain Gauge    |
|                   |                                   | • Timeline tracks & playhead      | • Scrub preview   |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 2. DistrictMap    | `emberwake_gui::region::RegionMap`| • Node graph rendering & travel   | • Sunk Ward art & |
|    (Exploration)  | `emberwake_world::explore`        | • Location markers & hover cards  |   cobblestone map |
|                   |                                   | • Exploration event log stream    | • Lantern patrol  |
|                   |                                   | • Encounter trigger plumbing      |   sight cones     |
|                   |                                   | • Party position tracking         | • Alert meter     |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 3. Safehouse Hub  | `emberwake_gui::hub::Hub`         | • Hub door navigation structure   | • 6-8 bunk status |
|    (The Sluice    | `emberwake_world::state`          | • Save/load modal integrations    |   overview card   |
|     Undercroft)   |                                   | • Party & Journal door hooks      | • Daily Salt and  |
|                   |                                   | • Background frame rendering      |   Tallow burn UI  |
|                   |                                   |                                   | • Facility doors  |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 4. Party & Deck   | `emberwake_gui::party::PartyScreen`• 4-tab tabbed modal architecture  | • Constrain to    |
|    Loadout Modal  | `emberwake_world::state`          | • Stats, Level Up, Skills, Gear   |   Active Deck (3A |
|                   |                                   | • Inventory & weapon slotting     |   + 2P + 1K)      |
|                   |                                   | • Drag-and-drop skill slotting    | • Calcification & |
|                   |                                   | • Attribute modification formulas |   Strain Floor UI |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 5. Apothecary     | `emberwake_gui::forge::ForgeScreen`• Recipe list & material check     | • Organ harvest   |
|    Slab Modal     | `emberwake_world::shop`           | • Item requirement validation     |   graft installer |
|    (Sister Aline) |                                   | • Confirmation popups & sounds    | • Strain floor    |
|                   |                                   |                                   |   elevation preview|
|                   |                                   |                                   | • Surgical triage |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 6. Tinker's Hearth| `emberwake_gui::forge::ForgeScreen`• Crafting recipe selection        | • Black iron slag |
|    Modal (Bram)   |                                   | • Gear upgrade confirmations      |   hardening stats |
|                   |                                   | • Material inventory subtraction  | • Bunk expansion  |
|                   |                                   |                                   |   project tiers   |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 7. BattleResults  | `emberwake_gui::hub`              | • Victory/defeat banner display   | • Aberrant Organ  |
|    Popup Modal    | (`battle_results_model.gd`)       | • XP & standard item reward lists |   harvest tallies |
|                   |                                   | • Continue / Return button hook   | • Calcification & |
|                   |                                   |                                   |   Crisis summary  |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
| 8. Main Menu &    | `emberwake_gui::menus`            | • Menu button stack (New/Load)    | • 30-second comic |
|    Prologue Intro | `aegis_ui::painter`               | • Settings modal & slot picker    |   vignette cards  |
|                   |                                   | • Resolution & audio options      | • Eldritch UI     |
|                   |                                   |                                   |   palette reskin  |
+-------------------+-----------------------------------+-----------------------------------+-------------------+
```

---

## 3. Deep Dive into Screen Specifications

### 3.1 Screen 1: The Tactical Battle Screen (`BattleScreen`)
* **File Base:** Reuses `games/emberwake/crates/emberwake_gui/src/battle.rs`.
* **State Management:** Operates on an isolated `emberwake_battle::session::Session`.
* **Key Visual Elements:**
  - **Grid Area:** Renders the $32 \times 32$ isometric grid, movement rings, and target cursor (100% existing code).
  - **Top Banner:** Red Tithing countdown and Safehouse Alert level.
  - **Party Strip:** 4 hero cards with HP, AP, **Poise Bar** (new), and **Strain Gauge** (new).
  - **Bottom Ruler & Tracks (`aegis_ui::timeline`):**
    - 6 discrete blocks ($T_1 \dots T_6$).
    - 4 party tracks + hostile tracks.
    - Maneuver segments with color-coded phases:
      - Amber hatching (`dashed: true`) $\longrightarrow$ Windup.
      - Crimson solid diamond $\longrightarrow$ Impact Tick.
      - Muted Slate bar $\longrightarrow$ Recovery.
      - Gold solid bar $\longrightarrow$ Active Guard.
* **Timeline Scrubbing:** Scrubbing the timeline slider previews combatants' locations and poise states at that exact tick, enabling the player to identify interrupt opportunities before hitting **[EXECUTE]**.

---

### 3.2 Screen 2: The District Exploration Map (`DistrictMap` / `RegionMap`)
* **File Base:** Reuses `games/emberwake/crates/emberwake_gui/src/region.rs`.
* **Role:** Overworld navigation through *The Sunk Ward*.
* **Key Visual Elements:**
  - **Cobblestone Street Grid:** A network of interconnected nodes representing streets, alleyways, flooded sluice gates, and tallow refineries.
  - **Inquisition Lantern Patrols (New):** Certain nodes feature moving patrol units with triangular vision cones. Stepping into a lantern cone immediately triggers an Inquisitor ambush fight.
  - **Event Log Stream (Existing):** Displays narrative text: *"The cell slips past the Cathedral bells... Rain water fills the gutter."*
  - **District Header:** Shows current Tallow reserves, Salt rations, and Inquisition Alert rating ($0\% \dots 100\%$).
* **Transition to Combat:** Calling `cx.nav.push(Nav::Push(Box::new(battle)))` suspends the exploration map and transitions smoothly into tactical combat.

---

### 3.3 Screen 3: The Safehouse Undercroft (`SafehouseHub`)
* **File Base:** Reuses `games/emberwake/crates/emberwake_gui/src/hub.rs`.
* **Role:** The nerve center of The Severed Root beneath Lower Cinderreach.
* **Key Visual Elements:**
  - **The Hearth Overview:** Shows the furnace fire, water level in the storm-drain, and daily burn rates:
    $$\text{Daily Burn} = 1 \text{ Salt Ration} + 1 \text{ Warded Tallow Candle}$$
  - **The Bunk Roster Panel:** Displays the 6 (or 8) bunks with operative status cards:
    - *Ready / Active* (Deployed on next mission)
    - *Resting / Bandaged* (Recovering from surgical graft or wound)
    - *Living Martyr* (Stage IV Calcification, temporary power-up shock troop)
  - **Facility Doors:**
    - `[TINKER'S HEARTH]` $\longrightarrow$ Opens Bram's Forge modal.
    - `[APOTHECARY SLAB]` $\longrightarrow$ Opens Aline's Surgery modal.
    - `[WATER-LOCK GATE]` $\longrightarrow$ Opens DistrictMap for deployment.

---

### 3.4 Screen 4: Operative Deck & Character Screen (`PartyScreen`)
* **File Base:** Reuses `games/emberwake/crates/emberwake_gui/src/party.rs`.
* **Modal Nature:** Implements `overlay() -> true`. Can be opened from the Safehouse Hub or during Exploration (to review loadouts).
* **Tab Breakdown:**
  1. **Tab 1: Stats:** Displays base attributes (`STR`, `DEX`, `CON`, `INT`, `KNOW`, `PERC`), permanent **Calcification Stage (I–V)**, and **Dormant Strain Floor**.
  2. **Tab 2: Maneuver Deck (Skills):** Replaces legacy open-skill slotting with the **Active Deck**:
     - Slot 1–3: Active Class Maneuvers (Cold Steel & Inoculated).
     - Slot 4–5: Passive Traits.
     - Slot 6: Keystone Perk.
  3. **Tab 3: Gear:** Mainhand, offhand, armor, and trinket slots (reused 100%).
  4. **Tab 4: Level Up:** Spends level-up points at Bram's forge.

---

### 3.5 Screen 5: The Apothecary's Slab (`SurgeryScreen`)
* **File Base:** Adapted from `emberwake_gui::forge::ForgeScreen`.
* **Role:** Sister Aline's heretical medical theater.
* **Key Visual Elements:**
  - **Operative Selector:** Choose the target patient (shows their current Calcification stage and Composure).
  - **Harvested Reagent Catalog:** Displays collected Aberrant Organs (e.g., *Stalker Optic Nerve*, *Titan Marrow Core*).
  - **Graft Preview Window:**
    - Shows the Inoculated skill that will be unlocked.
    - Shows the **Permanent Strain Floor Increase** (e.g. $+15\%$).
    - Shows the risk of triggering Stage advancing trauma.
  - **Confirm Surgery Button:** Plays a wet, visceral audio cue and binds the heretical graft to the operative.

---

### 3.6 Screen 6: Combat Results & Triage Popup (`BattleResults`)
* **File Base:** Reuses `emberwake_gui::hub` (`battle_results_model.gd`).
* **Modal Nature:** Appears as an overlay popup immediately after `BattleScreen` pops from the stack.
* **Key Visual Elements:**
  - **Encounter Banner:** *"Inquisition Patrol Shattered"* or *"Tactical Withdrawal Achieved"*.
  - **Casualty & Strain Report:** Summarizes operative HP lost, current Strain levels, and flags if any operative advanced a Calcification stage.
  - **Aberrant Harvest Section (New):** Lists newly harvested organ grafts and warded tallow vials recovered from hostile corpses.
  - **[RETURN TO EXPLORATION] / [RETURN TO SAFEHOUSE] Button.**

---

## 4. Prototype Phasing: What to Build First

To maintain rapid development velocity and ensure empirical testing of mechanics, screen implementation follows three distinct phases:

```text
PHASE 1: THE TACTICAL COMBAT SANDBOX (Dojo Test Arena)
- Focus: Screen 1 (BattleScreen) + aegis_ui::timeline integration.
- Deliverable: Load 1-2 operatives vs. Inquisitors in a bare arena.
- Tests: Timeline scrub, Snap interrupts, Active Guard mitigation, Strain surfing.
- Zero exploration or savegame overhead needed.
                          |
                          v
PHASE 2: SAFEHOUSE & LOADOUT DECK INTEGRATION
- Focus: Screen 3 (Safehouse Hub) + Screen 4 (PartyScreen Active Deck).
- Deliverable: Selecting 4-operative cells and building 3-Active decks.
- Tests: Bunk limitations (6 to 8) and Strain floor tracking.
                          |
                          v
PHASE 3: DISTRICT EXPLORATION & FULL MACRO LOOP
- Focus: Screen 2 (DistrictMap) + Screen 6 (BattleResults popup).
- Deliverable: Moving across Sunk Ward street nodes, dodging lantern cones,
  entering combat on encounter nodes, and returning with harvested organ loot.
```
