# Core Gameplay Loop & UI Component Prototype Integration

> **Category:** Gameplay Experience, UI/UX Architecture & Prototype Roadmap  
> **Location:** `games/emberwake/lore/systems/gameplay_loop_and_ui_prototype.md`  
> **Related Architecture:** [rules_engine_architecture.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/rules_engine_architecture.md), [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md), [skill_acquisition_and_loadout_systems.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/skill_acquisition_and_loadout_systems.md)  
> **Existing Engine Crates:** `aegis_ui`, `emberwake_gui`, `emberwake_battle`, `emberwake_world`

---

## 1. The Core Gameplay Loop

The *Emberwake* experience operates across two tightly coupled cyclical loops:
1. **The Macro Loop (Safehouse Expedition Cycle):** Safehouse preparation $\longrightarrow$ Infiltration navigation $\longrightarrow$ Tactical combat $\longrightarrow$ Extraction & triage.
2. **The Micro Loop (The 6-Tick Tactical Timeline):** Telegraph inspection $\longrightarrow$ Maneuver scrub & commitment $\longrightarrow$ Simultaneous tick resolution $\longrightarrow$ Crisis & poise aftermath.

```text
===================================================================================================
                                      [ THE MACRO EXPEDITION LOOP ]
===================================================================================================

       [ THE SLUICE UNDERCROFT ] (Safehouse Hub)
       - Manage 6-8 Bunks (Rest, Triage, Bandages)
       - The Tinker's Hearth (Bram: Cold Steel Drills & Equipment)
       - The Apothecary Slab (Aline: Organ Grafts & Strain Surgery)
       - Safehouse Supplies (Burn daily Salt & Warded Tallow)
                               |
                               v (Select 4-Operative Cell & Loadouts)
       [ THE SUNK WARD INFILTRATION ] (Node / Grid Map)
       - Advance through street chokepoints & tallow vats
       - Avoid Inquisition Lantern Patrols (Infiltration Clock)
       - Salvage black iron slag, marrow vials, and church draughts
                               |
                               v (Encounter Triggered)
       [ TACTICAL TIMELINE COMBAT ] (The 6-Tick Arena)
       - Manage AP, Strain Gauge, and Poise Bars
       - Stagger interrupts & speed tier commitments
                               |
                               v (Victory or Tactical Withdrawal)
       [ EXTRACTION & SAFE HOUSE RETURN ]
       - Harvest Aberrant Organs from elite corpses
       - Triage wounded; track irreversible Calcification advances
       - Advance Cathedral Red Tithing Clock & Safehouse Alert Level
```

---

## 2. The Micro Combat Loop (1 Round = 6 Timeline Ticks)

Every combat encounter is resolved in discrete rounds containing **6 simultaneous ticks**. Unlike turn-based I-Go-You-Go systems, both sides act at once:

```text
+-------------------------------------------------------------------------------------------------+
| THE 4-STEP TACTICAL COMBAT MICRO-LOOP                                                           |
+-------------------------------------------------------------------------------------------------+
| Step 1: TELEGRAPH SCRUBBING (Round Start)                                                       |
| • The player inspects enemy planned actions across Ticks 1–6.                                   |
| • Enemy Windup windows and projected impact tiles are clearly indicated with warning overlays.   |
+-------------------------------------------------------------------------------------------------+
| Step 2: CELL COMMITMENT (Planning Phase)                                                        |
| • Player drafts 4 operatives' actions using AP and Strain budgets.                              |
| • Operatives assign Snap (T2 impact), Standard (T3/T4 impact), Heavy (T4/T5 impact), or Guard.  |
| • Player scrubs the Timeline UI slider to preview expected outcomes, interrupts, and hits.     |
+-------------------------------------------------------------------------------------------------+
| Step 3: SIMULTANEOUS RESOLUTION (Execution Phase)                                               |
| • The round steps through Ticks 1 to 6 automatically (or scrubbed step-by-step).                |
| • Ticks 1–2: Snap attacks land; fast strikes interrupt slow heavy enemy windups.               |
| • Ticks 3–4: Standard strikes and Active Guards resolve; armor mitigation and poise absorb blows|
| • Ticks 5–6: Heavy finishers land; spent units enter recovery windows.                          |
+-------------------------------------------------------------------------------------------------+
| Step 4: ROUND AFTERMATH & ABERRATION CHECKS                                                     |
| • Any unit at 100%+ Strain rolls on the Aberration Crisis Table (mutation, frenzy, or rupture).|
| • Poise resets toward baseline; status effects (bleed, poison, warding) tick.                   |
| • AP regenerates (+4 AP base); if enemies remain, advance to Step 1 for next round.             |
+-------------------------------------------------------------------------------------------------+
```

