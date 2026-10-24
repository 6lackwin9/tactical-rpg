# Worldbuilding Decisions & Architecture Log

> **Category:** Decision Tracking & Architectural Alignment  
> **Status:** Active Canon Established (2026-10-24)  
> **Location:** `games/emberwake/lore/specs/worldbuilding_decisions.md`  
> **Source Specification:** [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)

---

## 1. Summary of Worldbuilding Pivot

| Vector | Legacy Setting (Pre-Oct 2026) | New Canonical Setting (Emberwake 2.0) |
| :--- | :--- | :--- |
| **Genre** | Generic High/Low Frontier Fantasy | Medieval Feudalism / Eldritch Cosmic Horror |
| **Atmosphere** | Frontier border war, volcanic ash drifts, dry season | Perpetual bruised-violet gloom (**The Shroud**), extinguishing of celestial light, warded tallow lanterns, ringing cathedral bells |
| **Primary Adversary** | The Ashen Horde (Goblin warbands, chieftains) | The Sovereign Clerisy (Ecclesiastical feudal hierarchy, Red Tithing) & The Iron Inquisition |
| **Player Faction** | Ember Wardens (Border order holding Warden Keep) | The Severed Root (Decentralized insurgent cells of "young blood" operatives aged 15–24) |
| **Natural Laws** | Standard fantasy biology and death | **The Stagnation of Mortality:** Flesh ossifies into brittle black bone; dead do not rot without sanctified rock-salt cremation |
| **Magic System** | Standard elemental / divine spells | **The Martial Dichotomy:** Cold Steel (0% Strain, timeline maneuvers & Active Guard) vs. The Bitter Inoculation (Marrow grafts, Strain gauge, Calcification clock) |
| **Combat Model** | Standard turn-based grid rounds | **6-Tick Simultaneous Timeline** with Maneuver Blocks (Windup, Impact, Recovery), Speed Tiers, and Stagger interrupts |
| **Player Hub** | Warden Keep (Fortress courtyard) | The Sluice Undercroft (Subterranean safehouse beneath Lower Cinderreach storm-drains) |
| **Campaign Opening** | The Ashlands (Cinder Pass, Ruined Courtyard) | The Sunk Ward (Ash-Crawl Tenements, Tallow Vats, Crucible Watchpost, Slag Gate) |

---

## 2. Settled Worldbuilding Decisions

The following architectural decisions have been documented from the user's initial design session with Gemini:

### DEC-LORE-01: The Cosmic Leviathan ("The Low Sleep")
- **Decision:** The continent rests directly upon the slumbering mass of an incomprehensible cosmic titan.
- **Consequence:** Earthquakes are leviathan tremors; volcanic vents emit mutagenic black marrow; mining deep fissures causes space to distort. The ruling Clerisy maintains authority by sacrificing youths (**The Red Tithing**) to lull the titan back to sleep.

### DEC-LORE-02: Generational Divide & The Permadeath Clock
- **Decision:** Humans aged 25+ (**The Spent Generation**) naturally calcify and become docile, dependent on church communion sedatives and warded tallow fumes.
- **Consequence:** The resistance is composed exclusively of youths aged 15–24 (**The Unsealed**). Heretical power accelerates permanent Calcification through five distinct physiological stages (ending in death or aberration loss).

### DEC-LORE-03: Cold Steel vs. The Bitter Inoculation
- **Decision:** Progression offers two mechanically and philosophically opposed branches for every class:
  1. *Cold Steel:* High AP, zero strain, timeline positioning & Active Guard, 100% permanent hero longevity.
  2. *Inoculated:* Low AP, high burst/piercing damage, generates in-combat Strain (0%–100%), risks Aberration Crises, advances permanent Calcification.

### DEC-LORE-04: The 6-Tick Simultaneous Timeline
- **Decision:** Combat actions resolve in real-time across a 6-tick timeline rather than sequential turn order.
- **Consequence:** Windup windows can be staggered and interrupted; attacks are locked into Snap (T2), Standard (T4), and Heavy (T5–T6) speed tiers. Movement is integrated via Advance (move-then-act) or Disengage (act-then-move).

