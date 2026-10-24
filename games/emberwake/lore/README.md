# Emberwake: The Low Sleep
## Canonical Lore, Worldbuilding & Systems Master Index

> **Root Path:** `games/emberwake/lore/`  
> **Official Title:** *Emberwake: The Low Sleep* (Internal Codename: `emberwake`)  
> **Status:** Active Canonical Specification  
> **Target Engine:** Aegis Tactical RPG Engine (`engine/crates/*`, `games/emberwake/crates/*`)  
> **Primary Audience:** AI Agents, Systems Designers, and Content Implementers  

---

## 1. AI Agent Operational Guidelines

When reading, updating, or implementing features from this documentation library, all AI agents **MUST** adhere to the following rules:

1. **Strict Isolation Boundary:**  
   Do not modify existing engine crates or legacy data files (`crates/`, `docs/`, `games/emberwake/data/`, `game.json`, etc.) until an explicit refactoring or implementation phase is authorized by the user. All design bibles, architectural mappings, and narrative tracking must reside strictly inside `games/emberwake/lore/`.
2. **Canonical Authority:**  
   The specifications inside this directory supersede all legacy frontier-fantasy references (e.g. Ashen Horde, Warden Keep, elemental magic). *Emberwake* is an **eldritch grimdark tactical RPG set in an oppressive feudal theocracy**.
3. **Internal Linking Standard:**  
   Always maintain valid clickable GitHub markdown links using the `file:///` scheme and forward slashes.

---

## 2. Directory Taxonomy & Folder Structure

The documentation library is organized into four modular domains:

```text
games/emberwake/lore/
├── README.md                              <-- Master AI Index and Cheat-Sheet (This File)
│
├── specs/                                 <-- Core Specifications, Decision Logs & Visuals
│   ├── master_specification.md            <-- Raw complete design bible export
│   ├── worldbuilding_decisions.md         <-- Logged strategic decisions (DEC-LORE-01 to DEC-LORE-12)
│   ├── art_and_visual_style.md            <-- 640x360 canvas, 32x32 tiles, palette constraints
│   └── rpg_kit_and_system_architecture.md <-- Engine & RPG kit reuse assessment
│
├── background/                            <-- Setting, Cosmology, Factions & Key NPCs
│   ├── cosmology_and_setting.md           <-- The Low Sleep, The Shroud, Stagnation of Mortality, Lexicon
│   ├── factions_and_sociology.md          <-- Sovereign Clerisy, Iron Inquisition, Tallow Guild, Severed Root
│   ├── world_geography_and_hub.md         <-- Sluice Undercroft safehouse layout & Sunk Ward district map
│   ├── sunk_ward_campaign_map.md          <-- Sunk Ward node graph, traversal rules & Phase 1 campaign staging
│   └── npcs_and_questlines.md             <-- Aline Vance, Bram Karr, Tessia Morn, Pip dossiers
│
├── narrative/                             <-- Campaign Narrative, Onboarding & Character Choices
│   ├── campaign_narrative_structure.md    <-- 3-Phase dramatic arc (Cold Steel -> Sacrifice -> Red Tithing)
│   ├── narrative_choices_and_dilemmas.md  <-- Irreconcilable moral choices, sacrifice, and doctrines
│   ├── narrative_agency_and_fail_states.md<-- Resilient ensemble, emergency recruits, 3 Game Over states
│   ├── recruits_and_prologue_architecture.md<-- 3-Step episodic prologue convergence & Living Martyrs
│   ├── six_founders_and_phase_progression.md<-- The 6 Phase 1 founders, bunk limits, ideological split
│   ├── prologue_and_starter_squad_lore.md <-- In-depth backstories, breaking points, and secret mutations
│   └── new_game_onboarding_and_intro.md   <-- 30-sec illustrated vignette & cold open in the rain
│
└── systems/                               <-- Combat Mechanics, Rules Engine & UI Architecture
    ├── combat_and_timeline.md             <-- 6-Tick Simultaneous Timeline, Speed Tiers, Maneuver Blocks
    ├── magic_and_corruption.md            <-- Cold Steel vs. Inoculation, Strain Gauge, Calcification Stages
    ├── classes_and_skill_trees.md         <-- Branch A & B complete trees for all 6 base classes
    ├── skill_acquisition_and_loadout_systems.md<-- Cold Steel drills vs. Surgical slab, Active Deck (3A+2P+1K)
    ├── rules_engine_architecture.md       <-- Engine math formulas, hyperbolic armor, poise breaks, code reuse
    ├── risk_reward_and_corruption_balance.md<-- 4 axes of balance, AP density, Strain surfing economics
    ├── roster_expansion_and_lifecycle.md  <-- Bunk capacity (6 to 8), recruitment jailbreaks, permadeath
    ├── gameplay_loop_and_ui_prototype.md  <-- Macro expedition loop, micro 4-step combat loop, UI reuse
    ├── screen_architecture_and_separation_plan.md<-- Combat UI vs. Exploration UI separation & 8 screens
    ├── enemy_bestiary_and_ai.md           <-- Base/elite bestiary, named bosses & modular encounter architecture
    └── safehouse_economy_and_triage.md    <-- Undercroft wings, resource economy, and repurposed vs new UI elements
```

