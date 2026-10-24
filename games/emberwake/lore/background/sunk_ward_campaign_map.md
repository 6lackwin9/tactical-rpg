# The Sunk Ward Campaign Graph & Narrative Encounter Architecture

> **Category:** Campaign Level Design, Overworld Graph & Narrative Staging  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/background/sunk_ward_campaign_map.md`  
> **Reference Specs:** [world_geography_and_hub.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/background/world_geography_and_hub.md), [enemy_bestiary_and_ai.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/enemy_bestiary_and_ai.md), [campaign_narrative_structure.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/campaign_narrative_structure.md), [six_founders_and_phase_progression.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/narrative/six_founders_and_phase_progression.md)  
> **Target Crates:** `emberwake_world` (`explore`, `travel`, `state`), `emberwake_gui` (`region`, `hud`)

---

## 1. Executive Summary & Level Architecture

Act I of *Emberwake: The Low Sleep* takes place entirely within **The Sunk Ward**—the lowest, most squalid industrial quarter of Cinderreach.

The overworld campaign is structured as a **Directed Node Graph** connecting street intersections, drainage conduits, tenement rooftops, and heavily guarded church bastions. Exploration is non-linear: operatives navigate between nodes, balancing route travel costs, avoiding inquisitorial searchlight sweeps, and managing the rising **Inquisition Alert Meter**.

```text
                                [ THE SUNK WARD NODE GRAPH ]
                                             │
                                   [ 07. THE SLAG GATE ]
                                  (Exit to Upper Cinderreach)
                                             ▲
                                             │ [Route: High Gorge]
                                   [ 06. CRUCIBLE WATCHPOST ]
                                  (ENC-EVT-06: Founder Sacrifice)
                                             ▲
                      ┌──────────────────────┴──────────────────────┐
                      │ [Route: Flue Climb]                         │ [Route: Convoy Track]
            [ 04. FOURTH KILN FOUNDRY ]                   [ 05. THE TALLOW VATS ]
           (ENC-EVT-03: Free Torin)                      (ENC-EVT-04: Orlov / Fuel)
                      ▲                                             ▲
                      │ [Route: Pipe Duct]                          │ [Route: Slum Alleys]
            [ 02. ALLEYWAY JUNCTION 7 ]                   [ 03. ASH-CRAWL CROSSROADS ]
           (ENC-EVT-01: Ambush / Vane)                   (ENC-EVT-02: Mortuary Cart)
                      ▲                                             ▲
                      │ [Route: Water Siphon]                       │ [Route: Drain Flue]
                      └──────────────────────┬──────────────────────┘
                                             │
                                   [ 01. DROWNED RUNOFF GRATE ]
                                  (Egress / Checkpoint Ingress)
                                             ▲
                                             │ [Submerged Sluice Gate]
                                   [ 00. SLUICE UNDERCROFT ]
                                  (ENC-EVT-05: Fissure Crisis)