### DEC-LORE-05: Hub & Campaign Geography
- **Decision:** The starting haven is **The Sluice Undercroft**, featuring four functional wings:
  - The Apothecary's Slab (Sister Aline Vance — surgeries, strain flushes, apex grafts).
  - The Tinker's Hearth (Bram Karr — cold steel forging, stagger tuning, demolition).
  - The Water-Lock Basin (gear storage, submerged bone-cipher dead-drops).
  - The Dry Hollows (resting bunks, psychological quarantine).
- **Decision:** Act I regional theatre is **The Sunk Ward**, bounded by the Ash-Crawl Tenements, Tallow Vats, Crucible Watchpost, and Slag Gate.

### DEC-LORE-06: Strict Codebase & File Isolation
- **Decision:** Existing engine and game files (`data/`, `crates/`, `docs/`, `game.json`) remain completely untouched during this worldbuilding consolidation phase.
- **Consequence:** All worldbuilding files reside exclusively in `games/emberwake/lore/` to establish the canonical reference library before any mechanical refactoring takes place.

### DEC-LORE-07: The 3-Phase Campaign Dramatic Arc
- **Decision:** Campaign splits into three distinct dramatic phases:
  1. *Phase 1 (The Young Vanguard, ~4–5 hrs):* Pure Cold Steel; 6 named founders; personal world-tether questlines; concludes with a scripted tactical sacrifice at the Crucible Watchpost.
  2. *Phase 2 (The First Rupture, ~7–9 hrs):* The Bitter Inoculation unlocked at the Apothecary's Slab; Strain Gauge and Calcification Clock ignite; procedural recruits liberated from culling wagons to sustain the mission; divergence between Flesh-Broker and Preservationist philosophies.
  3. *Phase 3 (The Awakening Abyss, ~5–6 hrs):* Endgame macro-corruption; cathedral bells induce out-of-combat Strain; warded tallow shortages cause nightmare neuroses; Inquisition Alert peaks; race against the Red Tithing Doom Clock.

### DEC-LORE-08: Six Phase 1 Founders & Safehouse Bunk Capacity
- **Decision:** The Sluice Undercroft begins with **6 bunks** in The Dry Hollows, matching the six engine base classes.
- **Consequence:** All 6 classes are represented by authored founding figures in Phase 1: Kaelen (Vanguard), Vesper (Gutter-Stalker), Maren (Heretic Acolyte), Bran (Arbalestier), Torin "Slag-Jaw" (Slag-Brute), and Elyse (Ichor Alchemist).

### DEC-LORE-09: Episodic 3-Step Prologue & In Medias Res Onboarding
- **Decision:** "New Game" begins with a 30-second illustrated vignette, dropping immediately into tactical control of Kaelen in a rain-slick Seventh Ward alleyway.
- **Consequence:** A 3-step interactive prologue introduces Kaelen's deserter flight, Vesper's smoke ambush, and the rescue of Maren and Bran from an iron mortuary cart. Before Act 1 begins, an intimate hearthside respite beat at Bram's forge establishes their backstories, taints, and mutual pact.

### DEC-LORE-10: Resilient Ensemble & True Campaign Fail States
- **Decision:** No single character death triggers a Game Over. The rebellion is an insurgent cell that outlives any individual.
- **Consequence:** Game Over occurs only upon: (1) Total Cell Annihilation ($0$ units in field and $0$ reserves in safehouse); (2) Safehouse Siege Breach (Inquisition Alert hits 100% and safehouse falls); or (3) The Red Tithing Doom Clock expiring. Captured characters at Stage III/IV Calcification can be deployed as temporary "Living Martyrs" before breaking down, leaving rare Apex Grafts upon death.

### DEC-LORE-11: Streamlining Combat by Dropping Legacy Stances
- **Decision:** The legacy four-stance system (`Neutral`, `Offensive`, `Defensive`, `Tactical`) and stance-switch AP costs are dropped in favor of the **6-Tick Timeline Phases** (Windup, Impact, Recovery, Active Guard) and the **Strain Brackets**.
- **Consequence:** Eliminates redundant cognitive clutter and artificial stance-lockout rules. A unit's vulnerability and defensive profile are organically determined by: (1) their current tick phase on the timeline (vulnerable during Windup, protected during Active Guard, committed during Recovery); (2) their Poise/Stagger Bar; and (3) their active Strain bracket (e.g., Flesh-Tension adding $+2$ Armor and $-20$ Stance Stability naturally). Skills interface directly with timeline states and stagger instead of abstract stances.

