# Skill Acquisition, Combat Loadouts & Engine Integration

> **Category:** Gameplay Systems & Catalog Architecture  
> **Location:** `games/emberwake/lore/systems/skill_acquisition_and_loadout_systems.md`  
> **Reference Lore:** [classes_and_skill_trees.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/classes_and_skill_trees.md), [rpg_kit_and_system_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/rpg_kit_and_system_architecture.md)  
> **Engine Catalog:** [RPG_KIT_CATALOG.md](file:///c:/dev/tactical-rpg/docs/RPG_KIT_CATALOG.md)

---

## 1. Skill Acquisition Architecture: The Dual Pathway

Skill progression mirrors the fundamental dichotomy of the world. An operative acquires abilities through two completely separate operational pathways:

```text
                                     [ SKILL ACQUISITION PATHS ]
                                                  |
                 +--------------------------------+--------------------------------+
                 |                                                                 |
                 v                                                                 v
    [ BRANCH A: COLD STEEL ]                                          [ BRANCH B: THE BITTER INOCULATION ]
    "Physical Conditioning & Martial Lore"                            "Surgical Organ Harvest & Marrow Graft"
    - Unlocked via Leveling & Weapon Proficiencies                    - Locked in Phase 1 (Unlocked Phase 2)
    - Mastered at Bram Karr's Tinker Forge                            - Installed at Sister Aline's Apothecary Slab
    - Zero Strain generation, zero biological risk                    - Generates Strain (+15%–35%) & raises Strain Floor
    - Permanent longevity across campaign                             - Ticks the Calcification Permadeath Clock
```

### 1.1 Acquiring Cold Steel Skills (Branch A)
* **Tier I Skills (Levels 1–3):** Unlocked automatically as an operative gains experience and achieves weapon proficiency thresholds (e.g., reaching Sword Rank 1 unlocks *Pommel Strike*).
* **Tier II Skills (Levels 4–6):** Unlocked by spending skill points earned on level-up. Requires training drills conducted at **The Tinker's Hearth** (Bram Karr).
* **Tier III & Keystones (Levels 7+):** Unlocked after completing Bram's master quest (*"Cold Slag, Black Iron"*). Requires forged masterwork steel equipment to execute.

### 1.2 Acquiring Inoculated Skills (Branch B)
* **Phase 1 Lock:** Inoculated skills are strictly locked during Phase 1. Operatives cannot install grafts until the narrative rupture occurs at the Crucible.
* **The Graft Harvest Loop (Phase 2+):**
  1. **Slay Aberrations / Inquisitors:** Defeating elite enemies yields harvested anatomical reagents (e.g., *Stalker Optic Nerve*, *Titan Marrow Core*, *Caustic Gall-Bladder*).
  2. **Surgical Grafting at The Apothecary's Slab:** The player takes the operative and the harvested reagent to Sister Aline Vance.
  3. **The Installation Surgery:** Aline performs an agonizing surgical graft:
     - The corresponding Inoculated skill is permanently bound to the operative's skill list.
     - The operative's **Dormant Strain Floor permanently rises** (e.g., $+10\%$ or $+30\%$).
     - The operative enters their next mission with fresh surgical bandages.

---

## 2. Combat Loadout Setup (The Active Deck)

In tactical combat, an operative cannot bring every skill they have ever learned. They must prepare an **Active Maneuver Deck** in the Safehouse before deploying:

```text
+---------------------------------------------------------------------------------------------------+
| OPERATIVE LOADOUT DECK STRUCTURE                                                                  |
+-------------------+-----------------------------------+-------------------------------------------+
| SLOT TYPE         | CAPACITY                          | SYSTEM CONSTRAINTS                        |
+-------------------+-----------------------------------+-------------------------------------------+
| 1. Baseline Moves | 2 Fixed Actions (Always Available)| • Move (Speed-based tiles per tick)       |
|                   |                                   | • Active Guard (Timeline damage & poise mitigation) |
+-------------------+-----------------------------------+-------------------------------------------+
| 2. Class Actives  | 3 Active Maneuvers (Equipped)     | • Must meet weapon proficiency rules      |
|                   |                                   | • Can mix Cold Steel and Inoculated       |
+-------------------+-----------------------------------+-------------------------------------------+
| 3. Passive Traits | 2 Passive Slots (Equipped)        | • Dictates baseline Strain floor & buffs  |
+-------------------+-----------------------------------+-------------------------------------------+
| 4. Keystone Perk  | 1 Keystone Slot (Equipped)        | • High-tier game-defining capstone ability|
+-------------------+-----------------------------------+-------------------------------------------+
```

### 2.1 Elimination of Legacy Stances in Favor of Organic Timeline States
Earlier conceptual designs featured a 4-stance machine (`Offensive`, `Defensive`, `Tactical`, `Neutral`). This stance layer was dropped to eliminate cognitive bloat and artificial state toggling:
* **Defense is an Action, Not a Stance:** An operative executes **Active Guard** across specific timeline ticks. During those ticks, physical damage is reduced by $50\%$ and Poise/Stagger defense doubles.
* **Vulnerability is Temporal, Not Modal:** Heavy attacks do not require an "Offensive Stance" penalty; instead, their lengthy **Windup ticks** leave the unit telegraphed and vulnerable to Stagger interrupts.
* **Damage Scaling is Biological:** Hyper-offensive scaling is governed by **Strain Brackets** (e.g., *Flesh-Tension* at 50%–74% Strain provides $+25\%$ damage and $+2$ armor at the cost of double Stagger intake).
* **Speed Replaces "Tactical Stance":** Action tempo is dictated by **Speed Tiers** (*Snap*, *Standard*, *Heavy*). Fast 2-tick actions naturally slip in before heavy swings.

---

## 3. Leveraging the Existing RPG Kit Skills

The engine's RPG kit already possesses **57 authored baseline skills** in `docs/RPG_KIT_CATALOG.md` and `emberwake_data`. Rather than discarding them, we map them directly to the **Cold Steel branches** across all six classes:

```text
+-------------------+-----------------------------+------------------------------------------------------------+
| CLASS ARCHETYPE   | NEW COLD STEEL SKILLS       | REUSED / MAPPED RPG KIT ENGINE SKILLS                      |
+-------------------+-----------------------------+------------------------------------------------------------+
| 1. The Vanguard   | Pommel Strike, Braced Poise,| • Reuses `shield_bash`, `guard`, `sunder`, `rend_armor`,   |
|    (Warrior)      | Intercepting Hook           |   `shield_wall`, `riposte`, `taunt_shout`                  |
+-------------------+-----------------------------+------------------------------------------------------------+
| 2. Gutter-Stalker | Hamstring Sever, Ash-Cloud, | • Reuses `quick_stab`, `hamstring`, `bleeding_cut`,        |
|    (Rogue)        | Throat-Carver's Art         |   `backflip` (disengage), `envenom`, `assassinate`         |
+-------------------+-----------------------------+------------------------------------------------------------+
| 3. Arbalestier    | Notched Quarrel, Snare Trap,| • Reuses `aimed_shot`, `pinning_shot`, `barbed_arrow`,     |
|    (Ranger)       | Heavy Winch Sunder          |   `deadeye`, `leap_back`, `crippling_shot`                 |
+-------------------+-----------------------------+------------------------------------------------------------+
| 4. Slag-Brute     | Jaw-Breaker, Slag Slam,     | • Reuses `body_blow`, `haymaker`, `lunge`, `sweep_kick`,   |
|    (Brawler)      | Foundry Shatter             |   `cyclone_kick`, `crushing_blow`                          |
+-------------------+-----------------------------+------------------------------------------------------------+
| 5. Heretic Acolyte| Brimstone Cauterize, Tonic, | • Reuses `mend`, `greater_mend`, `purify`, `smite`,        |
|    (Priest)       | Resilience Ward             |   `renewal`, `aegis` (ward buff)                           |
+-------------------+-----------------------------+------------------------------------------------------------+
| 6. Ichor Alchemist| Phosphorus Pot, Flash-Bomb, | • Reuses `firebomb`, `arc_lance`, `arcane_bolt`,           |
|    (Mage)         | Suffocating Choke-Gas       |   `glacial_burst`, `inferno`                               |
+-------------------+-----------------------------+------------------------------------------------------------+
```

---

## 4. What Needs to Be Built in `SkillDef` (Data Schema Extensions)

To bridge the existing Rust data structures (`emberwake_data::SkillDef`) with the simultaneous timeline and corruption mechanics, we must add six specialized fields:

```rust
// Proposed extensions to emberwake_data::SkillDef:
pub struct SkillDef {
    // --- EXISTING ENGINE FIELDS REUSED ---
    pub id: String,
    pub name: String,
    pub kind: String,           // "ATTACK", "DEFEND", "SUPPORT", "DISENGAGE", etc.
    pub level: i32,
    pub blocks: i32,            // Maps to timeline tick duration
    pub weapon_types: Vec<String>,
    pub reach_min: i32,
    pub reach_max: i32,
    pub area: String,           // "NONE", "BURST", "LINE", "SWEEP", "PLUS"
    pub power_mult: f64,
    pub status_effect: String,  // "stagger", "bleed", "root", "burn", etc.

    // --- NEW EMBERWAKE WORLD & TIMELINE FIELDS TO BUILD ---
    /// Strain cost generated upon execution (0 for Cold Steel, +15 to +35 for Inoculated).
    pub strain_cost: i32,

    /// Timeline window anchor: "SNAP" (T2 impact), "STANDARD" (T4 impact), "HEAVY" (T5/T6 impact).
    pub speed_tier: String,

    /// Direct posture/poise damage applied to target's real-time Stagger Bar.
    pub stagger_damage: i32,

    /// If true, damage completely bypasses target's Armor Rating (true corrosive/eldritch damage).
    pub armor_piercing: bool,

    /// Required aberrant organ graft ID to unlock (None for Cold Steel).
    pub graft_requirement: Option<String>,

    /// Self-inflicted recoil damage percentage (e.g. 0.10 for 10% recoil in Aberrant Edge).
    pub recoil_pct: f64,
}
```

---

## 5. Summary: Progression Flow for an Operative

$$\text{Martial Drill / Level-Up} \longrightarrow \text{Unlock Cold Steel Skills} \longrightarrow \text{Bram's Forge Upgrades}$$
$$\Big\downarrow$$
$$\text{Harvest Aberrant Organs} \longrightarrow \text{Surgical Slab Surgery} \longrightarrow \text{Unlock Inoculated Skills} \longrightarrow \text{Raise Strain Floor}$$
$$\Big\downarrow$$
$$\text{Assemble 3 Actives + 2 Passives + 1 Keystone into Combat Loadout}$$