---

## 3. UI Component Reuse Matrix: What Already Exists in the Engine

A major finding from inspecting the codebase is that **nearly all necessary visual panels and timeline controls already exist** in `engine/crates/aegis_ui` and `games/emberwake/crates/emberwake_gui`:

| Target Emberwake UI Need | Existing Engine UI Component | Source File Location | Reuse Level & Adaptation Strategy |
| :--- | :--- | :--- | :--- |
| **6-Tick Multi-Track Timeline** | `aegis_ui::timeline::Timeline` | `engine/crates/aegis_ui/src/timeline.rs` | **95% Reused.** Already implements tracks, ruler of blocks, playhead, colored segments, and hover tooltips. Bind tracks to the 4 operatives + active enemies. |
| **Tactical Grid Battlefield** | `emberwake_gui::battle::BattleScreen` | `games/emberwake/crates/emberwake_gui/src/battle.rs` | **90% Reused.** Already renders $32 \times 32$ tile grid, isometric/2D sprites, movement range rings, target reticles, and damage flyoffs. |
| **Operative HUD (HP / AP / Strain / Poise)** | `emberwake_gui::hud` & `battle::PANELS` | `games/emberwake/crates/emberwake_gui/src/hud.rs` | **85% Reused.** Existing hero strip displays HP and AP. Add two compact bar gauges: **Poise Bar** (blue/yellow) and **Strain Gauge** (violet/red). |
| **Character & Loadout Deck Screen** | `emberwake_gui::party::PartyScreen` | `games/emberwake/crates/emberwake_gui/src/party.rs` | **90% Reused.** Already has 4 tabs: `Stats`, `Level up`, `Skills`, and `Gear`. Skills tab already supports slotting active class abilities and extras. |
| **Safehouse Hub Navigation** | `emberwake_gui::hub::Hub` | `games/emberwake/crates/emberwake_gui/src/hub.rs` | **85% Reused.** Existing hub menu over the map with party, triage, journal, and save doors. Reskin labels to Tinker's Hearth, Apothecary Slab, and Water-Lock. |
| **Overworld District Map** | `emberwake_gui::region::RegionMap` | `games/emberwake/crates/emberwake_gui/src/region.rs` | **80% Reused.** Node travel, event logs, fog of war, and encounter trigger hooks operate out-of-the-box. |
| **Combat Test Sandbox** | `emberwake_gui::dojo::Dojo` & `runner.rs` | `games/emberwake/crates/emberwake_gui/src/dojo.rs` | **100% Reused.** The engine's "Test Arena" allows launching instant combat skirmishes directly without loading save games. |

---

## 4. Visualizing the Combat UI Prototype Layout

The combat layout leverages `emberwake_gui::battle`'s existing 7-panel architecture (`"strip"`, `"party"`, `"hero"`, `"grid"`, `"hostiles"`, `"intel"`, `"bar"`):

```text
+---------------------------------------------------------------------------------------------------+
| TOP BAR: Turn 2/6 | Cathedral Red Tithing Clock: 14 Days | Inquisition Alert: 35% | [Menu] [Help]  |
+---------------------------------------------------------------------------------------------------+
| (LEFT: PARTY CELL)       | (CENTER: BATTLEFIELD GRID)                 | (RIGHT: TARGET INTEL)     |
| [1] KAELEN (Vanguard)    |                                            | INQUISITION PURIFIER      |
|     HP: 42/48 | AP: 8/10 |     [ . ][ . ][ . ][ E1][ . ]              | HP: 65/65 | Armor: 8      |
|     Poise: [========|]   |     [ . ][ P1][ . ][ . ][ . ]              | Poise: [======---|]       |
|     Strain: [==|     ]   |     [ . ][ . ][ P2][ . ][ E2]              | Intended: Heavy Cleave    |
|                          |     [ . ][ P3][ . ][ . ][ . ]              | (Impact: Tick 5 on P1)    |
| [2] VESPER (Stalker)     |     [ . ][ . ][ P4][ . ][ . ]              | Vulnerability: Bludgeon   |
|     HP: 28/32 | AP: 10   |                                            +---------------------------+
|     Poise: [=====---|]   |     Terrain: Rain-slick cobblestones       | SELECTED SKILL DECK:      |
|     Strain: [=====|  ]   |     Cover: Broken barricade (30%)          | [1] Pommel Strike (2 AP)  |
|                          |                                            | [2] Braced Poise  (1 AP)  |
| [3] MAREN (Acolyte)      |                                            | [3] Active Guard  (1 AP)  |
| [4] BRAN (Arbalestier)   |                                            | [*] Aberrant Surge(0 AP)  |
+--------------------------+--------------------------------------------+---------------------------+
| BOTTOM: THE 6-TICK MULTI-TRACK TIMELINE (aegis_ui::timeline)                                      |
| RULER:       | Tick 1  |  Tick 2   |  Tick 3   |  Tick 4   |  Tick 5   |  Tick 6   | [EXECUTE]    |
| Track: Kaelen| Move -->| [Pommel Strike (Snap)]| Active Guard ========>|           |              |
| Track: Vesper| Advance ========>| [Throat-Carver (Std)] | Recovery ====>|           | [SCRUB < >]  |
| Track: Purif.| ----- Windup ------------------>| [Cleave (Heavy)]! ===>| Recovery  |              |
+---------------------------------------------------------------------------------------------------+
```