### DEC-LORE-12: Rules Engine Architecture & Preferred Gameplay Baselines
- **Decision:** Maximum reuse (~85%) of existing engine crates (`emberwake_rules` stats, gear, and hyperbolic armor; `emberwake_battle` 6-block `Resolver` and grid pathing).
- **Consequence:** Three core rules options are adopted as **Preferred Baselines (Pending Gameplay Slice Review)** rather than permanently locked decisions:
  1. *Real-Time Strain Accumulation:* Maneuvers immediately add Strain upon action start to enable dynamic mid-round Strain surfing.
  2. *Binary Armor Piercing:* Inoculated attacks completely bypass physical armor rating, maintaining a visceral contrast against Cold Steel sunder/poise mechanics.
  3. *Speed Tier Impact Mapping:* Predictable telegraphing via Speed Tiers (Snap impact on T2, Standard impact on T3/T4, Heavy impact on T4/T5).
  All three baselines remain explicitly open to tuning, balancing, and adjustment based on playtest feedback once the initial playable slice is running.

### DEC-LORE-13: Canonical Title Formulation ("Emberwake: The Low Sleep")
- **Decision:** The game's official public-facing title is established as **Emberwake: The Low Sleep**.
- **Consequence:**
  - *Engineering Continuity:* Retains `emberwake` as the project codename, module prefix, and repository path (`games/emberwake/`), requiring zero directory renames.
  - *Thematic Grounding:* The subtitle (*The Low Sleep*) eliminates generic high-fantasy fire assumptions and immediately anchors the game in cosmic dread and feudal gothic horror.
  - *Lore Re-anchoring:* "Ember" is canonized as the dying, fleeting warmth of doomed youth (The Unsealed, ages 15–24) before calcifying into cold black stone; "Wake" represents both a funeral vigil over a dead world and the subterranean stirrings of the Low Sleep titan beneath Cinderreach.

---

## 3. Engine Alignment & Mapping Matrix

When the implementation phase commences, the new worldbuilding maps directly onto existing engine abstractions:

| Engine Concept (`aegis`) | Legacy Implementation | New Worldbuilding Binding |
| :--- | :--- | :--- |
| `ClassCatalog` (6 classes) | Warrior, Ranger, Rogue, Brawler, Priest, Mage | The Vanguard, The Arbalestier, The Gutter-Stalker, The Slag-Brute, The Heretic Acolyte, The Ichor Alchemist |
| `PartyState` | 3–4 hero roster | The Severed Root 4-person cell (Kaelen, Vesper, Maren, Bran) |
| `ResourceState` | HP, Mana/Stamina | HP, Action Points (AP), Strain Gauge (0–100%), Calcification Stages (I–V) |
| `LocationKind::Hub` | `warden_keep` | `the_sluice_undercroft` (Apothecary Slab, Tinker's Hearth, Water-Lock, Dry Hollows) |
| `RegionKind` | `ashen_marches`, `cinder_pass` | `the_sunk_ward` (Ash-Crawl Tenements, Tallow Vats, Crucible Watchpost, Slag Gate) |
| `FactionCatalog` | `ember_wardens`, `ashen_horde` | `severed_root`, `sovereign_clerisy`, `iron_inquisition`, `tallow_guild` |
| `Timeline / CombatSession` | Sequential initiative rounds | 6-Tick simultaneous Maneuver Blocks with Windup/Impact/Recovery |

---

## 4. Open Queries & Next Steps for the User

The following open design items have been identified for user confirmation before gameplay implementation begins:

1. **Class Trees 3–6 Full Detailing:** Classes 1 (Vanguard) and 2 (Gutter-Stalker) have complete Branch A and Branch B active/passive skill trees specified in the design bible. Would you like to draft the exact skills for Classes 3–6 (Arbalestier, Slag-Brute, Heretic Acolyte, Ichor Alchemist) within `classes_and_skill_trees.md` next?
2. **Engine Migration Strategy for Combat:** The current Rust runtime engine (`engine/combat` / `crates/`) uses a turn-based grid system. Should the 6-tick simultaneous timeline be designed as an evolution of the existing combat engine, or will we build a dedicated timeline prototype?
3. **Legacy Ashlands Data Policy:** Should existing files in `games/emberwake/data/` (Ashlands maps, goblin catalog entries) eventually be migrated/re-skinned in-place to The Sunk Ward, or should they be preserved as a legacy world module while creating a new `sunk_ward` campaign folder?