---

## 3. Comprehensive Document Catalog

### Domain A: Specifications & Decision Tracking (`specs/`)
* [master_specification.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)  
  *Tags:* `#spec` `#master` `#bible`  
  *Summary:* Complete verbatim source specification defining the worldbuilding transition, lore, combat mechanics, and tone.
* [worldbuilding_decisions.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/worldbuilding_decisions.md)  
  *Tags:* `#decisions` `#tracking` `#engine-mapping`  
  *Summary:* Chronological decision log (DEC-LORE-01 through DEC-LORE-12) establishing setting boundaries, mechanics shifts, and mapping matrices to engine crates.
* [art_and_visual_style.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/art_and_visual_style.md)  
  *Tags:* `#art` `#palette` `#resolution`  
  *Summary:* Viewport architecture ($640 \times 360$), $32 \times 32$ grid tiles, $32 \times 48$ sprite bounds, restricted 5-accent color palette, and telegraph visual hierarchy.
* [rpg_kit_and_system_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/rpg_kit_and_system_architecture.md)  
  *Tags:* `#architecture` `#rpg-kit` `#reuse`  
  *Summary:* Detailed audit of Aegis engine systems: reusable components (Grid, Stats, Inventory) vs. required extensions (Strain, Poise, Simultaneous Timeline).

---

### Domain B: Setting, World & Sociology (`background/`)
* [cosmology_and_setting.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/cosmology_and_setting.md)  
  *Tags:* `#cosmology` `#lore` `#the-shroud`  
  *Summary:* The slumbering cosmic leviathan (*The Low Sleep*), perpetual twilight (*The Shroud*), physiological ossification (*The Stagnation of Mortality*), and world lexicon.
* [factions_and_sociology.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/factions_and_sociology.md)  
  *Tags:* `#factions` `#clerisy` `#inquisition` `#resistance`  
  *Summary:* Power structures: The Sovereign Clerisy, Iron Inquisition, Tallow Guild, docile adult masses, and The Severed Root youth resistance cell.
* [world_geography_and_hub.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/world_geography_and_hub.md)  
  *Tags:* `#geography` `#hub` `#safehouse` `#sunk-ward`  
  *Summary:* Subterranean safehouse (*The Sluice Undercroft*), environmental hazards (tallow fumes, sewer miasma), and the Act 1 district map (*The Sunk Ward*).
* [sunk_ward_campaign_map.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/sunk_ward_campaign_map.md)  
  *Tags:* `#campaign-map` `#node-graph` `#encounters` `#prologue` `#sacrifice`  
  *Summary:* The Sunk Ward 8-node exploration graph, stealth/speed route mechanics, Inquisition Alert thresholds, and the 5 authored Phase 1 missions integrating `ENC-EVT-01` to `06`.
