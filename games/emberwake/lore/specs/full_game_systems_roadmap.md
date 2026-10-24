# Full Game Systems Roadmap: Post-Vertical Slice & Late-Game Architecture

> **Category:** Long-Term Systems Architecture & Campaign Pacing  
> **Status:** Active Canonical Specification & Systems Roadmap  
> **Location:** `games/emberwake/lore/specs/full_game_systems_roadmap.md`  
> **Reference Specs:** [campaign_narrative_structure.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/campaign_narrative_structure.md), [screen_architecture_and_separation_plan.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/screen_architecture_and_separation_plan.md), [safehouse_economy_and_triage.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/safehouse_economy_and_triage.md)

---

## 1. Executive Summary & Readiness Verification

This document establishes two architectural objectives:
1. **Vertical Slice Sign-Off:** Formally verifies that 100% of the foundational specifications required for the playable Phase 1 vertical slice are complete, mathematically grounded, and aligned with the existing codebase.
2. **Late-Game Systems Roadmap & Placeholder Architectures:** Inventories and defines the placeholder data schemas and mechanics for systems introduced in **Phase 2 (The Transhuman Rupture)**, **Phase 3 (The Awakening Abyss)**, and the **Endgame Climax**. 

While these late-game systems do not need narrative fleshing out today, their structural hooks must be defined now so they can be implemented rapidly and cleanly without refactoring the core engine.

---

## 2. Phase 1 Vertical Slice Readiness Matrix

Every component required to build, test, and balance the playable slice is fully specified:

```text
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| COMPONENT DOMAIN      | SUBSYSTEM SPECIFIED         | CANONICAL REFERENCE DOCUMENT       | ENGINE TARGET CRATE      |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 1. Tactical Combat    | 6-Tick Simultaneous Timeline| [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md)  | `emberwake_battle`       |
|                       | Speed Tiers (Snap/Std/Hvy)  |                                    |                          |
|                       | Poise & Stagger Interrupts  |                                    |                          |
|                       | Simultaneous Tie-Breakers   |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 2. Rules & Damage     | Hyperbolic Armor & Piercing | [rules_engine_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/rules_engine_architecture.md)| `emberwake_rules`        |
|                       | Real-Time Strain Surfing    |                                    |                          |
|                       | Aberration Crisis Engine    |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 3. Classes & Roster   | 6 Base Class Trees (A & B)  | [classes_and_skill_trees.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/classes_and_skill_trees.md)  | `emberwake_data`         |
|                       | 6 Founders: Traits & Caps   |                                    |                          |
|                       | Active Deck (3A + 2P + 1K)  |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 4. Enemy Bestiary     | 4 Factions (Base & Elite)   | [enemy_bestiary_and_ai.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/enemy_bestiary_and_ai.md)    | `emberwake_data`,        |
|                       | 5 Named Boss Specializations|                                    | `emberwake_battle`       |
|                       | Modular Encounters (01–06)  |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 5. Campaign & Overworld| Sunk Ward 8-Node Graph      | [sunk_ward_campaign_map.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/sunk_ward_campaign_map.md)| `emberwake_world`        |
|                       | Stealth/Speed Travel Rules  |                                    |                          |
|                       | Inquisition Alert Meter     |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 6. Safehouse Economy  | Sluice Undercroft (4 Wings) | [safehouse_economy_and_triage.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/safehouse_economy_and_triage.md)| `emberwake_world`,       |
|                       | Bunks, Tallow, Salt, Scrap  |                                    | `emberwake_gui`          |
|                       | UI Reuse & New Widgets      |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
| 7. Visual Design      | 640x360 Integer Scaling     | [art_and_visual_style.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/art_and_visual_style.md)        | `aegis_ui`,              |
|                       | 16-Color Master Palette     |                                    | `emberwake_gui`          |
|                       | Telemetry Decal Language    |                                    |                          |
+-----------------------+-----------------------------+------------------------------------+--------------------------+
```

---

## 3. Post-Vertical Slice: Full Game Systems Roadmap

The following table traces the game systems that enter the campaign after the Phase 1 Crucible climax:

```text
                                [ CAMPAIGN SYSTEMS LIFECYCLE ]
                                              │
             ┌────────────────────────────────┼────────────────────────────────┐
             ▼                                ▼                                ▼
    [ PHASE 1: YOUNG VANGUARD ]      [ PHASE 2: TRANSHUMAN RUPTURE ]   [ PHASE 3: AWAKENING ABYSS ]
    (The Vertical Slice)             • Procedural Youth Recruits       • Continental Resonance
    • Pure Cold Steel                • Full Inoculation & Grafts       • Tallow Blackouts & Crypts
    • 6 Authored Founders            • Living Martyr Deployments       • Safehouse Siege Defense
    • Sunk Ward Infiltration         • Flesh-Broker vs Preservationist • Red Tithing Endgame Clock
```

---

## 4. Phase 2 Placeholder Systems Architecture

With the scripted sacrifice at the Crucible Watchpost ([ENC-EVT-06](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/sunk_ward_campaign_map.md#L90-L105)) and the unlocking of Sister Aline's **Apothecary Slab**, Phase 2 introduces four core systems:

### 4.1 System P2-01: Procedural Youth Recruitment Engine
* **Narrative Context:** With founding heroes subject to injury, trauma, and permanent Calcification, the cell liberates condemned youths from Inquisition transport wagons to replenish safehouse bunks.
* **Target Crate:** `emberwake_world::recruitment`
* **Mechanics:** Rescued youths roll random attributes, psychological quirks, and class aptitudes:

```rust
// Proposed placeholder schema in emberwake_world::recruitment:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralRecruit {
    pub name: String,
    pub age: u8, // Clamped between 15 and 24 ("The Unsealed")
    pub class_aptitude: String, // "vanguard", "stalker", etc.
    pub personal_quirk: RecruitQuirk,
    pub baseline_composure: i32,
    pub starting_calcification: u8, // Usually 0, occasionally 1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecruitQuirk {
    Stoic(i32),         // +Poise bonus
    MarrowSensitive,    // Gains +10% damage when Inoculated, but +5% Strain
    TenementCourier,    // +1 Movement tile on round 1
    CoughingBile,       // Takes -10% healing from draughts
}
```

---

### 4.2 System P2-02: Apex Grafting & Surgical Excision
* **Narrative Context:** Elite aberrations and inquisitor officers yield unique, non-standard anatomical organs (e.g. *Chitinous Heart*, *Void-Gland*, *Colossus Sinew*).
* **Target Crate:** `emberwake_rules::grafts`
* **Mechanics:**
  - Operatives possess exactly **4 Graft Slots** (Cranial, Thoracic, Forearms, Visceral).
  - Installing an Apex Graft imparts game-changing passives or active skills, but permanently ratchets up the hero's **Dormant Strain Floor**.
  - **Surgical Excision:** Sister Aline can cut out an installed graft, but the surgical trauma permanently reduces the hero's **Max HP by -15%** and leaves permanent tissue scarring.

---

### 4.3 System P2-03: Living Martyr Deployment System
* **Narrative Context:** When an operative reaches **Stage IV Calcification ("The Shakes")**, their skeletal ossification is irreversible. Before they petrify into stone or lose their minds, the player can authorize a final **Martyr Strike**.
* **Target Crate:** `emberwake_world::state` & `emberwake_battle::session`
* **Mechanics:**
  - The hero is deployed for **one final mission**.
  - In combat, they gain permanent **Aberrant Trance**: immunity to crowd control, $+100\%$ true damage, and all skills cost 0 AP.
  - At the conclusion of the mission, the hero collapses and permanently petrifies. Their body crystallizes into an **Authored Apex Relic** that can be equipped by surviving squad members.

---

### 4.4 System P2-04: Ideological Alignment (Flesh-Broker vs. Preservationist)
* **Narrative Context:** The cell fractures over doctrine. Sister Aline advocates embracing corruption to match church power (*The Flesh-Brokers*); Bram Karr demands preserving human dignity through cold steel (*The Preservationists*).
* **Target Crate:** `emberwake_world::reputation`
* **Mechanics:**
  - A dynamic reputation slider tracks player choices:
    $$\text{Alignment} = \sum (\text{Grafts Installed} + \text{High Strain Battles}) - \sum (\text{Forge Upgrades} + \text{Salt Flushes})$$
  - Unlocks exclusive Safehouse facilities, unique dialog branches, and alters the campaign ending.

---

## 5. Phase 3 Placeholder Systems Architecture

In Phase 3, the slumbering cosmic titan (**The Low Sleep**) begins to stir, causing corruption to leak directly into safehouse survival and overworld exploration:

### 5.1 System P3-01: Continental Resonance & Bell Tremor Engine
* **Narrative Context:** Cathedral bells toll across the city in harmonic sympathy with the titan's subterranean tremors.
* **Target Crate:** `emberwake_world::travel` & `emberwake_battle::resolver`
* **Mechanics:**
  - A world clock tracks the bell cycles.
  - When bells toll, any operative with $\ge 1$ point of Calcification feels their bones vibrate:
    - *During Overworld Travel:* Accumulates $+10\%$ passive Strain before entering battle.
    - *In Combat:* Generates a battlefield-wide pulse dealing 10 true damage to all units with $\ge 50\%$ Strain.

---

### 5.2 System P3-02: Citywide Tallow Blackout & Sanity Crypts
* **Narrative Context:** The Tallow Guild shuts down surface supply lines, causing catastrophic fuel shortages.
* **Target Crate:** `emberwake_world::safehouse`
* **Mechanics:**
  - Daily warded tallow consumption doubles.
  - If tallow reserves hit 0, the safehouse flame dies. Sleeping operatives suffer **Nightmare Psychosis Rolls** (`D100 vs Composure`), developing afflictions: *Paranoid Insomnia*, *Terrified Claustrophobia*, or *Violent Sleepwalking*.
  - Players must allocate Stage IV heroes to sealed **Isolation Crypts** in the Dry Hollows to prevent psychological panic from spreading to young recruits.

---

### 5.3 System P3-03: Safehouse Siege Incursion (Alert 100%)
* **Narrative Context:** If the cell operates with reckless brutality, the Inquisition locates the Sluice Undercroft.
* **Target Crate:** `emberwake_world::crisis`
* **Mechanics:**
  - When the citywide Inquisition Alert Meter reaches $100\%$, an emergency siege triggers immediately upon returning to the safehouse.
  - Tactical battle takes place inside the **Sluice Undercroft Central Basin**.
  - Hostile purge squads attack Sister Aline's slab and Bram's forge. If an installation is destroyed, its crafting/surgery services are disabled until repaired with Scrap Iron.

---

### 5.4 System P3-04: The Red Tithing Doom Clock
* **Narrative Context:** The grand ritual sacrifice of youth at the High Cathedral approaches.
* **Target Crate:** `emberwake_world::campaign`
* **Mechanics:**
  - A persistent global calendar tracks days remaining until mass sacrifice ($D_0 \dots D_{\text{Tithing}}$).
  - Every mission and rest advances the clock.
  - If the clock hits 0 before the player breaches the Upper Bastion, the campaign ends in **True Campaign Failure (The Great Extinguishing)**.

---

## 6. Endgame Climax & Campaign Ending Engine

```text
                                [ THE THREE CAMPAIGN ENDINGS ]
                                               │
             ┌─────────────────────────────────┼─────────────────────────────────┐
             ▼                                 ▼                                 ▼
    [ ENDING A: PRESERVATION ]        [ ENDING B: ASCENDANCY ]          [ ENDING C: EXTINGUISHED ]
    "The Human Line"                  "The New Leviathans"              "The Sleeper Awakens"
    Sever the Clerisy without         Merge with the Low Sleep titan,   Fail the Doom Clock or Safehouse;
    aberrant corruption; human        overthrowing the church to rule   the cosmic titan shatters
    mortality and dignity prevail.    as transhuman marrow oligarchs.   Cinderreach in cataclysmic ruin.
```

---

## 7. Master System Integration Summary

```text
[ VERTICAL SLICE (READY TO IMPLEMENT) ]
  ├── emberwake_battle: 6-Tick Timeline Resolver, Speed Tiers, Poise Interrupts.
  ├── emberwake_rules: StrainState, PoiseState, Hyperbolic Armor + Piercing.
  ├── emberwake_data: 6 Class Trees (A & B), 6 Founders, 4-Faction Bestiary.
  ├── emberwake_world: Sunk Ward 8-Node Graph, 5 Missions, Safehouse State.
  └── emberwake_gui: Screen Separation, Active Deck Builder, HUD Tickers.

[ POST-SLICE HOOKS (PLACEHOLDERS LOGGED) ]
  ├── Phase 2: ProceduralRecruit, ApexGrafts, LivingMartyr, IdeologicalReputation.
  ├── Phase 3: BellResonance, TallowBlackout, SafehouseSiege, RedTithingClock.
  └── Endgame: MultiDistrictGraph, TitanMacroBoss, TripartiteEndingEngine.
```