### Timeline Visual Elements (Leveraging `aegis_ui::timeline::Segment`)
* **Windup Segments:** Rendered with a striped/diagonal hatching (`dashed: true`) in **Amber/Orange**. Indicates that the unit is telegraphing and vulnerable to Stagger.
* **Impact Tick:** Rendered with a solid diamond icon and bold **Crimson/Red** fill. This is the exact tick damage/effects land.
* **Recovery Segments:** Rendered in **Muted Slate/Gray**. Unit is locked in animation recovery.
* **Active Guard Segments:** Rendered with a solid **Steel/Gold** fill and shield icon. Unit gains $+50\%$ damage mitigation and double poise.
* **Aberrant / Inoculated Maneuvers:** Rendered with glowing **Bruised Violet** borders, indicating Strain generation.

---

## 5. Strategic Options for the Prototyping Roadmap

To validate core combat feel as quickly and smoothly as possible, we have two strategic paths for assembling the initial playable slice:

```text
+---------------------------------------------------------------------------------------------------+
| PROTOTYPE PATHWAYS                                                                                |
+----------------------------------+----------------------------------------------------------------+
| OPTION 1: THE DOJO ARENA FIRST   | OPTION 2: FULL VERTICAL SLICE                                  |
| (Recommended for rapid testing)  | (Comprehensive loop testing)                                   |
+----------------------------------+----------------------------------------------------------------+
| 1. Use existing `emberwake_gui:: | 1. Implement Prologue Step 1 (Kaelen alleyway escape).          |
|    dojo` test sandbox.           | 2. Connect RegionMap to Sunk Ward nodes.                       |
| 2. Load 1 hero (Kaelen) vs 1-2   | 3. Integrate safehouse resting loop and triage.                |
|    Inquisition foes in arena.    | 4. Test macro-progression alongside tactical combat.           |
| 3. Test:                         |                                                                |
|    • Timeline scrub & execution  |                                                                |
|    • Snap vs Heavy interrupt     |                                                                |
|    • Strain surge damage feeling |                                                                |
|                                  |                                                                |
| Time-to-Playable: FASTEST        | Time-to-Playable: LONGER                                       |
| Focus: Pure tactile combat feel  | Focus: Complete macro-progression loop                         |
+----------------------------------+----------------------------------------------------------------+
```

### Why Option 1 (Dojo Test Arena First) is the Recommended Starting Step
1. **Zero Overworld Overhead:** We don't need to hook up save games, quest flags, or dialogue trees just to test whether an attack interrupts an enemy windup on Tick 2.
2. **Immediate Feel Feedback:** We can empirically verify:
   - Does scrubbing the timeline slider feel intuitive?
   - Does interrupting a heavy enemy attack feel punchy and rewarding?
   - Does pushing into the *Flesh-Tension* Strain bracket feel like a high-stakes gamble?
3. **Smooth Transition:** Once the combat mechanics in the Dojo feel great, we plug the exact same combat session into the Act 1 campaign missions and safehouse loop.

---

## 6. Summary of UI Adjustments for the Prototype

```text
[ aegis_ui::timeline ]
  └── Bind track rows to active combatants (4 party members + hostiles).
  └── Color-code segment kinds: Amber (Windup), Red (Impact), Slate (Recovery), Gold (Guard).

[ emberwake_gui::hud / battle ]
  └── Add Poise Bar (current/max) below HP bar on hero cards.
  └── Add Strain Gauge (0–100%) with visual markers at 30%, 50%, 75%, and 100%.
  └── Wire Timeline scrubbing slider to preview planned combat states.

[ emberwake_gui::dojo ]
  └── Author 1 prototype skirmish: "The Alleyway Ambush" (Kaelen & Vesper vs. 2 Iron Inquisitors).
```