* [npcs_and_questlines.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/npcs_and_questlines.md)  
  *Tags:* `#npcs` `#quests` `#triage` `#forge`  
  *Summary:* Full dossiers and personal quest arcs for Sister Aline Vance, Bram Karr, Tessia Morn, and Pip.

---

### Domain C: Narrative, Onboarding & Character Arcs (`narrative/`)
* [campaign_narrative_structure.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/campaign_narrative_structure.md)  
  *Tags:* `#campaign` `#narrative` `#3-phases`  
  *Summary:* The 3-phase dramatic campaign arc (Phase 1 Pure Cold Steel $\to$ Phase 2 Crucible Sacrifice & Inoculation $\to$ Phase 3 Endgame Red Tithing Clock).
* [narrative_choices_and_dilemmas.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/narrative_choices_and_dilemmas.md)  
  *Tags:* `#choices` `#sacrifices` `#flesh-broker` `#preservationist`  
  *Summary:* Major campaign dilemmas: The Crucible founder sacrifice, Flesh-Broker vs. Preservationist doctrines, Living Martyr deployment ethics, and the Red Tithing resolution.
* [narrative_agency_and_fail_states.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/narrative_agency_and_fail_states.md)  
  *Tags:* `#permadeath` `#fail-states` `#ensemble`  
  *Summary:* Resilient ensemble design: why no single hero death ends the game, replacement recruits, and the 3 true Game Over conditions.
* [recruits_and_prologue_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/recruits_and_prologue_architecture.md)  
  *Tags:* `#prologue` `#recruitment` `#living-martyrs`  
  *Summary:* The 3-step episodic prologue convergence, safehouse guests, and captured Stage IV "Living Martyrs" as temporary shock troops.
* [six_founders_and_phase_progression.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/six_founders_and_phase_progression.md)  
  *Tags:* `#founders` `#6-bunks` `#phase-progression`  
  *Summary:* Progressive acquisition of all 6 class founders during Phase 1, filling the 6 bunks, controlled narrative Inoculation teasers, and the ideological divide.
* [prologue_and_starter_squad_lore.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/prologue_and_starter_squad_lore.md)  
  *Tags:* `#starter-squad` `#dossiers` `#kaelen` `#vesper`  
  *Summary:* Deep lore, psychological breaking points, and secret mutations for Kaelen, Vesper, Maren, and Bran, plus the hearthside respite dialogue.
* [new_game_onboarding_and_intro.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/new_game_onboarding_and_intro.md)  
  *Tags:* `#onboarding` `#new-game` `#intro`  
  *Summary:* 30-second illustrated vignette, cold open in the rain, tactile first combat encounter, and visceral faction introduction.

---

### Domain D: Combat, Systems & Engine Architecture (`systems/`)
* [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md)  
  *Tags:* `#combat` `#timeline` `#speed-tiers` `#stagger`  
  *Summary:* 6-Tick Simultaneous Timeline, Speed Tiers (Snap/Standard/Heavy), Maneuver Blocks, Poise/Stagger bar math, interrupts, and timeline UI scrubbing.
* [magic_and_corruption.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/magic_and_corruption.md)  
  *Tags:* `#strain` `#corruption` `#calcification`  
  *Summary:* The Martial Dichotomy, Strain Gauge (0–100%), Aberration Crisis Table (d100), and permanent Calcification Stages (I to V).
* [classes_and_skill_trees.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/classes_and_skill_trees.md)  
  *Tags:* `#classes` `#skill-trees` `#act-1-skills`  
  *Summary:* Complete Branch A (Cold Steel) and Branch B (Inoculated) skill trees, passives, and keystones for all 6 classes.
* [skill_acquisition_and_loadout_systems.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/skill_acquisition_and_loadout_systems.md)  
  *Tags:* `#loadouts` `#active-deck` `#skill-def`  
  *Summary:* Dual skill acquisition (Forge drills vs. Surgical slab), Active Maneuver Deck (3 Actives, 2 Passives, 1 Keystone), and engine schema extensions.
* [rules_engine_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/rules_engine_architecture.md)  
  *Tags:* `#rules-engine` `#damage-math` `#code-reuse`  
  *Summary:* Engine code reuse analysis (~85% reused), hyperbolic armor curve, real-time Strain accumulation, poise damage, and interrupt resolution.