```

---

## 2. Overworld Navigation & Exploration Mechanics

Traversing the Sunk Ward uses the engine's node-travel runtime (`emberwake_world::travel` and `explore`):

### 2.1 Route Travel Mechanics
* **Action & Supply Costs:** Moving along a route consumes **1 Time Unit** and has a baseline road-encounter chance ($15\%$ standard, $+20\%$ if traveling while unsupplied).
* **Stealth Travel vs. Forced March:**
  - *Stealth Infiltration (Default):* Operatives move under cover of fog and chimney soot. Low encounter risk, but takes 1 additional time tick.
  - *Forced March:* Rapid traversal. Saves time, but increases Inquisition Alert by $+5\%$ per leg and guarantees a roamer encounter if alert is $\ge 50\%$.

### 2.2 The Inquisition Alert Meter & Detection Cones
The citywide **Inquisition Alert Meter** ($0\%$ to $100\%$) tracks the regime's search intensity:
* **Detection Cones (Searchlights & Curfew Patrols):** On the District Map, certain routes are swept by rotating yellow searchlight cones from watchtowers. Entering a node while illuminated immediately triggers an ambush encounter and spikes alert by $+10\%$.
* **Alert Thresholds:**
  - *0%–30% (Guarded):* Standard watchmen patrols; routes are largely open.
  - *31%–70% (Suspicious):* Checkpoint gates locked; Castellan Footmen guard bridge nodes; tolls required to pass without combat.
  - *71%–99% (Lockdown):* Roaming Render-Beast hunter packs on all main thoroughfares; searchlights sweep every 2 ticks.
  - *100% (Safehouse Breach):* Triggers **ENC-EVT-05** as an immediate hostile siege breach on the Sluice Undercroft.

---

## 3. The Phase 1 Narrative Campaign: Integrating Placeholder Encounters

The six modular encounter events defined in [enemy_bestiary_and_ai.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/enemy_bestiary_and_ai.md) (`ENC-EVT-01` to `06`) form the backbone of the Phase 1 narrative progression. Each mission introduces a new tactical layer, recruits a new founding class specialist, and escalates the macro-stakes:

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| PHASE 1 MISSION       | PRIMARY MAP NODE            | INTEGRATED ENCOUNTER EVENT    | NARRATIVE PROGRESSION PAYOFF     |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Mission 1: The Flight | Node 02: Alleyway Junction 7| ENC-EVT-01 (Captain Vane)     | Kaelen & Vesper reach safehouse. |
| Mission 2: The Heist  | Node 03: Ash-Crawl Cross.   | ENC-EVT-02 (Proctor Malakar)  | Rescue Maren (Priest) & Bran (Rng)|
| Mission 3: The Kiln   | Node 04: Fourth Kiln Foundry| ENC-EVT-03 (Overseer Grendel) | Rescue Torin (Slag-Brute / Brawler)|
| Mission 4: The Purge  | Node 05: The Tallow Vats    | ENC-EVT-04 (Guild-Master Orlov)| Rescue Elyse (Alchemist / Mage). |
| Crisis: The Fissure   | Node 00: Sub-Sluice Catac.  | ENC-EVT-05 (Calcified Amalgam)| Emergency Incursion / Marrow Core.|
| Mission 5: The Climax | Node 06: Crucible Watchpost | ENC-EVT-06 (Inquisitor Bastion)| Scripted Founder Tactical Sacrifice|
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

### 3.1 Mission 1: The Deserter's Flight (Prologue Step 1 & 2)
* **Primary Node:** `Node 02: Alleyway Junction 7` (Seventh Ward Curfew District).
* **Narrative Context:** Kaelen deserts the Seventh Ward Watch after refusing to execute a pen of condemned youths. Pursued into the rain-slick alleys by his former comrades, he is ambushed. Vesper ambushes the pursuers with chimney soot, forming a desperate pact.
* **Integrated Encounter:** **`ENC-EVT-01: Seventh Ward Alleyway Ambush`**.
  - *Adversaries:* Captain Vane (scaled to tutorial skirmish) + 2 Castellan Footmen.
  - *Tactical Focus:* Teaching the **6-Tick Timeline**, Snap interrupts (Kaelen's *Pommel Strike* on T2 cancelling a footman's T4 thrust), and flanking backstabs with Vesper.
* **Harvest & Safehouse Ingress:** Defeating Vane's vanguard forces him to retreat. Vesper leads Kaelen through `Node 01: Drowned Runoff Grate` into the Sluice Undercroft, unlocking the safehouse hub and meeting Sister Aline and Bram Karr.

---

### 3.2 Mission 2: The Mortuary Cart Hijack (Prologue Step 3)
* **Primary Node:** `Node 03: Ash-Crawl Crossroads`.
* **Narrative Context:** Street informants report an Inquisition prisoner-transport cart moving through the tenements, carrying condemned heretics to the cathedral dissection slabs. Among the captives are Sister Maren (a defrocked scribe with medical training) and Bran (a tollhouse poacher).
* **Integrated Encounter:** **`ENC-EVT-02: The Mortuary Cart Hijack`**.
  - *Adversaries:* Proctor Malakar + 1 Inquisitorial Purifier + 2 Mortuary Flagellants.
  - *Dynamic Objective:* The Flagellants execute 1 captive youth every 2 rounds. The squad must rush through narrow tenement stalls to break the cage locks.
  - *Tactical Focus:* Prioritizing fast targets, dodging Purifier phosphorus ground hazards, and ranged suppression.
* **Roster Expansion:** Unlocks **Sister Maren (The Heretic Acolyte)** and **Bran (The Arbalestier)**, filling Bunks 3 and 4 in the Dry Hollows. Unlocks the 4-man squad deployment rule.

---

### 3.3 Mission 3: The Bone-Mill Crucible Raid
* **Primary Node:** `Node 04: Fourth Kiln Foundry`.
* **Narrative Context:** The Inquisition is constructing fortified watch-bastions using mortar tempered with calcified human bone pulverized in the Fourth Kiln. The cell infiltrates the factory to destroy the pulverizing machinery and liberate the penal chain gang.
* **Integrated Encounter:** **`ENC-EVT-03: The Bone-Mill Crucible Raid`**.
  - *Adversaries:* Overseer Grendel + 2 Tallow Dredgers + 1 Bailiff Enforcer.
  - *Dynamic Hazard:* Moving industrial conveyor belts pushing combatants toward crushing rollers; bursting steam vents venting boiling steam on T3 and T6.
  - *Tactical Focus:* Environmental positioning, knockback wall slams, and managing armored heavy enemies.
* **Roster Expansion:** Unlocks **Torin "Slag-Jaw" (The Slag-Brute)**, filling Bunk 5. Bram Karr's forge unlocks heavy armor reinforcement recipes using salvaged foundry scrap.

---

### 3.4 Mission 4: The Cellar Purge & Tallow Sabotage
* **Primary Node:** `Node 05: The Tallow Vats`.
* **Narrative Context:** Cathedral purge teams and Tallow Guild mercenaries are cleansing the tenement cellars with scalding grease. Elyse, an underground hedge chemist brewing counterfeit tallow, is cornered in her distillery beneath the rendering caldrons.
* **Integrated Encounter:** **`ENC-EVT-04: The Tallow Vat Incursion`**.
  - *Adversaries:* Guild-Master Orlov + 2 Render-Beasts + 2 Tallow Dredgers.
  - *Dynamic Hazard:* Narrow wooden cat-walks over boiling fat vats; grease puddles that make units Flammable and reduce movement.
  - *Tactical Focus:* Managing fast beast auto-chase pounces, chemical fire reactions, and Vesper's personal vendetta mechanics against Orlov.
* **Roster Expansion:** Unlocks **Elyse (The Ichor Alchemist)**, filling Bunk 6. The Six Founders are now fully assembled! Unlocks safehouse tallow fuel raids.

---

### 3.5 Emergency Interlude: The Awakening Fissure
* **Primary Node:** `Node 00: Sluice Undercroft (Sub-Basin Catacombs)`.
* **Narrative Context:** A deep subterranean tremor shakes Cinderreach. The stone foundation beneath the Undercroft splits open, venting mutagenic black marrow and disgorging an eldritch horror from the Low Sleep.
* **Integrated Encounter:** **`ENC-EVT-05: The Awakening Fissure`**.
  - *Adversaries:* The Calcified Amalgam + 3 Ash-Crawlers + 2 Calcified Husks.
  - *Dynamic Crisis:* The chamber pulses with the titan's heartbeat; operatives passively gain +5% Strain per round.
  - *The Inoculation Teaser:* Against the Amalgam's impenetrable bone carapace, Sister Aline administers a single experimental dose of raw extracted marrow to one operative for 1 round. The hero gains +100% true damage, shattering the beast, but collapses with blackened veins (Stage I Calcification), demonstrating the terrifying double-edged bargain of the marrow.
* **Strategic Payoff:** Yields the first *Apex Marrow Core*, establishing the narrative foundation for Phase 2's Inoculation unlocking.

---

### 3.6 Mission 5: The Crucible Watchpost Climax (The Scripted Sacrifice)
* **Primary Node:** `Node 06: Crucible Watchpost` $\to$ `Node 07: The Slag Gate`.
* **Narrative Context:** With the six founders assembled, the cell launches an all-out assault on the Crucible Watchpost to blow the Slag Gate and break the cordon separating the Sunk Ward from the High Cathedral district.
* **Integrated Encounter:** **`ENC-EVT-06: The Crucible Watchpost Climax`**.
  - *Adversaries:* High Inquisitor Bastion + 2 Inquisitorial Purifiers + 2 Castellan Footmen + 2 Watch Arbalestiers.
  - *The Climax Mechanism:* The portcullis winch is jammed by iron bolts. To open the Slag Gate and allow the squad to escape the collapsing ward, **one operative must remain behind on the winch tile**, operating the lever while exposed to heavy enemy fire.
  - *The Dramatic Choice:* The player chooses which of the six founders makes the ultimate sacrifice. The chosen hero holds the gate, buying time for their comrades before succumbing to Bastion’s executioners.
* **The Phase Transition:**
  - Concludes **Phase 1 (The Young Vanguard)**.
  - The remaining 5 founders escape through the Slag Gate.
  - Unlocks **Phase 2 (The Transhuman Rupture)**: Sister Aline's Apothecary Slab opens for surgical grafts, the Bitter Inoculation is unlocked, procedural recruits are liberated from culling wagons to fill vacant bunks, and the ideological divide between *Flesh-Brokers* and *Preservationists* begins.

---

## 4. Narrative Modularity & Side Quest Decoupling

Because the overarching campaign script is intentionally modular, this architecture allows narrative elements to shift seamlessly:

```text
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENCOUNTER EVENT       | PRIMARY MAIN QUEST ROLE     | MODULAR SIDE QUEST / CRISIS FALLBACK ROLE             |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-EVT-01 (Vane)     | Prologue Deserter Flight    | Optional Bounty: Hunt Captain Vane in the ruins to    |
|                       |                             | lower citywide Inquisition Alert by -25%.             |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-EVT-02 (Malakar)  | Mortuary Cart Rescue        | Procedural Prison Raid: Liberate 2 procedural youths  |
|                       | (Maren & Bran)              | during Phase 2 to replenish lost casualties.          |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-EVT-03 (Grendel)  | Fourth Kiln Foundry Raid    | Industrial Sabotage Side Mission: Disable cement      |
|                       | (Torin Rescue)              | production to reduce fortress wall defenses.          |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-EVT-04 (Orlov)    | Tallow Vat Purge            | Vesper's Personal Loyalty Quest: Settle family blood  |
|                       | (Elyse Rescue)              | feud and seize 10 rations of safehouse tallow fuel.   |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-EVT-05 (Amalgam)  | Safehouse Incursion Teaser  | Macro Crisis Event: Triggers dynamically if citywide  |
|                       | (Marrow Harvest)            | Inquisition Alert hits 100% (Safehouse Siege).        |
+-----------------------+-----------------------------+-------------------------------------------------------+
| ENC-EVT-06 (Bastion)  | Crucible Watchpost Climax   | Permanent Narrative Milestone: The transition trigger |
|                       | (Phase 1 Sacrifice)         | from Cold Steel to Inoculation.                       |
+-----------------------+-----------------------------+-------------------------------------------------------+
```

---

## 5. Engine Data Integration (`world.json` & `routes.json`)

To map this graph into `games/emberwake/crates/emberwake_world`, the nodes and routes bind to existing catalog schemas:

```json
{
  "locations": [
    {
      "id": "sunk_ward_sluice_undercroft",
      "name": "The Sluice Undercroft",
      "kind": "hub",
      "facilities": ["apothecary_slab", "tinker_hearth", "dry_hollows", "water_lock"]
    },
    {
      "id": "sunk_ward_alleyway_junction_7",
      "name": "Seventh Ward Alleyway Junction",
      "kind": "exploration",
      "encounter_ref": "ENC-EVT-01",
      "hazard_type": "rain_slick_cobbles"
    },
    {
      "id": "sunk_ward_ash_crawl_crossroads",
      "name": "Ash-Crawl Crossroads",
      "kind": "exploration",
      "encounter_ref": "ENC-EVT-02",
      "hazard_type": "tenement_chokepoints"
    },
    {
      "id": "sunk_ward_fourth_kiln_foundry",
      "name": "Fourth Kiln Foundry",
      "kind": "exploration",
      "encounter_ref": "ENC-EVT-03",
      "hazard_type": "steam_vents_conveyors"
    },
    {
      "id": "sunk_ward_tallow_vats",
      "name": "The Tallow Vats",
      "kind": "exploration",
      "encounter_ref": "ENC-EVT-04",
      "hazard_type": "scalding_corpse_fat"
    },
    {
      "id": "sunk_ward_crucible_watchpost",
      "name": "The Crucible Watchpost",
      "kind": "dungeon_climax",
      "encounter_ref": "ENC-EVT-06",
      "hazard_type": "siege_portcullis"
    }
  ]
}
```