* [risk_reward_and_corruption_balance.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/risk_reward_and_corruption_balance.md)  
  *Tags:* `#balance` `#strain-surfing` `#macro-economy`  
  *Summary:* The 4 balance axes: AP density, tactical reliability vs burst, Strain surfing risk brackets, and roster lifespan economics.
* [roster_expansion_and_lifecycle.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/roster_expansion_and_lifecycle.md)  
  *Tags:* `#roster` `#bunks` `#lifecycle`  
  *Summary:* Cell operations: starter squad (4 heroes), safehouse bunk limits (6 to 8), recruitment jailbreaks, and Stage V permadeath retirement.
* [gameplay_loop_and_ui_prototype.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/gameplay_loop_and_ui_prototype.md)  
  *Tags:* `#gameplay-loop` `#ui-prototype` `#dojo-sandbox`  
  *Summary:* Macro expedition loop, micro 4-step combat loop, UI component reuse matrix (`aegis_ui::timeline`), and the Dojo Test Arena prototype roadmap.
* [screen_architecture_and_separation_plan.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/screen_architecture_and_separation_plan.md)  
  *Tags:* `#screen-stack` `#exploration-vs-combat` `#ui-breakdown`  
  *Summary:* Architectural decoupling of Combat UI (`BattleScreen`) and Exploration UI (`DistrictMap`) via `aegis_ui::screen::Screen`, with a breakdown of all 8 game screens.
* [enemy_bestiary_and_ai.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/enemy_bestiary_and_ai.md)  
  *Tags:* `#bestiary` `#enemies` `#named-bosses` `#encounters` `#ai`  
  *Summary:* Base and elite bestiary across 4 factions, named boss variants (Captain Vane, Proctor Malakar, Orlov, Grendel, Amalgam), and 6 modular placeholder encounter events (`ENC-EVT-01` to `06`).
* [safehouse_economy_and_triage.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/safehouse_economy_and_triage.md)  
  *Tags:* `#safehouse` `#economy` `#triage` `#ui-reuse` `#deck-builder`  
  *Summary:* The Sluice Undercroft facilities (Slab, Forge, Bunks, Basin), resource conversion (Tallow, Salt, Scrap, Marrow), repurposed engine UI widgets, and 4 new UI widgets.

---

## 4. Quick-Reference System Constants

```text
+----------------------------+----------------------------------------------------------------------+
| PARAMETER                  | CANONICAL GAMEPLAY VALUE                                             |
+----------------------------+----------------------------------------------------------------------+
| Operative Eligible Age     | Ages 15–24 ("The Unsealed"). Ages 25+ are docile / calcified.         |
| Active Combat Squad        | Exactly 4 Operatives deployed per mission.                            |
| Safehouse Bunk Capacity    | 6 Bunks in Phase 1 (expandable to 8 in Phase 2 at Bram's Forge).     |
| Combat Timeline Duration   | Exactly 6 Simultaneous Ticks per round.                              |
| Action Point (AP) Baseline | 10 Max AP base, +4 AP regenerated per round.                         |
| Cold Steel Strain Cost     | Exactly 0% Strain generated (100% permanent hero longevity).         |
| Inoculated Strain Cost     | +15% to +35% Strain per maneuver execution.                          |
| Strain Surfing Brackets    | 0-29% (Clear), 30-49% (Twitch), 50-74% (Flesh-Tension), 75-99% (Rupt)|
| Aberration Crisis Trigger  | Strain >= 100% (Rolls D100 + Strain Floor - Composure).              |
| Calcification Stages       | I (Veins), II (Brittle), III (Gargoyle), IV (Martyr), V (Death/Aberr)|
| Active Maneuver Deck       | 2 Base Moves + 3 Class Actives + 2 Passive Traits + 1 Keystone Perk. |
| Grid Tile Dimensions       | 32 x 32 canvas pixels (Viewport: 640 x 360).                         |
+----------------------------+----------------------------------------------------------------------+
```
